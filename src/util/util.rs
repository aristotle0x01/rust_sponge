use crate::StringView;

#[derive(Debug)]
pub struct InternetChecksum {
    sum: u32,
    parity: bool,
}
impl InternetChecksum {
    #[allow(dead_code)]
    pub fn new(_sum: u32) -> InternetChecksum {
        InternetChecksum {
            sum: _sum,
            parity: false,
        }
    }

    #[allow(dead_code)]
    pub fn value(&self) -> u16 {
        let mut ret = self.sum;

        while ret > 0xffff {
            ret = (ret >> 16) + (ret & 0xffff);
        }

        !ret as u16
    }

    #[allow(dead_code)]
    pub fn add(&mut self, data: StringView<'_>) {
        let bytes = data.as_bytes();
        for _i in 0..bytes.len() {
            let mut val: u16 = bytes[_i] as u16;
            if !self.parity {
                val <<= 8;
            }
            self.sum += val as u32;
            self.parity = !self.parity;
        }
    }
}
