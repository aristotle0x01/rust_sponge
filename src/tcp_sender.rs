use crate::byte_stream::ByteStream;
use crate::tcp_helpers::tcp_config::TCPConfig;
use crate::tcp_helpers::tcp_header::TCPHeader;
use crate::tcp_helpers::tcp_segment::TCPSegment;
use crate::util::buffer::Buffer;
use crate::util::tcp_timer::TcpTimer;
use crate::wrapping_integers::WrappingInt32;
use crate::SizeT;
use std::collections::{BTreeMap, LinkedList, VecDeque};

#[derive(Debug)]
pub struct TCPSender {
    isn: WrappingInt32,
    segments_out: VecDeque<TCPSegment>,
    outstanding: BTreeMap<u64, TCPSegment>,
    stream: ByteStream,
    timer: TcpTimer,
    initial_retransmission_timeout: u32,
    retransmission_timeout: u32,
    ms_total_tick: SizeT,
    consecutive_retransmissions: SizeT,
    next_abs_seq_no: u64,
    check_point: u64,
    last_ack_no: WrappingInt32,
    wnd_left_abs_no: u64,
    wnd_right_abs_no: u64,
    window_size: u16,
}
impl TCPSender {
    #[allow(dead_code)]
    pub fn new(_capacity: SizeT, retx_timeout: u16, fixed_isn: Option<WrappingInt32>) -> TCPSender {
        TCPSender {
            isn: fixed_isn.unwrap(),
            segments_out: Default::default(),
            outstanding: Default::default(),
            stream: ByteStream::new(_capacity),
            timer: TcpTimer::new(retx_timeout as u32),
            initial_retransmission_timeout: retx_timeout as u32,
            retransmission_timeout: retx_timeout as u32,
            ms_total_tick: 0,
            consecutive_retransmissions: 0,
            next_abs_seq_no: 0,
            check_point: 0,
            last_ack_no: WrappingInt32::new(0),
            wnd_left_abs_no: 0,
            wnd_right_abs_no: 0,
            window_size: 1,
        }
    }

    #[allow(dead_code)]
    pub fn stream_in(&self) -> &ByteStream {
        &self.stream
    }

    #[allow(dead_code)]
    pub fn stream_in_mut(&mut self) -> &mut ByteStream {
        &mut self.stream
    }

    #[allow(dead_code)]
    pub fn ack_received(&mut self, ackno: WrappingInt32, window_size: u16) {
        let abs_ack_no = WrappingInt32::unwrap(&ackno, &self.isn, self.check_point);

        let last_abs_ack_no = WrappingInt32::unwrap(&self.last_ack_no, &self.isn, self.check_point);
        // let last_abs_ack_no = self.wnd_left_abs_no;
        if abs_ack_no > last_abs_ack_no {
            self.retransmission_timeout = self.initial_retransmission_timeout;
            self.consecutive_retransmissions = 0;
            if !self.outstanding.is_empty() {
                self.timer
                    .restart(self.ms_total_tick, self.retransmission_timeout);
            }
        }

        if abs_ack_no > self.next_abs_seq_no || abs_ack_no < self.wnd_left_abs_no {
            // Impossible ackno (beyond next seqno) is ignored or repeated ack
            return;
        }

        let mut list: LinkedList<u64> = LinkedList::new();
        for (first, second) in self.outstanding.iter() {
            if (first + (second.length_in_sequence_space() as u64) - 1) < abs_ack_no {
                list.push_back(*first);
            }
        }
        for n in list {
            self.outstanding.remove(&n);
        }
        if self.outstanding.is_empty() {
            self.timer.stop();
        }

        // What should I do if the window size is zero? If the receiver has announced a
        // window size of zero, the fill window method should act like the window size is one.
        // When filling window, treat a '0' window size as equal to '1' but don't back off RTO
        // so when _window_size == 0, then (_wnd_right_abs_no-_wnd_left_abs_no+1)==1
        self.window_size = window_size;
        self.last_ack_no = ackno;
        self.wnd_left_abs_no =
            WrappingInt32::unwrap(&self.last_ack_no, &self.isn, self.check_point);
        // self.wnd_left_abs_no = abs_ack_no;
        self.wnd_right_abs_no = self.wnd_left_abs_no
            + (if self.window_size == 0 {
                1
            } else {
                self.window_size
            }) as u64
            - 1;
        if self.wnd_left_abs_no > self.check_point && self.window_size > 0 {
            self.check_point = self.wnd_left_abs_no - 1;
        }
    }

    #[allow(dead_code)]
    pub fn send_empty_segment(&mut self, rst: bool) {
        self.segments_out.push_back(TCPSender::build_segment(
            vec![],
            false,
            false,
            rst,
            WrappingInt32::wrap(self.next_abs_seq_no, &self.isn.clone()),
        ));
    }

    #[allow(dead_code)]
    pub fn fill_window(&mut self) {
        // previous way of matching (let state = TCPState::state_summary_sender(&self)) when error would prevent further sending
        if self.next_abs_seq_no == 0 {
            let seg = TCPSender::build_segment(vec![], true, false, false, self.isn.clone());
            let n_ = self.next_abs_seq_no + seg.length_in_sequence_space() as u64;
            self.segments_out.push_back(seg.clone());
            self.outstanding.insert(self.next_abs_seq_no, seg);
            self.next_abs_seq_no = n_;
            self.timer
                .start(self.ms_total_tick, self.retransmission_timeout);
        } else if self.next_abs_seq_no == self.bytes_in_flight() as u64 {
        } else if !self.stream_in().eof()
            || (self.next_abs_seq_no < (self.stream_in().bytes_written() + 2) as u64)
        {
            let mut fin = false;
            while !self.stream.buffer_empty() && self.next_abs_seq_no <= self.wnd_right_abs_no {
                let gap: SizeT = (self.wnd_right_abs_no - self.next_abs_seq_no + 1) as SizeT;
                let vec = vec![TCPConfig::MAX_PAYLOAD_SIZE, gap, self.stream.buffer_size()];
                let readable: SizeT = *vec.iter().min().unwrap();
                let data = self.stream.read(readable);
                if self.stream.eof()
                    && (self.next_abs_seq_no + readable as u64) <= self.wnd_right_abs_no
                {
                    fin = true;
                }
                let seg = TCPSender::build_segment(
                    data,
                    false,
                    fin,
                    false,
                    WrappingInt32::wrap(self.next_abs_seq_no, &self.isn),
                );
                let n_ = self.next_abs_seq_no + seg.length_in_sequence_space() as u64;
                self.segments_out.push_back(seg.clone());
                self.outstanding.insert(self.next_abs_seq_no, seg);
                self.next_abs_seq_no = n_;
                self.timer
                    .start(self.ms_total_tick, self.retransmission_timeout);
            }
            if fin == false && self.stream.eof() && self.next_abs_seq_no <= self.wnd_right_abs_no {
                let seg = TCPSender::build_segment(
                    vec![],
                    false,
                    true,
                    false,
                    WrappingInt32::wrap(self.next_abs_seq_no, &self.isn),
                );
                let n_ = self.next_abs_seq_no + seg.length_in_sequence_space() as u64;
                self.segments_out.push_back(seg.clone());
                self.outstanding.insert(self.next_abs_seq_no, seg);
                self.next_abs_seq_no = n_;
                self.timer
                    .start(self.ms_total_tick, self.retransmission_timeout);
            }
        }
    }

    #[allow(dead_code)]
    pub fn tick(&mut self, ms_since_last_tick: SizeT) {
        self.ms_total_tick = self.ms_total_tick + ms_since_last_tick;
        let expired = self.timer.expire(self.ms_total_tick);

        if self.outstanding.is_empty() {
            self.timer.stop();
            return;
        }

        if expired {
            let _entry = self.outstanding.iter().next().unwrap();
            // todo: clone here
            self.segments_out.push_back(_entry.1.clone());
            if self.window_size > 0 {
                self.retransmission_timeout = self.retransmission_timeout * 2;
                self.consecutive_retransmissions += 1;
            }
            self.timer
                .restart(self.ms_total_tick, self.retransmission_timeout);
        }
    }

    #[allow(dead_code)]
    pub fn bytes_in_flight(&self) -> SizeT {
        let mut bytes = 0;
        for (_first, _second) in self.outstanding.iter() {
            bytes = bytes + _second.length_in_sequence_space();
        }
        return bytes;
    }

    #[allow(dead_code)]
    pub fn consecutive_retransmissions(&self) -> u32 {
        self.consecutive_retransmissions as u32
    }

    #[allow(dead_code)]
    pub fn segments_out(&self) -> &VecDeque<TCPSegment> {
        &self.segments_out
    }

    #[allow(dead_code)]
    pub fn segments_out_mut(&mut self) -> &mut VecDeque<TCPSegment> {
        &mut self.segments_out
    }

    #[allow(dead_code)]
    pub fn next_seqno_absolute(&self) -> u64 {
        self.next_abs_seq_no
    }

    #[allow(dead_code)]
    pub fn next_seqno(&self) -> WrappingInt32 {
        WrappingInt32::wrap(self.next_abs_seq_no, &self.isn)
    }

    fn build_segment(
        data: Vec<u8>,
        syn: bool,
        fin: bool,
        rst: bool,
        _seq_no: WrappingInt32,
    ) -> TCPSegment {
        let mut header = TCPHeader::new();
        header.fin = fin;
        header.syn = syn;
        header.rst = rst;
        header.seqno = _seq_no;

        TCPSegment::new(header, Buffer::new(data))
    }
}
