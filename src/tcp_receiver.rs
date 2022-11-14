use crate::byte_stream::ByteStream;
use crate::stream_reassembler::StreamReassembler;
use crate::tcp_helpers::tcp_segment::TCPSegment;
use crate::wrapping_integers::WrappingInt32;
use crate::SizeT;

#[derive(Debug)]
#[allow(dead_code)]
pub struct TCPReceiver {
    capacity: SizeT,
    reassembler: StreamReassembler,
    syn: (u32, u64, bool),
    fin: (u32, u64, bool),
}
impl TCPReceiver {
    #[allow(dead_code)]
    pub fn new(_capacity: SizeT) -> TCPReceiver {
        TCPReceiver {
            capacity: _capacity,
            reassembler: StreamReassembler::new(_capacity),
            syn: (0, 0, false),
            fin: (0, 0, false),
        }
    }

    #[allow(dead_code)]
    pub fn ackno(&self) -> Option<WrappingInt32> {
        if !self.syn.2 {
            return None;
        }

        let next_stream_index = self.stream_out().bytes_written();
        let next_abs_index = (next_stream_index + 1) as u64;
        if self.fin.2 && self.fin.1 == next_abs_index {
            Some(WrappingInt32::wrap(
                next_abs_index + 1,
                &WrappingInt32::new(self.syn.0),
            ))
        } else {
            Some(WrappingInt32::wrap(
                next_abs_index,
                &WrappingInt32::new(self.syn.0),
            ))
        }
    }

    #[allow(dead_code)]
    pub fn window_size(&self) -> SizeT {
        self.stream_out().remaining_capacity()
    }

    #[allow(dead_code)]
    pub fn unassembled_bytes(&self) -> SizeT {
        self.reassembler.unassembled_bytes()
    }

    #[allow(dead_code)]
    pub fn segment_received(&mut self, seg: &TCPSegment) {
        let seq_no: u32 = seg.header().seqno.raw_value();
        if seg.header().syn {
            self.syn = (seq_no, 0, true);
        }
        if !self.syn.2 {
            return;
        }

        let checkpoint: u64 = self.stream_out().bytes_written() as u64;
        let abs_seq_no: u64 = WrappingInt32::unwrap(
            &seg.header().seqno,
            &WrappingInt32::new(self.syn.0),
            checkpoint,
        );
        let next_valid_seq_no: u64 = if self.ackno().is_some() {
            WrappingInt32::unwrap(
                &self.ackno().unwrap(),
                &WrappingInt32::new(self.syn.0),
                checkpoint,
            )
        } else {
            0
        };
        let tw: SizeT = if self.window_size() == 0 {
            0
        } else {
            self.window_size() - 1
        };
        // discard segments out of current wnd range
        if abs_seq_no > (next_valid_seq_no + (tw as u64)) {
            return;
        }

        let mut _fin = false;
        if seg.header().fin {
            _fin = true;
            let t: u64 = (((seq_no as SizeT) + seg.length_in_sequence_space() - 1) as u64
                % ((1 as u64) << 32)) as u64;
            let fin_seq = t as u32;
            let fin_abs_seq: u64 = WrappingInt32::unwrap(
                &WrappingInt32::new(fin_seq),
                &WrappingInt32::new(self.syn.0),
                checkpoint,
            );
            self.fin = (fin_seq, fin_abs_seq, true);
        }

        let stream_index: u64;
        if seg.header().syn {
            stream_index = 0;
        } else if _fin && seg.payload().size() == 0 {
            if 0 == checkpoint {
                stream_index = 0;
            } else {
                if abs_seq_no < 2 {
                    return;
                }
                stream_index = abs_seq_no - 2;
            }
        } else {
            // abs index here shouldn't be zero
            if 0 == abs_seq_no {
                return;
            }
            stream_index = abs_seq_no - 1;
        }

        if seg.payload().size() > 0 || _fin {
            // todo: copied?
            self.reassembler.push_substring(
                &String::from_utf8(Vec::from(seg.payload().str())).unwrap(),
                stream_index,
                _fin,
            );
        }
    }

    #[allow(dead_code)]
    pub fn stream_out(&self) -> &ByteStream {
        self.reassembler.stream_out()
    }

    #[allow(dead_code)]
    pub fn stream_out_mut(&mut self) -> &mut ByteStream {
        self.reassembler.stream_out_mut()
    }
}
