use crate::byte_stream::ByteStream;
use crate::tcp_helpers::tcp_config::TCPConfig;
use crate::tcp_helpers::tcp_segment::TCPSegment;
use crate::tcp_helpers::tcp_state::{TCPSenderStateSummary, TCPState};
use crate::tcp_receiver::TCPReceiver;
use crate::tcp_sender::TCPSender;
use crate::SizeT;
use std::collections::VecDeque;
use std::rc::Rc;

#[derive(Debug)]
pub struct TCPConnection {
    cfg: TCPConfig,
    receiver: TCPReceiver,
    sender: TCPSender,
    segments_out: VecDeque<Rc<TCPSegment>>,
    linger_after_streams_finish: bool,
    total_tick: SizeT,
    last_recv_seg_tick: SizeT,
    active: bool,
    fin_received: bool,
    fin_sent: bool,
    syn_sent_or_recv: bool,
}
impl TCPConnection {
    #[allow(dead_code)]
    pub fn new(cnf: TCPConfig) -> TCPConnection {
        TCPConnection {
            cfg: cnf.clone(),
            receiver: TCPReceiver::new(cnf.recv_capacity),
            sender: TCPSender::new(cnf.send_capacity, cnf.rt_timeout, cnf.fixed_isn),
            segments_out: Default::default(),
            linger_after_streams_finish: true,
            total_tick: 0,
            last_recv_seg_tick: 0,
            active: true,
            fin_received: false,
            fin_sent: false,
            syn_sent_or_recv: false,
        }
    }

    #[allow(dead_code)]
    pub fn connect(&mut self) {
        self.sender.fill_window();

        while !self.sender.segments_out_mut().is_empty() {
            let seg = self.sender.segments_out_mut().pop_front().unwrap();
            self.segments_out.push_back(seg);
            self.syn_sent_or_recv = true;
        }
    }

    #[allow(dead_code)]
    pub fn write(&mut self, data: &String) -> SizeT {
        let written = self.sender.stream_in_mut().write(data);
        self.sender.fill_window();

        while !self.sender.segments_out_mut().is_empty() {
            let mut seg = self.sender.segments_out_mut().front_mut().unwrap();
            if self.receiver.ackno().is_some() {
                let mut_seg = Rc::make_mut(&mut seg);

                mut_seg.header_mut().ack = true;
                mut_seg.header_mut().ackno = self.receiver.ackno().unwrap();
                if self.receiver.window_size() >= u16::MAX as SizeT {
                    mut_seg.header_mut().win = u16::MAX;
                } else {
                    mut_seg.header_mut().win = self.receiver.window_size() as u16;
                }
            }
            self.segments_out.push_back(seg.clone());
            if seg.header().fin {
                self.fin_sent = true;
            }
            self.sender.segments_out_mut().pop_front();
        }

        self.check_active();

        written
    }

    #[allow(dead_code)]
    pub fn remaining_outbound_capacity(&self) -> SizeT {
        self.sender.stream_in().remaining_capacity()
    }

    #[allow(dead_code)]
    pub fn end_input_stream(&mut self) {
        self.sender.stream_in_mut().end_input();
        self.sender.fill_window();
        self.write(&String::new());
    }

    #[allow(dead_code)]
    pub fn inbound_stream_mut(&mut self) -> &mut ByteStream {
        self.receiver.stream_out_mut()
    }

    #[allow(dead_code)]
    pub fn bytes_in_flight(&self) -> SizeT {
        self.sender.bytes_in_flight()
    }

    #[allow(dead_code)]
    pub fn unassembled_bytes(&self) -> SizeT {
        self.receiver.unassembled_bytes()
    }

    #[allow(dead_code)]
    pub fn time_since_last_segment_received(&self) -> SizeT {
        self.total_tick - self.last_recv_seg_tick
    }

    #[allow(dead_code)]
    pub fn state(&self) -> TCPState {
        TCPState::new(
            &self.sender,
            &self.receiver,
            self.active,
            self.linger_after_streams_finish,
        )
    }

    #[allow(dead_code)]
    pub fn segment_received(&mut self, seg: &TCPSegment) {
        self.last_recv_seg_tick = self.total_tick;

        self.receiver.segment_received(seg);

        if seg.header().syn && 0 == self.sender.next_seqno_absolute() {
            self.write(&String::new());
            self.syn_sent_or_recv = true;

            return;
        }

        if !self.syn_sent_or_recv {
            return;
        }

        if seg.header().rst {
            self.active = false;
            self.sender.stream_in_mut().set_error();
            self.receiver.stream_out_mut().set_error();
        }

        if seg.header().fin {
            self.fin_received = true;
            if !self.fin_sent {
                self.linger_after_streams_finish = false;
            }
        }

        if seg.header().ack {
            self.sender
                .ack_received(seg.header().ackno, seg.header().win);
            self.write(&String::new());
        }

        if seg.length_in_sequence_space() > 0 {
            self.sender.send_empty_segment();
            self.write(&String::new());
        }

        if self.receiver.ackno().is_some()
            && seg.length_in_sequence_space() == 0
            && seg.header().seqno == (self.receiver.ackno().unwrap() - 1)
        {
            self.sender.send_empty_segment();
            self.write(&String::new());
        }

        self.check_active();
    }

    #[allow(dead_code)]
    pub fn tick(&mut self, ms_since_last_tick: SizeT) {
        if self.sender.consecutive_retransmissions() >= TCPConfig::MAX_RETX_ATTEMPTS {
            self.send_reset();
            return;
        }

        let l_old = self.sender.segments_out_mut().len() as SizeT;
        self.total_tick += ms_since_last_tick;
        self.sender.tick(ms_since_last_tick);
        let l_new = self.sender.segments_out_mut().len() as SizeT;
        if l_new > l_old {
            self.write(&String::new());
        }

        self.check_active();
    }

    #[allow(dead_code)]
    pub fn segments_out_mut(&mut self) -> &mut VecDeque<Rc<TCPSegment>> {
        &mut self.segments_out
    }

    #[allow(dead_code)]
    pub fn active(&self) -> bool {
        self.active
    }

    #[allow(dead_code)]
    fn send_reset(&mut self) {
        self.sender.send_empty_segment();
        let mut seg = self.sender.segments_out_mut().back_mut().unwrap();
        Rc::make_mut(&mut seg).header_mut().rst = true;
        self.write(&String::new());

        self.sender.stream_in_mut().set_error();
        self.receiver.stream_out_mut().set_error();
        self.active = false;
    }

    #[allow(dead_code)]
    fn check_active(&mut self) {
        if !self.active {
            return;
        }

        let r =
            (0 == self.receiver.unassembled_bytes()) && self.receiver.stream_out().input_ended();
        let s = self.fin_sent
            && (TCPSenderStateSummary::FIN_ACKED == TCPState::state_summary_sender(&self.sender));
        if !(r && s) {
            return;
        }

        if self.linger_after_streams_finish {
            if self.time_since_last_segment_received() >= (10 * self.cfg.rt_timeout) as SizeT {
                self.active = false;
            }
        } else {
            if self.fin_received {
                self.active = false;
            }
        }
    }
}
impl Drop for TCPConnection {
    fn drop(&mut self) {
        if self.active() {
            eprintln!("Warning: Unclean shutdown of TCPConnection\n");
            self.send_reset();
        }
    }
}
