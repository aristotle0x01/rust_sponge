use crate::util::parser::{NetParser, NetUnparser, ParseResult};
use crate::util::util::InternetChecksum;
use crate::SizeT;
use std::net::Ipv4Addr;

#[derive(Debug, Copy, Clone)]
pub struct IPv4Header {
    ver: u8,
    pub(crate) hlen: u8,
    tos: u8,
    pub len: u16,
    id: u16,
    df: bool,
    mf: bool,
    offset: u16,
    ttl: u8,
    pub proto: u8,
    pub(crate) cksum: u16,
    pub(crate) src: u32,
    pub(crate) dst: u32,
}
impl IPv4Header {
    pub const LENGTH: SizeT = 20 as SizeT;
    pub const DEFAULT_TTL: u8 = 128;
    pub const PROTO_TCP: u8 = 6;

    #[allow(dead_code)]
    pub fn new() -> IPv4Header {
        IPv4Header {
            ver: 4,
            hlen: (IPv4Header::LENGTH / 4) as u8,
            tos: 0,
            len: 0,
            id: 0,
            df: true,
            mf: false,
            offset: 0,
            ttl: IPv4Header::DEFAULT_TTL,
            proto: IPv4Header::PROTO_TCP,
            cksum: 0,
            src: 0,
            dst: 0,
        }
    }

    pub fn parse(&mut self, p: &mut NetParser) -> ParseResult {
        let original_serialized_version = p.buffer().clone();

        let data_size = p.buffer().size();
        if data_size < IPv4Header::LENGTH {
            return ParseResult::PacketTooShort;
        }

        let first_byte: u8 = p.u8();
        self.ver = first_byte >> 4;
        self.hlen = first_byte & 0x0f;
        self.tos = p.u8();
        self.len = p.u16();
        self.id = p.u16();

        let fo_val: u16 = p.u16();
        self.df = if (fo_val & 0x4000) != 0 { true } else { false };
        self.mf = if (fo_val & 0x2000) != 0 { true } else { false };
        self.offset = fo_val & 0x1fff;

        self.ttl = p.u8();
        self.proto = p.u8();
        self.cksum = p.u16();
        self.src = p.u32();
        self.dst = p.u32();

        if data_size < (4 * self.hlen) as usize {
            return ParseResult::PacketTooShort;
        }
        if self.ver != 4 {
            return ParseResult::WrongIPVersion;
        }
        if self.hlen < 5 {
            return ParseResult::HeaderTooShort;
        }
        if data_size != self.len as usize {
            return ParseResult::TruncatedPacket;
        }

        p.remove_prefix((self.hlen * 4 - IPv4Header::LENGTH as u8) as SizeT);
        if p.error() {
            return p.get_error();
        }

        let mut check = InternetChecksum::new(0);
        check.add(original_serialized_version.str());
        if check.value() != 0 {
            return ParseResult::BadChecksum;
        }

        return ParseResult::NoError;
    }

    pub fn serialize(&self) -> Vec<u8> {
        // sanity check
        assert_eq!(self.ver, 4, "wrong IP version");
        assert!(
            4 * self.hlen >= IPv4Header::LENGTH as u8,
            "IP header too short"
        );

        let mut ret: Vec<u8> = Vec::with_capacity((4 * self.hlen) as usize);

        let first_byte: u8 = (self.ver << 4) | (self.hlen & 0xf);
        NetUnparser::u8(&mut ret, first_byte);
        NetUnparser::u8(&mut ret, self.tos);
        NetUnparser::u16(&mut ret, self.len);
        NetUnparser::u16(&mut ret, self.id);

        let fo_val: u16 = if self.df { 0x4000 } else { 0 }
            | if self.mf { 0x2000 } else { 0 }
            | (self.offset & 0x1fff);
        NetUnparser::u16(&mut ret, fo_val);

        NetUnparser::u8(&mut ret, self.ttl);
        NetUnparser::u8(&mut ret, self.proto);

        NetUnparser::u16(&mut ret, self.cksum);

        NetUnparser::u32(&mut ret, self.src);
        NetUnparser::u32(&mut ret, self.dst);

        ret.shrink_to((4 * self.hlen) as usize);

        ret
    }

    pub fn payload_length(&self) -> u16 {
        println!("payload_length: {} {}", self.len, self.hlen);
        self.len - (4 * self.hlen) as u16
    }

    pub fn pseudo_cksum(&self) -> u32 {
        let mut pcksum: u32 = (self.src >> 16) + (self.src & 0xffff);
        pcksum += (self.dst >> 16) + (self.dst & 0xffff);
        pcksum += self.proto as u32;
        pcksum += self.payload_length() as u32;

        pcksum
    }

    pub fn to_string(&self) -> String {
        format!("IP version: {}\nIP hdr len: {}\nIP tos: {}\nIP dgram len: {}\nIP id: {}\nFlags: df: {} mf: {}\nOffset: {}\nTTL: {}\nProtocol: {}\nChecksum: {}\nSrc addr: {}\nDst addr: {}\n", self.ver, self.hlen, self.tos, self.len, self.id, self.df, self.mf, self.offset, self.ttl, self.proto, self.cksum, self.src, self.dst)
    }

    pub fn summary(&self) -> String {
        format!(
            "IPv{}, len={}, protocol={}, ttl={}, src={}, dst={})",
            self.ver,
            self.len,
            self.proto,
            if self.ttl >= 10 {
                "".to_string()
            } else {
                self.ttl.to_string()
            },
            Ipv4Addr::from(self.src).to_string(),
            Ipv4Addr::from(self.dst).to_string()
        )
    }
}
impl PartialEq<Self> for IPv4Header {
    fn eq(&self, other: &Self) -> bool {
        self.ver == other.ver
            && self.hlen == other.hlen
            && self.tos == other.tos
            && self.len == other.len
            && self.id == other.id
            && self.df == other.df
            && self.mf == other.mf
            && self.offset == other.offset
            && self.ttl == other.ttl
            && self.proto == other.proto
            && self.cksum == other.cksum
            && self.src == other.src
            && self.dst == other.dst
    }
}
impl Eq for IPv4Header {}
