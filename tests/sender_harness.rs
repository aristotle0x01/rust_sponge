use rust_sponge::tcp_helpers::tcp_config::TCPConfig;
use rust_sponge::tcp_helpers::tcp_segment::TCPSegment;
use rust_sponge::tcp_helpers::tcp_state::TCPState;
use rust_sponge::tcp_sender::TCPSender;
use rust_sponge::wrapping_integers::WrappingInt32;
use rust_sponge::SizeT;
use std::cmp::min;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub const DEFAULT_TEST_WINDOW: u32 = 137;

pub trait SenderTestStep {
    fn execute(&self, sender: &mut TCPSender, segments: &mut VecDeque<TCPSegment>);
}

pub trait SenderExpectation: SenderTestStep {
    fn description(&self) -> String;
    fn to_string(&self) -> String {
        String::from(format!("Expectation: {}", self.description()))
    }
}

pub struct ExpectState {
    state: String,
}
impl SenderTestStep for ExpectState {
    fn execute(&self, sender: &mut TCPSender, _segments: &mut VecDeque<TCPSegment>) {
        println!("  step: {}", SenderExpectation::to_string(self));

        let b = TCPState::state_summary_sender(sender) == self.state;
        assert!(
            b,
            "The TCPSender was in state `{}`, but it was expected to be in state `{}`",
            TCPState::state_summary_sender(sender),
            self.state
        );
    }
}
impl SenderExpectation for ExpectState {
    fn description(&self) -> String {
        format!("in state `{}`", self.state)
    }
}
impl ExpectState {
    #[allow(dead_code)]
    pub fn new(s: String) -> ExpectState {
        ExpectState { state: s }
    }
}

pub struct ExpectSeqno {
    seqno: WrappingInt32,
}
impl SenderTestStep for ExpectSeqno {
    fn execute(&self, sender: &mut TCPSender, _segments: &mut VecDeque<TCPSegment>) {
        println!("  step: {}", SenderExpectation::to_string(self));

        let b = sender.next_seqno() == self.seqno;
        assert!(
            b,
            "The TCPSender reported that the next seqno is `{}`, but it was expected to be `{}`",
            sender.next_seqno().raw_value(),
            self.seqno.raw_value()
        );
    }
}
impl SenderExpectation for ExpectSeqno {
    fn description(&self) -> String {
        format!("next seqno  {}", self.seqno.raw_value())
    }
}
impl ExpectSeqno {
    #[allow(dead_code)]
    pub fn new(op: WrappingInt32) -> ExpectSeqno {
        ExpectSeqno { seqno: op }
    }
}

pub struct ExpectBytesInFlight {
    n_bytes: SizeT,
}
impl SenderTestStep for ExpectBytesInFlight {
    fn execute(&self, sender: &mut TCPSender, _segments: &mut VecDeque<TCPSegment>) {
        println!("  step: {}", SenderExpectation::to_string(self));

        let b = sender.bytes_in_flight() == self.n_bytes;
        assert!(
            b,
            "The TCPSender reported `{}` bytes in flight, but it was expected to be `{}` bytes in flight",
            sender.bytes_in_flight(),
            self.n_bytes
        );
    }
}
impl SenderExpectation for ExpectBytesInFlight {
    fn description(&self) -> String {
        format!("{} bytes in flight", self.n_bytes)
    }
}
impl ExpectBytesInFlight {
    #[allow(dead_code)]
    pub fn new(w: SizeT) -> ExpectBytesInFlight {
        ExpectBytesInFlight { n_bytes: w }
    }
}

pub struct ExpectNoSegment {}
impl SenderTestStep for ExpectNoSegment {
    fn execute(&self, _sender: &mut TCPSender, _segments: &mut VecDeque<TCPSegment>) {
        println!("  step: {}", SenderExpectation::to_string(self));

        if !_segments.is_empty() {
            let seg = _segments.back().unwrap();
            let s = format!("The TCPSender sent a segment, but should not have. Segment info:\n\t{} with {} bytes", seg.header().summary(), seg.payload().size());
            assert!(false, "{}", s);
        }
    }
}
impl SenderExpectation for ExpectNoSegment {
    fn description(&self) -> String {
        "no (more) segments".to_string()
    }
}

pub trait SenderAction: SenderTestStep {
    fn description(&self) -> String;
    fn to_string(&self) -> String {
        String::from(format!("Action:      {}", self.description()))
    }
}

pub struct WriteBytes {
    bytes: String,
    end_input: bool,
}
impl SenderTestStep for WriteBytes {
    fn execute(&self, sender: &mut TCPSender, _segments: &mut VecDeque<TCPSegment>) {
        println!("  step: {}", SenderAction::to_string(self));

        sender
            .stream_in_mut()
            .write(&self.bytes.clone().into_bytes());
        if self.end_input {
            sender.stream_in_mut().end_input();
        }
        sender.fill_window();
    }
}
impl SenderAction for WriteBytes {
    fn description(&self) -> String {
        format!(
            "write bytes: \"{}{}\"{}",
            if self.bytes.len() >= 16 {
                self.bytes[..16].to_string()
            } else {
                self.bytes[..self.bytes.len()].to_string()
            },
            if self.bytes.len() > 16 { "..." } else { "" },
            if self.end_input { " + EOF" } else { "" }
        )
    }
}
impl WriteBytes {
    #[allow(dead_code)]
    pub fn new(_bytes: String) -> WriteBytes {
        WriteBytes {
            bytes: _bytes,
            end_input: false,
        }
    }

    #[allow(dead_code)]
    pub fn with_end_input(&mut self, _end_input: bool) -> &WriteBytes {
        self.end_input = _end_input;
        self
    }
}

pub struct Tick {
    ms: SizeT,
    max_retx_exceeded: Option<bool>,
}
impl SenderTestStep for Tick {
    fn execute(&self, sender: &mut TCPSender, _segments: &mut VecDeque<TCPSegment>) {
        println!("  step: {}", SenderAction::to_string(self));

        sender.tick(self.ms);
        if self.max_retx_exceeded.is_some()
            && self.max_retx_exceeded.unwrap()
                != (sender.consecutive_retransmissions() > TCPConfig::MAX_RETX_ATTEMPTS)
        {
            let mut ss = String::new();
            ss.push_str(&*format!("after {}ms passed the TCP Sender reported\n\tconsecutive_retransmissions = {}\nbut it should have been\n\t", self.ms, sender.consecutive_retransmissions()));
            if self.max_retx_exceeded.unwrap() {
                ss.push_str("greater than ");
            } else {
                ss.push_str("less than or equal to ");
            }
            ss.push_str(&*format!("{}\n", TCPConfig::MAX_RETX_ATTEMPTS));
            assert!(false, "{}", ss);
        }
    }
}
impl SenderAction for Tick {
    fn description(&self) -> String {
        format!(
            "{} ms pass{}",
            self.ms,
            if self.max_retx_exceeded.is_some() {
                format!(
                    " with max_retx_exceeded = {}",
                    self.max_retx_exceeded.unwrap()
                )
            } else {
                "".to_string()
            }
        )
    }
}
impl Tick {
    #[allow(dead_code)]
    pub fn new(_ms: SizeT) -> Tick {
        Tick {
            ms: _ms,
            max_retx_exceeded: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_max_retx_exceeded(&mut self, max_retx_exceeded_: bool) -> &Tick {
        let _ = self.max_retx_exceeded.insert(max_retx_exceeded_);
        self
    }
}

pub struct AckReceived {
    ackno: WrappingInt32,
    window_advertisement: Option<u16>,
}
impl SenderTestStep for AckReceived {
    fn execute(&self, sender: &mut TCPSender, _segments: &mut VecDeque<TCPSegment>) {
        println!("  step: {}", SenderAction::to_string(self));

        sender.ack_received(
            self.ackno,
            self.window_advertisement
                .unwrap_or(DEFAULT_TEST_WINDOW as u16),
        );
        sender.fill_window();
    }
}
impl SenderAction for AckReceived {
    fn description(&self) -> String {
        format!(
            "ack {} winsize {}",
            self.ackno.raw_value(),
            self.window_advertisement
                .unwrap_or(DEFAULT_TEST_WINDOW as u16)
        )
    }
}
impl AckReceived {
    #[allow(dead_code)]
    pub fn new(_ackno: WrappingInt32) -> AckReceived {
        AckReceived {
            ackno: _ackno,
            window_advertisement: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_win(&mut self, win: u16) -> &AckReceived {
        let _ = self.window_advertisement.insert(win);
        self
    }
}

pub struct Close {}
impl SenderTestStep for Close {
    fn execute(&self, sender: &mut TCPSender, _segments: &mut VecDeque<TCPSegment>) {
        println!("  step: {}", SenderAction::to_string(self));

        sender.stream_in_mut().end_input();
        sender.fill_window();
    }
}
impl SenderAction for Close {
    fn description(&self) -> String {
        "close".to_string()
    }
}

pub struct ExpectSegment {
    ack: Option<bool>,
    rst: Option<bool>,
    syn: Option<bool>,
    fin: Option<bool>,
    seqno: Option<WrappingInt32>,
    ackno: Option<WrappingInt32>,
    win: Option<u16>,
    payload_size: Option<SizeT>,
    data: Option<String>,
}
impl SenderTestStep for ExpectSegment {
    fn execute(&self, _sender: &mut TCPSender, _segments: &mut VecDeque<TCPSegment>) {
        println!("  step: {}", SenderExpectation::to_string(self));

        assert!(!_segments.is_empty(), "existed");
        let t_ = _segments.pop_front().unwrap();
        let seg = t_;
        if self.ack.is_some() && seg.header().ack != self.ack.unwrap() {
            let f = self.violated_field(
                "ack",
                &*format!("{}", self.ack.unwrap()),
                &*format!("{}", seg.header().ack),
            );
            assert!(false, "{}", f);
        }
        if self.rst.is_some() && seg.header().rst != self.rst.unwrap() {
            let f = self.violated_field(
                "rst",
                &*format!("{}", self.rst.unwrap()),
                &*format!("{}", seg.header().rst),
            );
            assert!(false, "{}", f);
        }
        if self.syn.is_some() && seg.header().syn != self.syn.unwrap() {
            let f = self.violated_field(
                "syn",
                &*format!("{}", self.syn.unwrap()),
                &*format!("{}", seg.header().syn),
            );
            assert!(false, "{}", f);
        }
        if self.fin.is_some() && seg.header().fin != self.fin.unwrap() {
            let f = self.violated_field(
                "fin",
                &*format!("{}", self.fin.unwrap()),
                &*format!("{}", seg.header().fin),
            );
            assert!(false, "{}", f);
        }
        if self.seqno.is_some() && seg.header().seqno.raw_value() != self.seqno.unwrap().raw_value()
        {
            let f = self.violated_field(
                "seqno",
                &*format!("{}", self.seqno.unwrap().raw_value()),
                &*format!("{}", seg.header().seqno.raw_value()),
            );
            assert!(false, "{}", f);
        }
        if self.ackno.is_some() && seg.header().ackno.raw_value() != self.ackno.unwrap().raw_value()
        {
            let f = self.violated_field(
                "ackno",
                &*format!("{}", self.ackno.unwrap().raw_value()),
                &*format!("{}", seg.header().ackno.raw_value()),
            );
            assert!(false, "{}", f);
        }
        if self.win.is_some() && seg.header().win != self.win.unwrap() {
            let f = self.violated_field(
                "win",
                &*format!("{}", self.win.unwrap()),
                &*format!("{}", seg.header().win),
            );
            assert!(false, "{}", f);
        }
        if self.payload_size.is_some() && seg.payload().size() != self.payload_size.unwrap() {
            let f = self.violated_field(
                "payload_size",
                &*format!("{}", self.payload_size.unwrap()),
                &*format!("{}", seg.payload().size()),
            );
            assert!(false, "{}", f);
        }
        if seg.payload().size() > TCPConfig::MAX_PAYLOAD_SIZE {
            let f = format!(
                "packet has length ({}) greater than the maximum",
                seg.payload().size()
            );
            assert!(false, "{}", f);
        }
        if self.data.is_some()
            && &String::from_utf8(seg.payload().str().to_vec()).unwrap()
                != self.data.as_ref().unwrap()
        {
            let f = format!(
                "payloads differ. expected \"{}\" but found \"{}\"",
                seg.payload().size(),
                String::from_utf8(seg.payload().str().to_vec()).unwrap()
            );
            assert!(false, "{}", f);
        }
    }
}
impl SenderExpectation for ExpectSegment {
    fn description(&self) -> String {
        format!("segment sent with {}", self.segment_description())
    }
}
impl ExpectSegment {
    pub fn violated_field(
        &self,
        field_name: &str,
        expected_value: &str,
        actual_value: &str,
    ) -> String {
        format!(
            "The Sender produced a segment with `{} = {}`, but {} was expected to be `{}`",
            field_name, actual_value, field_name, expected_value
        )
    }

    #[allow(dead_code)]
    pub fn new() -> ExpectSegment {
        ExpectSegment {
            ack: None,
            rst: None,
            syn: None,
            fin: None,
            seqno: None,
            ackno: None,
            win: None,
            payload_size: None,
            data: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_ack(&mut self, ack: bool) -> &mut ExpectSegment {
        let _ = self.ack.insert(ack);
        self
    }

    #[allow(dead_code)]
    pub fn with_rst(&mut self, rst: bool) -> &mut ExpectSegment {
        let _ = self.rst.insert(rst);
        self
    }

    #[allow(dead_code)]
    pub fn with_syn(&mut self, syn: bool) -> &mut ExpectSegment {
        let _ = self.syn.insert(syn);
        self
    }

    #[allow(dead_code)]
    pub fn with_fin(&mut self, fin: bool) -> &mut ExpectSegment {
        let _ = self.fin.insert(fin);
        self
    }

    #[allow(dead_code)]
    pub fn with_no_flags(&mut self) -> &mut ExpectSegment {
        let _ = self.ack.insert(false);
        let _ = self.fin.insert(false);
        let _ = self.rst.insert(false);
        let _ = self.syn.insert(false);
        self
    }

    #[allow(dead_code)]
    pub fn with_seqno(&mut self, seqno_: WrappingInt32) -> &mut ExpectSegment {
        let _ = self.seqno.insert(seqno_);
        self
    }

    #[allow(dead_code)]
    pub fn with_seqno_32(&mut self, seqno_: u32) -> &mut ExpectSegment {
        let _ = self.seqno.insert(WrappingInt32::new(seqno_));
        self
    }

    #[allow(dead_code)]
    pub fn with_ackno(&mut self, ackno_: WrappingInt32) -> &mut ExpectSegment {
        let _ = self.ackno.insert(ackno_);
        self
    }

    #[allow(dead_code)]
    pub fn with_ackno_32(&mut self, ackno_: u32) -> &mut ExpectSegment {
        let _ = self.ackno.insert(WrappingInt32::new(ackno_));
        self
    }

    #[allow(dead_code)]
    pub fn with_win(&mut self, win_: u16) -> &mut ExpectSegment {
        let _ = self.win.insert(win_);
        self
    }

    #[allow(dead_code)]
    pub fn with_payload_size(&mut self, payload_size_: SizeT) -> &mut ExpectSegment {
        let _ = self.payload_size.insert(payload_size_);
        self
    }

    #[allow(dead_code)]
    pub fn with_data(&mut self, data_: String) -> &mut ExpectSegment {
        let _ = self.data.insert(data_);
        self
    }

    #[allow(dead_code)]
    pub fn segment_description(&self) -> String {
        let mut o = String::new();
        o.push_str("(");
        if self.ack.is_some() {
            o.push_str(if self.ack.unwrap() { "A=1," } else { "A=0," });
        }
        if self.rst.is_some() {
            o.push_str(if self.rst.unwrap() { "R=1," } else { "R=0," });
        }
        if self.syn.is_some() {
            o.push_str(if self.syn.unwrap() { "S=1," } else { "S=0," });
        }
        if self.fin.is_some() {
            o.push_str(if self.fin.unwrap() { "F=1," } else { "F=0," });
        }
        if self.ackno.is_some() {
            o.push_str(&*format!("ackno={},", self.ackno.unwrap()));
        }
        if self.win.is_some() {
            o.push_str(&*format!("win={},", self.win.unwrap()));
        }
        if self.seqno.is_some() {
            o.push_str(&*format!("seqno={},", self.seqno.unwrap()));
        }
        if self.payload_size.is_some() {
            o.push_str(&*format!("payload_size={},", self.payload_size.unwrap()));
        }
        if self.data.is_some() {
            o.push_str("\"");
            let l = min(self.data.as_ref().unwrap().len(), 16);
            for _i in 0..l {
                o.push_str(&*format!("{}", self.data.as_ref().unwrap().as_bytes()[_i]));
            }
            if self.data.as_ref().unwrap().len() > 16 {
                o.push_str("...");
            }
            o.push_str("\",");
        }
        o.push_str("...)");
        o.to_string()
    }
}

pub struct TCPSenderTestHarness {
    sender: TCPSender,
    outbound_segments: VecDeque<TCPSegment>,
    name: String,
}
impl TCPSenderTestHarness {
    #[allow(dead_code)]
    pub fn new(name_: String, config: &TCPConfig) -> TCPSenderTestHarness {
        let mut harness = TCPSenderTestHarness {
            sender: TCPSender::new(config.send_capacity, config.rt_timeout, config.fixed_isn),
            outbound_segments: Default::default(),
            name: name_,
        };
        harness.sender.fill_window();
        harness.collect_output();

        println!(
            "test:=> Initialized {} (retx-timeout={} ) and called fill_window()",
            harness.name, config.rt_timeout
        );
        harness
    }

    #[allow(dead_code)]
    pub fn execute(&mut self, step: &dyn SenderTestStep) {
        step.execute(&mut self.sender, &mut self.outbound_segments);
        self.collect_output();
    }

    fn collect_output(&mut self) {
        while !self.sender.segments_out_mut().is_empty() {
            self.outbound_segments
                .push_back(self.sender.segments_out_mut().pop_front().unwrap());
        }
    }
}
