use crate::util::buffer::Buffer;
use crate::SizeT;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ParseResult {
    NoError = 0,
    BadChecksum,
    PacketTooShort,
    WrongIPVersion,
    HeaderTooShort,
    TruncatedPacket,
    Unsupported,
}

pub fn as_string(r: ParseResult) -> String {
    match r {
        ParseResult::NoError => "NoError".to_string(),
        ParseResult::BadChecksum => "BadChecksum".to_string(),
        ParseResult::PacketTooShort => "PacketTooShort".to_string(),
        ParseResult::WrongIPVersion => "WrongIPVersion".to_string(),
        ParseResult::HeaderTooShort => "HeaderTooShort".to_string(),
        ParseResult::TruncatedPacket => "TruncatedPacket".to_string(),
        _ => panic!("Unsupported"),
    }
}

#[derive(Debug)]
pub struct NetParser<'a> {
    buffer: &'a mut Buffer,
    error: ParseResult,
}
impl NetParser<'_> {
    #[allow(dead_code)]
    pub fn new(_buffer: &mut Buffer) -> NetParser<'_> {
        NetParser {
            buffer: _buffer,
            error: ParseResult::NoError,
        }
    }

    #[allow(dead_code)]
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    #[allow(dead_code)]
    pub fn get_error(&self) -> ParseResult {
        self.error
    }

    #[allow(dead_code)]
    pub fn set_error(&mut self, res: ParseResult) {
        self.error = res;
    }

    #[allow(dead_code)]
    pub fn error(&self) -> bool {
        self.get_error() != ParseResult::NoError
    }

    #[allow(dead_code)]
    pub fn u32(&mut self) -> u32 {
        let len: SizeT = 4;
        self._check_size(len);
        if self.error() {
            return 0;
        }

        let mut ret: u32 = 0;
        for _i in 0..len {
            ret <<= 8;
            ret += self.buffer.at(_i) as u32;
        }

        self.buffer.remove_prefix(len);

        ret
    }

    #[allow(dead_code)]
    pub fn u16(&mut self) -> u16 {
        let len: SizeT = 2;
        self._check_size(len);
        if self.error() {
            return 0;
        }

        let mut ret: u16 = 0;
        for _i in 0..len {
            ret <<= 8;
            ret += self.buffer.at(_i) as u16;
        }

        self.buffer.remove_prefix(len);

        ret
    }

    #[allow(dead_code)]
    pub fn u8(&mut self) -> u8 {
        let len: SizeT = 1;
        self._check_size(len);
        if self.error() {
            return 0;
        }

        let ret: u8 = self.buffer.at(0);
        self.buffer.remove_prefix(len);

        ret
    }

    #[allow(dead_code)]
    pub fn remove_prefix(&mut self, n: SizeT) {
        self._check_size(n);
        if self.error() {
            return;
        }
        self.buffer.remove_prefix(n);
    }

    fn _check_size(&mut self, size: SizeT) {
        if size > self.buffer.size() {
            self.set_error(ParseResult::PacketTooShort);
        }
    }
}

#[derive(Debug)]
pub struct NetUnparser;
impl NetUnparser {
    #[allow(dead_code)]
    pub fn new() -> NetUnparser {
        NetUnparser {}
    }

    pub fn u32(s: &mut Vec<u8>, val: u32) {
        let len: SizeT = 4;
        for _i in 0..len {
            let the_byte: u8 = ((val >> ((len - _i - 1) * 8)) & 0xff) as u8;
            s.push(the_byte);
        }
    }

    pub fn u16(s: &mut Vec<u8>, val: u16) {
        let len: SizeT = 2;
        for _i in 0..len {
            let the_byte: u8 = ((val >> ((len - _i - 1) * 8)) & 0xff) as u8;
            s.push(the_byte);
        }
    }

    pub fn u8(s: &mut Vec<u8>, val: u8) {
        let len: SizeT = 1;
        for _i in 0..len {
            let the_byte: u8 = ((val >> ((len - _i - 1) * 8)) & 0xff) as u8;
            s.push(the_byte);
        }
    }
}
