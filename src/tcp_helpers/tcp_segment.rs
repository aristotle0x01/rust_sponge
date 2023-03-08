use crate::tcp_helpers::tcp_header::TCPHeader;
use crate::util::buffer::Buffer;
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

    #[allow(dead_code)]
    pub fn parse_new(bytes: Buffer, checksum: u32) -> Result<TCPSegment, ParseResult> {
        let mut t = TCPSegment {
            header: TCPHeader::new(),
            payload: bytes,
        };
        let r = t.parse(checksum);
        match r {
            ParseResult::NoError => Ok(t),
            _ => Err(r),
        }
    }

    pub fn parse(&mut self, _datagram_layer_checksum: u32) -> ParseResult {
        assert!(self.payload.len() > 0);

        let mut check = InternetChecksum::new(_datagram_layer_checksum);
        check.add(self.payload.str());
        if check.value() != 0 {
            return ParseResult::BadChecksum;
        }

        let mut p = NetParser::new(&mut self.payload);
        self.header.parse(&mut p);

        return p.get_error();
    }

    #[allow(dead_code)]
    pub fn serialize(&mut self, _datagram_layer_checksum: u32) -> Vec<u8> {
        let header_out = &mut self.header;

        // calculate checksum -- taken over entire segment
        let mut check = InternetChecksum::new(_datagram_layer_checksum);
        check.add(header_out.serialize().as_ref());
        check.add(self.payload.str());
        header_out.cksum = check.value();

        [&header_out.serialize()[..], self.payload.str()].concat()
    }

    #[allow(dead_code)]
    pub fn length_in_sequence_space(&self) -> SizeT {
        self.payload().len()
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
impl Clone for TCPSegment {
    fn clone(&self) -> Self {
        TCPSegment {
            header: self.header.clone(),
            payload: self.payload.clone(),
        }
    }
}
