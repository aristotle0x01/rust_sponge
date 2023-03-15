use crate::tcp_helpers::ipv4_header::IPv4Header;
use crate::util::buffer::Buffer;
use crate::util::parser::{NetParser, ParseResult};
use crate::util::util::InternetChecksum;
use crate::SizeT;

#[derive(Debug)]
pub struct IPv4Datagram {
    header: IPv4Header,
    pub payload: Buffer,
}
impl IPv4Datagram {
    #[allow(dead_code)]
    pub fn new(head: IPv4Header, load: Buffer) -> IPv4Datagram {
        IPv4Datagram {
            header: head,
            payload: load,
        }
    }

    pub fn parse(&mut self, _datagram_layer_checksum: u32) -> ParseResult {
        let mut p = NetParser::new(&mut self.payload);
        self.header.parse(&mut p);

        let err = p.get_error();
        drop(p);
        if err != ParseResult::NoError {
            return err;
        }

        if self.payload.size() != self.header.payload_length() as usize {
            return ParseResult::PacketTooShort;
        }

        eprintln!(
            "     ****IPv4Datagram {}, {}",
            self.header.summary(),
            self.payload.size()
        );

        err
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

    // partial move out a field
    #[allow(dead_code)]
    fn swap(&mut self) -> Buffer {
        std::mem::replace(&mut self.payload, Buffer::new(vec![]))
    }
}
