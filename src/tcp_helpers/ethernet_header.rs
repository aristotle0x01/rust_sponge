use crate::util::parser::{NetParser, NetUnparser, ParseResult};
use crate::SizeT;

pub type EthernetAddress = [u8; 6];

pub const ETHERNET_BROADCAST: EthernetAddress = [0xffu8; 6];

pub fn to_string(address: &EthernetAddress) -> String {
    address.map(|x| format!("{:02X?}", x)).join(":")
}

#[derive(Debug, Copy, Clone)]
pub struct EthernetHeader {
    pub(crate) dst: EthernetAddress,
    pub(crate) src: EthernetAddress,
    pub pro_type: u16,
}
impl EthernetHeader {
    pub const LENGTH: SizeT = 14 as SizeT;
    pub const TYPE_IPV4: u16 = 0x800;
    pub const TYPE_ARP: u16 = 0x806;

    #[allow(dead_code)]
    pub fn new() -> EthernetHeader {
        EthernetHeader {
            dst: [0u8; 6],
            src: [0u8; 6],
            pro_type: 0,
        }
    }

    pub fn parse(&mut self, p: &mut NetParser<'_>) -> ParseResult {
        if p.buffer().size() < EthernetHeader::LENGTH {
            return ParseResult::PacketTooShort;
        }

        for b in self.dst.iter_mut() {
            *b = p.u8();
        }
        for b in self.src.iter_mut() {
            *b = p.u8();
        }
        self.pro_type = p.u16();

        p.get_error()
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut ret: Vec<u8> = Vec::with_capacity(EthernetHeader::LENGTH);

        for b in self.dst {
            NetUnparser::u8(&mut ret, b);
        }
        for b in self.src {
            NetUnparser::u8(&mut ret, b);
        }
        NetUnparser::u16(&mut ret, self.pro_type);

        ret
    }

    pub fn to_string(&self) -> String {
        format!(
            "dst={}, src={}, type={}",
            to_string(&self.dst),
            to_string(&self.src),
            if self.pro_type == EthernetHeader::TYPE_IPV4 {
                "IPV4".to_string()
            } else if self.pro_type == EthernetHeader::TYPE_ARP {
                "ARP".to_string()
            } else {
                format!("unknown type {}!", self.pro_type)
            }
        )
    }
}
impl PartialEq<Self> for EthernetHeader {
    fn eq(&self, other: &Self) -> bool {
        self.pro_type == other.pro_type && self.src == other.src && self.dst == other.dst
    }
}
impl Eq for EthernetHeader {}

#[cfg(test)]
mod tests {
    use crate::tcp_helpers::ethernet_header::EthernetHeader;
    use crate::util::buffer::Buffer;
    use crate::util::parser::NetParser;

    // cargo test --lib test_ether_header_to_string -- --show-output
    #[test]
    fn test_ether_header_to_string() {
        let eh = EthernetHeader {
            dst: [2u8; 6],
            src: [1u8; 6],
            pro_type: EthernetHeader::TYPE_IPV4,
        };
        assert!(eh.to_string().contains("IPV4"));
        println!("{}", eh.to_string());

        let eh = EthernetHeader {
            dst: [3u8; 6],
            src: [2u8; 6],
            pro_type: EthernetHeader::TYPE_ARP,
        };
        assert!(eh.to_string().contains("ARP"));
        println!("{}", eh.to_string());

        let eh = EthernetHeader {
            dst: [3u8; 6],
            src: [5u8; 6],
            pro_type: 10,
        };
        assert!(eh.to_string().contains("unknown"));
        println!("{}", eh.to_string());
    }

    // cargo test --lib test_ether_header_equal -- --show-output
    #[test]
    fn test_ether_header_equal() {
        let eh1 = EthernetHeader {
            dst: [2u8; 6],
            src: [1u8; 6],
            pro_type: EthernetHeader::TYPE_IPV4,
        };
        let eh2 = EthernetHeader {
            dst: [2u8; 6],
            src: [1u8; 6],
            pro_type: EthernetHeader::TYPE_IPV4,
        };
        assert_eq!(eh1, eh2);

        let eh3 = EthernetHeader {
            dst: [2u8; 6],
            src: [2u8; 6],
            pro_type: EthernetHeader::TYPE_IPV4,
        };
        assert_ne!(eh1, eh3);
    }

    // cargo test --lib test_ether_header_parse -- --show-output
    #[test]
    fn test_ether_header_parse() {
        let eh = EthernetHeader {
            dst: [2u8; 6],
            src: [1u8; 6],
            pro_type: EthernetHeader::TYPE_IPV4,
        };
        println!("eh: {}", eh.to_string());
        let mut bytes = Buffer::new(eh.serialize());

        let mut parser = NetParser::new(&mut bytes);
        let mut eh2 = EthernetHeader {
            dst: [0u8; 6],
            src: [0u8; 6],
            pro_type: 0,
        };
        println!("eh2 before parse: {}", eh2.to_string());
        eh2.parse(&mut parser);
        println!("eh2 after parse: {}", eh2.to_string());

        assert_eq!(eh2, eh);
    }
}
