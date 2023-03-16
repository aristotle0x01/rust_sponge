use crate::tcp_helpers::ethernet_header::{to_string, EthernetAddress, EthernetHeader};
use crate::util::buffer::Buffer;
use crate::util::parser::{NetParser, NetUnparser, ParseResult};
use crate::SizeT;
use std::net::Ipv4Addr;

#[derive(Debug, Copy, Clone)]
pub struct ARPMessage {
    pub(crate) hardware_type: u16,
    pub(crate) protocol_type: u16,
    pub(crate) hardware_address_size: u8,
    pub(crate) protocol_address_size: u8,
    pub(crate) opcode: u16,
    pub(crate) sender_ethernet_address: EthernetAddress,
    pub(crate) sender_ip_address: u32,
    pub(crate) target_ethernet_address: EthernetAddress,
    pub(crate) target_ip_address: u32,
}
impl ARPMessage {
    pub const LENGTH: SizeT = 28 as SizeT;
    pub const TYPE_ETHERNET: u16 = 1;
    pub const OPCODE_REQUEST: u16 = 1;
    pub const OPCODE_REPLY: u16 = 2;

    #[allow(dead_code)]
    pub fn new() -> ARPMessage {
        ARPMessage {
            hardware_type: ARPMessage::TYPE_ETHERNET,
            protocol_type: EthernetHeader::TYPE_IPV4,
            hardware_address_size: 6,
            protocol_address_size: 4,
            opcode: 0,
            sender_ethernet_address: [0u8; 6],
            sender_ip_address: 0,
            target_ethernet_address: [0u8; 6],
            target_ip_address: 0,
        }
    }

    pub fn parse(&mut self, bytes: Vec<u8>) -> ParseResult {
        let mut buf = Buffer::new(bytes);
        let mut p = NetParser::new(&mut buf);

        if p.buffer().size() < ARPMessage::LENGTH {
            return ParseResult::PacketTooShort;
        }

        self.hardware_type = p.u16();
        self.protocol_type = p.u16();
        self.hardware_address_size = p.u8();
        self.protocol_address_size = p.u8();
        self.opcode = p.u16();

        if !self.supported() {
            return ParseResult::Unsupported;
        }

        for b in self.sender_ethernet_address.iter_mut() {
            *b = p.u8();
        }
        self.sender_ip_address = p.u32();

        for b in self.target_ethernet_address.iter_mut() {
            *b = p.u8();
        }
        self.target_ip_address = p.u32();

        p.get_error()
    }

    pub fn supported(&self) -> bool {
        self.hardware_type == ARPMessage::TYPE_ETHERNET
            && self.protocol_type == EthernetHeader::TYPE_IPV4
            && self.hardware_address_size == 6
            && self.protocol_address_size == 4
            && (self.opcode == ARPMessage::OPCODE_REQUEST
                || self.opcode == ARPMessage::OPCODE_REPLY)
    }

    pub fn serialize(&self) -> Vec<u8> {
        assert!(self.supported(), "ARPMessage::serialize(): unsupported field combination (must be Ethernet/IP, and request or reply)");

        let mut ret: Vec<u8> = Vec::with_capacity(ARPMessage::LENGTH);

        NetUnparser::u16(&mut ret, self.hardware_type);
        NetUnparser::u16(&mut ret, self.protocol_type);
        NetUnparser::u8(&mut ret, self.hardware_address_size);
        NetUnparser::u8(&mut ret, self.protocol_address_size);
        NetUnparser::u16(&mut ret, self.opcode);

        for b in self.sender_ethernet_address {
            NetUnparser::u8(&mut ret, b);
        }
        NetUnparser::u32(&mut ret, self.sender_ip_address);

        for b in self.target_ethernet_address {
            NetUnparser::u8(&mut ret, b);
        }
        NetUnparser::u32(&mut ret, self.target_ip_address);

        ret
    }

    pub fn to_string(&self) -> String {
        format!(
            "opcode={}, sender={}/{}, target={}/{}",
            if self.opcode == ARPMessage::OPCODE_REQUEST {
                "REQUEST".to_string()
            } else if self.opcode == ARPMessage::OPCODE_REPLY {
                "REPLY".to_string()
            } else {
                "unknown type".to_string()
            },
            to_string(&self.sender_ethernet_address),
            Ipv4Addr::from(self.sender_ip_address).to_string(),
            to_string(&self.target_ethernet_address),
            Ipv4Addr::from(self.target_ip_address).to_string()
        )
    }
}
