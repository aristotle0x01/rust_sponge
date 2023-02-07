use crate::SizeT;
use crate::tcp_helpers::ipv4_header::IPv4Header;
use crate::util::buffer::Buffer;
use crate::util::parser::{NetParser, ParseResult};
use crate::util::util::InternetChecksum;

#[derive(Debug)]
pub struct IPv4Datagram {
    header: IPv4Header,
    payload: Buffer,
}
impl IPv4Datagram {
    #[allow(dead_code)]
    pub fn new(head: IPv4Header, load: Buffer) -> IPv4Datagram {
        IPv4Datagram {
            header: head,
            payload: load,
        }
    }

    pub fn parse(&mut self, _buffer: &Buffer, _datagram_layer_checksum: u32) -> ParseResult {
        let mut p = NetParser::new(_buffer.clone());
        self.header.parse(&mut p);
        self.payload = p.buffer().clone();

        if self.payload.size() != self.header.payload_length() as usize {
            return ParseResult::PacketTooShort;
        }

        return p.get_error();
    }

    #[allow(dead_code)]
    pub fn serialize(&mut self) -> Vec<u8> {
        assert_eq!(
            self.payload.size(),
            self.header.payload_length() as SizeT,
            "IPv4Datagram::serialize: payload is wrong size"
        );

        let header_out = &mut self.header;

        let mut check = InternetChecksum::new(0);
        check.add(header_out.serialize().as_slice());
        header_out.cksum = check.value();

        [&header_out.serialize()[..], self.payload.str()].concat()
    }

    #[allow(dead_code)]
    pub fn header(&self) -> &IPv4Header {
        &self.header
    }

    #[allow(dead_code)]
    pub fn header_mut(&mut self) -> &mut IPv4Header {
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
impl Clone for IPv4Datagram {
    fn clone(&self) -> IPv4Datagram {
        IPv4Datagram {
            header: self.header.clone(),
            payload: self.payload.clone(),
        }
    }
}
