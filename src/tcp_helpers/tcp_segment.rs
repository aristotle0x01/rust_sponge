use crate::tcp_helpers::tcp_header::TCPHeader;
use crate::util::buffer::{Buffer, BufferList};
use crate::util::parser::{NetParser, ParseResult};
use crate::util::util::InternetChecksum;
use crate::SizeT;

#[derive(Debug)]
pub struct TCPSegment {
    header: TCPHeader,
    payload: Buffer,
}
impl TCPSegment {
    #[allow(dead_code)]
    pub fn new(head: TCPHeader, load: Buffer) -> TCPSegment {
        TCPSegment {
            header: head,
            payload: load,
        }
    }

    pub fn parse(&mut self, _buffer: &Buffer, _datagram_layer_checksum: u32) -> ParseResult {
        let mut check = InternetChecksum::new(_datagram_layer_checksum);
        check.add(_buffer.str());
        if check.value() != 0 {
            return ParseResult::BadChecksum;
        }

        let mut p = NetParser::new(Buffer::new(_buffer.str().to_vec()));
        self.header.parse(&mut p);
        // todo: copied, not the original shared ref way
        // self.payload = p.buffer(); // c++
        self.payload = p.buffer().clone();

        return p.get_error();
    }

    #[allow(dead_code)]
    pub fn serialize(&mut self, _datagram_layer_checksum: u32) -> BufferList {
        let header_out = &mut self.header;
        header_out.cksum = 0;

        // calculate checksum -- taken over entire segment
        let mut check = InternetChecksum::new(_datagram_layer_checksum);
        let serialized = header_out.serialize();
        check.add(serialized.as_bytes());
        check.add(self.payload.str());
        header_out.cksum = check.value();

        let mut ret = BufferList::new(Buffer::new(serialized.into_bytes()));
        ret.append(&self.payload.clone().into());

        ret
    }

    #[allow(dead_code)]
    pub fn length_in_sequence_space(&self) -> SizeT {
        self.payload().str().len()
            + (if self.header().syn { 1 } else { 0 })
            + (if self.header().fin { 1 } else { 0 })
    }

    #[allow(dead_code)]
    pub fn header(&self) -> &TCPHeader {
        &self.header
    }

    #[allow(dead_code)]
    pub fn header_mut(&mut self) -> &mut TCPHeader {
        &mut self.header
    }

    #[allow(dead_code)]
    pub fn payload(&self) -> &Buffer {
        &self.payload
    }

    #[allow(dead_code)]
    pub fn payload_mut(&mut self) -> &mut Buffer {
        &mut self.payload
    }
}
