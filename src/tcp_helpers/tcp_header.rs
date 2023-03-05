use crate::util::parser::{NetParser, NetUnparser, ParseResult};
use crate::wrapping_integers::WrappingInt32;
use crate::SizeT;

#[derive(Debug, Copy, Clone)]
pub struct TCPHeader {
    pub sport: u16,
    pub dport: u16,
    pub seqno: WrappingInt32,
    pub ackno: WrappingInt32,
    pub(crate) doff: u8,
    urg: bool,
    pub ack: bool,
    psh: bool,
    pub rst: bool,
    pub syn: bool,
    pub fin: bool,
    pub win: u16,
    pub(crate) cksum: u16,
    uptr: u16,
}
impl TCPHeader {
    pub const LENGTH: SizeT = 20 as SizeT;

    #[allow(dead_code)]
    pub fn new() -> TCPHeader {
        TCPHeader {
            sport: 0,
            dport: 0,
            seqno: WrappingInt32::new(0),
            ackno: WrappingInt32::new(0),
            doff: (TCPHeader::LENGTH / 4) as u8,
            urg: false,
            ack: false,
            psh: false,
            rst: false,
            syn: false,
            fin: false,
            win: 0,
            cksum: 0,
            uptr: 0,
        }
    }

    pub fn parse(&mut self, p: &mut NetParser<'_>) -> ParseResult {
        self.sport = p.u16();
        self.dport = p.u16();
        self.seqno = WrappingInt32::new(p.u32());
        self.ackno = WrappingInt32::new(p.u32());
        self.doff = p.u8() >> 4;

        let fl_b = p.u8();
        self.urg = if fl_b & 0b00100000 != 0 { true } else { false };
        self.ack = if fl_b & 0b00010000 != 0 { true } else { false };
        self.psh = if fl_b & 0b00001000 != 0 { true } else { false };
        self.rst = if fl_b & 0b00000100 != 0 { true } else { false };
        self.syn = if fl_b & 0b00000010 != 0 { true } else { false };
        self.fin = if fl_b & 0b00000001 != 0 { true } else { false };

        self.win = p.u16();
        self.cksum = p.u16();
        self.uptr = p.u16();

        if self.doff < 5 {
            return ParseResult::HeaderTooShort;
        }

        // skip any options or anything extra in the header
        p.remove_prefix(((self.doff * 4) as SizeT - TCPHeader::LENGTH) as SizeT);

        if p.error() {
            return p.get_error();
        }

        return ParseResult::NoError;
    }

    pub fn serialize(&self) -> Vec<u8> {
        // sanity check
        assert!(self.doff >= 5, "TCP header too short");

        let mut ret: Vec<u8> = Vec::with_capacity((4 * self.doff) as usize);

        NetUnparser::u16(&mut ret, self.sport);
        NetUnparser::u16(&mut ret, self.dport);
        NetUnparser::u32(&mut ret, self.seqno.raw_value());
        NetUnparser::u32(&mut ret, self.ackno.raw_value());
        NetUnparser::u8(&mut ret, self.doff << 4);

        let fl_b: u8 = if self.urg { 0b00100000 } else { 0 }
            | if self.ack { 0b00010000 } else { 0 }
            | if self.psh { 0b00001000 } else { 0 }
            | if self.rst { 0b00000100 } else { 0 }
            | if self.syn { 0b00000010 } else { 0 }
            | if self.fin { 0b00000001 } else { 0 };
        NetUnparser::u8(&mut ret, fl_b);
        NetUnparser::u16(&mut ret, self.win);

        NetUnparser::u16(&mut ret, self.cksum);

        NetUnparser::u16(&mut ret, self.uptr);

        ret.shrink_to((4 * self.doff) as usize);

        ret
    }

    pub fn to_string(&self) -> String {
        format!("TCP source port: {}\nTCP dest port: {}\nTCP seqno: {}\nTCP ackno: {}\nTCP doff: {}\nFlags: urg: {} ack: {} psh: {} rst: {} syn: {} fin: {}\nTCP winsize: {}\nTCP cksum: {}\nTCP uptr: {}\n", self.sport, self.dport, self.seqno, self.ackno, self.doff, self.urg, self.ack, self.psh, self.rst, self.syn, self.fin, self.win, self.cksum, self.uptr)
    }

    pub fn summary(&self) -> String {
        format!(
            "Header(flags={}{}{}{},seqno={},ack={},win={})",
            if self.syn { "S" } else { "" },
            if self.ack { "A" } else { "" },
            if self.rst { "R" } else { "" },
            if self.fin { "F" } else { "" },
            self.seqno,
            self.ackno,
            self.win
        )
    }
}

impl PartialEq<Self> for TCPHeader {
    fn eq(&self, other: &Self) -> bool {
        self.seqno == other.seqno
            && self.ackno == other.ackno
            && self.doff == other.doff
            && self.urg == other.urg
            && self.ack == other.ack
            && self.psh == other.psh
            && self.rst == other.rst
            && self.syn == other.syn
            && self.fin == other.fin
            && self.win == other.win
            && self.uptr == other.uptr
    }
}
impl Eq for TCPHeader {}
