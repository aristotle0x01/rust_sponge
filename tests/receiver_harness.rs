use rust_sponge::tcp_helpers::tcp_header::TCPHeader;
use rust_sponge::tcp_helpers::tcp_segment::TCPSegment;
use rust_sponge::tcp_helpers::tcp_state::TCPState;
use rust_sponge::tcp_receiver::TCPReceiver;
use rust_sponge::util::buffer::Buffer;
use rust_sponge::wrapping_integers::WrappingInt32;
use rust_sponge::SizeT;

pub trait ReceiverTestStep {
    fn execute(&self, bs: &mut TCPReceiver);
}

pub trait ReceiverExpectation: ReceiverTestStep {
    fn description(&self) -> String;
    fn to_string(&self) -> String {
        String::from(format!("Expectation: {}", self.description()))
    }
}

pub struct ExpectState {
    state: String,
}
impl ReceiverTestStep for ExpectState {
    fn execute(&self, receiver: &mut TCPReceiver) {
        println!("  step: {}", ReceiverExpectation::to_string(self));

        let b = TCPState::state_summary(receiver) == self.state;
        assert!(
            b,
            "The TCPReceiver was in state `{}`, but it was expected to be in state `{}`",
            TCPState::state_summary(receiver),
            self.state
        );
    }
}
impl ReceiverExpectation for ExpectState {
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

pub struct ExpectAckno {
    ackno: Option<WrappingInt32>,
}
impl ReceiverTestStep for ExpectAckno {
    fn execute(&self, receiver: &mut TCPReceiver) {
        println!("  step: {}", ReceiverExpectation::to_string(self));

        let b = receiver.ackno() == self.ackno;
        let reported = format!(
            "{}",
            if receiver.ackno().is_some() {
                receiver.ackno().unwrap().to_string()
            } else {
                "none".to_string()
            }
        );
        let expected = format!(
            "{}",
            if self.ackno.is_some() {
                self.ackno.as_ref().unwrap().to_string()
            } else {
                "none".to_string()
            }
        );
        assert!(
            b,
            "The TCPReceiver reported ackno `{}`, but it was expected to be `{}`",
            reported, expected
        );
    }
}
impl ReceiverExpectation for ExpectAckno {
    fn description(&self) -> String {
        if self.ackno.is_some() {
            format!("ackno {}", self.ackno.as_ref().unwrap().raw_value())
        } else {
            "no ackno available".to_string()
        }
    }
}
impl ExpectAckno {
    #[allow(dead_code)]
    pub fn new(op: Option<WrappingInt32>) -> ExpectAckno {
        ExpectAckno { ackno: op }
    }
}

pub struct ExpectWindow {
    window: SizeT,
}
impl ReceiverTestStep for ExpectWindow {
    fn execute(&self, receiver: &mut TCPReceiver) {
        println!("  step: {}", ReceiverExpectation::to_string(self));

        let b = receiver.window_size() == self.window;
        assert!(
            b,
            "The TCPReceiver reported window `{}`, but it was expected to be `{}`",
            receiver.window_size(),
            self.window
        );
    }
}
impl ReceiverExpectation for ExpectWindow {
    fn description(&self) -> String {
        format!("window {}", self.window)
    }
}
impl ExpectWindow {
    #[allow(dead_code)]
    pub fn new(w: SizeT) -> ExpectWindow {
        ExpectWindow { window: w }
    }
}

pub struct ExpectUnassembledBytes {
    n_bytes: SizeT,
}
impl ReceiverTestStep for ExpectUnassembledBytes {
    fn execute(&self, receiver: &mut TCPReceiver) {
        println!("  step: {}", ReceiverExpectation::to_string(self));

        let b = receiver.unassembled_bytes() == self.n_bytes;
        assert!(b, "The TCPReceiver reported `{}` unassembled bytes, but there was expected to be `{}` unassembled bytes", receiver.unassembled_bytes(), self.n_bytes);
    }
}
impl ReceiverExpectation for ExpectUnassembledBytes {
    fn description(&self) -> String {
        format!("{} unassembled bytes", self.n_bytes)
    }
}
impl ExpectUnassembledBytes {
    #[allow(dead_code)]
    pub fn new(n: SizeT) -> ExpectUnassembledBytes {
        ExpectUnassembledBytes { n_bytes: n }
    }
}

pub struct ExpectTotalAssembledBytes {
    n_bytes: SizeT,
}
impl ReceiverTestStep for ExpectTotalAssembledBytes {
    fn execute(&self, receiver: &mut TCPReceiver) {
        println!("  step: {}", ReceiverExpectation::to_string(self));

        let b = receiver.stream_out().bytes_written() == self.n_bytes;
        assert!(b, "The TCPReceiver reported `{}` bytes written, but there was expected to be `{}` bytes written (in total)", receiver.stream_out().bytes_written(), self.n_bytes);
    }
}
impl ReceiverExpectation for ExpectTotalAssembledBytes {
    fn description(&self) -> String {
        format!("{} assembled bytes, in total", self.n_bytes)
    }
}
impl ExpectTotalAssembledBytes {
    #[allow(dead_code)]
    pub fn new(n: SizeT) -> ExpectTotalAssembledBytes {
        ExpectTotalAssembledBytes { n_bytes: n }
    }
}

pub struct ExpectEof {}
impl ReceiverTestStep for ExpectEof {
    fn execute(&self, receiver: &mut TCPReceiver) {
        println!("  step: {}", ReceiverExpectation::to_string(self));

        let b = receiver.stream_out().eof();
        let s =
            format!("The TCPReceiver stream reported eof() == false, but was expected to be true");
        assert!(b, "{}", s);
    }
}
impl ReceiverExpectation for ExpectEof {
    fn description(&self) -> String {
        format!("receiver.stream_out().eof() == true")
    }
}
impl ExpectEof {
    #[allow(dead_code)]
    pub fn new() -> ExpectEof {
        ExpectEof {}
    }
}

pub struct ExpectInputNotEnded {}
impl ReceiverTestStep for ExpectInputNotEnded {
    fn execute(&self, receiver: &mut TCPReceiver) {
        println!("  step: {}", ReceiverExpectation::to_string(self));

        let b = !receiver.stream_out().input_ended();
        let s = format!(
            "The TCPReceiver stream reported input_ended() == true, but was expected to be false"
        );
        assert!(b, "{}", s);
    }
}
impl ReceiverExpectation for ExpectInputNotEnded {
    fn description(&self) -> String {
        format!("receiver.stream_out().input_ended() == false")
    }
}
impl ExpectInputNotEnded {
    #[allow(dead_code)]
    pub fn new() -> ExpectInputNotEnded {
        ExpectInputNotEnded {}
    }
}

pub struct ExpectBytes {
    bytes: String,
}
impl ReceiverTestStep for ExpectBytes {
    fn execute(&self, receiver: &mut TCPReceiver) {
        println!("  step: {}", ReceiverExpectation::to_string(self));

        let stream = receiver.stream_out_mut();
        let s = format!("The TCPReceiver reported `{}` bytes available, but there were expected to be `{}` bytes available", stream.buffer_size(), self.bytes.len());
        assert_eq!(stream.buffer_size(), self.bytes.len(), "{}", s);
        let bytes = stream.read(self.bytes.len());

        let s1 = format!(
            "the TCPReceiver assembled \"{}\", but was expected to assemble \"{}\".",
            bytes, self.bytes
        );
        assert_eq!(bytes, self.bytes, "{}", s1);
    }
}
impl ReceiverExpectation for ExpectBytes {
    fn description(&self) -> String {
        format!("bytes available: \"{}\"", self.bytes)
    }
}
impl ExpectBytes {
    #[allow(dead_code)]
    pub fn new(b: String) -> ExpectBytes {
        ExpectBytes { bytes: b }
    }
}

pub trait ReceiverAction: ReceiverTestStep {
    fn description(&self) -> String;
    fn to_string(&self) -> String {
        String::from(format!("Action:      {}", self.description()))
    }
}

#[derive(PartialEq)]
pub enum Result {
    NOT_SYN,
    OK,
}
pub struct SegmentArrives {
    ack: bool,
    rst: bool,
    syn: bool,
    fin: bool,
    seqno: WrappingInt32,
    ackno: WrappingInt32,
    win: u16,
    data: String,
    result: Option<Result>,
}
impl ReceiverTestStep for SegmentArrives {
    fn execute(&self, receiver: &mut TCPReceiver) {
        println!("  step: {}", ReceiverAction::to_string(self));

        let seg = self.build_segment();
        let mut o = String::new();
        o.push_str(seg.header().summary().as_str());
        if self.data.len() > 0 {
            let s = format!(" with data \"{}\"", self.data);
            o.push_str(s.as_str());
        }

        receiver.segment_received(&seg);

        let res: Result;
        if receiver.ackno().is_none() {
            res = Result::NOT_SYN;
        } else {
            res = Result::OK;
        }

        if self.result.is_some() {
            let b = self.result.as_ref().unwrap() == &res;
            let s = format!("TCPReceiver::segment_received() reported `{}` in response to `{}`, but it was expected to report `{}`", SegmentArrives::result_name(&res), o, SegmentArrives::result_name(self.result.as_ref().unwrap()));
            assert!(b, "{}", s);
        }
    }
}
impl ReceiverAction for SegmentArrives {
    fn description(&self) -> String {
        let seg = self.build_segment();
        format!(
            "segment arrives {} with data \"{}\"",
            seg.header().summary(),
            self.data
        )
    }
}
impl SegmentArrives {
    #[allow(dead_code)]
    pub fn new(b: String) -> SegmentArrives {
        SegmentArrives {
            ack: false,
            rst: false,
            syn: false,
            fin: false,
            seqno: WrappingInt32::new(0),
            ackno: WrappingInt32::new(0),
            win: 0,
            data: b,
            result: None,
        }
    }

    #[allow(dead_code)]
    pub fn build_segment(&self) -> TCPSegment {
        let mut header = TCPHeader::new();
        header.ack = self.ack;
        header.fin = self.fin;
        header.syn = self.syn;
        header.rst = self.rst;
        header.ackno = WrappingInt32::new(self.ackno.raw_value());
        header.seqno = WrappingInt32::new(self.seqno.raw_value());
        header.win = self.win;

        let buf = Buffer::new(String::from(self.data.as_str()).into_bytes());

        TCPSegment::new(header, buf)
    }

    #[allow(dead_code)]
    pub fn result_name(res: &Result) -> String {
        match res {
            Result::NOT_SYN => "(no SYN received, so no ackno available)".to_string(),
            Result::OK => "(SYN received, so ackno available)".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn with_ack(&mut self, _ackno: WrappingInt32) -> &mut SegmentArrives {
        self.ack = true;
        self.ackno = _ackno;
        self
    }

    #[allow(dead_code)]
    pub fn with_ack_u32(&mut self, _ackno: u32) -> &mut SegmentArrives {
        self.with_ack(WrappingInt32::new(_ackno))
    }

    #[allow(dead_code)]
    pub fn with_rst(&mut self) -> &mut SegmentArrives {
        self.rst = true;
        self
    }

    #[allow(dead_code)]
    pub fn with_syn(&mut self) -> &mut SegmentArrives {
        self.syn = true;
        self
    }

    #[allow(dead_code)]
    pub fn with_fin(&mut self) -> &mut SegmentArrives {
        self.fin = true;
        self
    }

    #[allow(dead_code)]
    pub fn with_seqno(&mut self, _seqno: WrappingInt32) -> &mut SegmentArrives {
        self.seqno = _seqno;
        self
    }

    #[allow(dead_code)]
    pub fn with_seqno_u32(&mut self, _seqno: u32) -> &mut SegmentArrives {
        self.with_seqno(WrappingInt32::new(_seqno))
    }

    #[allow(dead_code)]
    pub fn with_win(&mut self, _win: u16) -> &mut SegmentArrives {
        self.win = _win;
        self
    }

    #[allow(dead_code)]
    pub fn with_data(&mut self, data: String) -> &mut SegmentArrives {
        self.data = data;
        self
    }

    #[allow(dead_code)]
    pub fn with_result(&mut self, _result: Result) -> &mut SegmentArrives {
        self.result = Option::from(_result);
        self
    }
}

pub struct TCPReceiverTestHarness {
    receiver: TCPReceiver,
}
impl TCPReceiverTestHarness {
    #[allow(dead_code)]
    pub fn new(capacity: SizeT) -> TCPReceiverTestHarness {
        let harness = TCPReceiverTestHarness {
            receiver: TCPReceiver::new(capacity),
        };
        println!("test:=> Initialized (capacity = {})", capacity);
        harness
    }

    #[allow(dead_code)]
    pub fn execute(&mut self, step: &dyn ReceiverTestStep) {
        step.execute(&mut self.receiver)
    }
}
