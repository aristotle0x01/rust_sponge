use crate::tcp_helpers::ethernet_header::EthernetHeader;
use crate::util::buffer::Buffer;
use crate::util::parser::{NetParser, ParseResult};

#[derive(Debug, Clone)]
pub struct EthernetFrame {
    pub(crate) header: EthernetHeader,
    pub(crate) payload: Buffer,
}
impl EthernetFrame {
    #[allow(dead_code)]
    pub fn new() -> EthernetFrame {
        EthernetFrame {
            header: EthernetHeader::new(),
            payload: Buffer::new(vec![]),
        }
    }

    pub fn parse(&mut self, bytes: Vec<u8>) -> ParseResult {
        self.payload = Buffer::new(bytes);

        let mut p = NetParser::new(&mut self.payload);
        self.header.parse(&mut p);

        p.get_error()
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend_from_slice(self.header.serialize().as_slice());
        bytes.extend_from_slice(self.payload.str());

        bytes
    }

    pub fn header(&self) -> &EthernetHeader {
        &self.header
    }

    pub fn header_mut(&mut self) -> &mut EthernetHeader {
        &mut self.header
    }

    pub fn payload(&self) -> &Buffer {
        &self.payload
    }

    pub fn payload_mut(&mut self) -> &mut Buffer {
        &mut self.payload
    }
}
