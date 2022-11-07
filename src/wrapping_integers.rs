use std::fmt::Formatter;
use std::ops;

#[derive(Debug)]
pub struct WrappingInt32 {
    raw_value: u32,
}
impl WrappingInt32 {
    #[allow(dead_code)]
    pub fn new(raw: u32) -> WrappingInt32 {
        WrappingInt32 { raw_value: raw }
    }

    pub fn wrap(n: u64, isn: &WrappingInt32) -> WrappingInt32 {
        let t64: u64 = n + (isn.raw_value() as u64);
        let t32: u64 = t64 % (1 << 32);
        WrappingInt32::new(t32 as u32)
    }

    pub fn unwrap(n: &WrappingInt32, isn: &WrappingInt32, checkpoint: u64) -> u64 {
        const STEP: u64 = 1 << 32;
        let gap: u64 = if n.raw_value() >= isn.raw_value() {
            (n.raw_value() - isn.raw_value()) as u64
        } else {
            STEP - (isn.raw_value() as u64) + (n.raw_value() as u64)
        };

        let multi: u64 = checkpoint / STEP;
        let mut i: u64 = 0;
        if multi > 2 {
            i = multi - 1;
        }

        let mut v1: u64 = gap + STEP * i;
        i += 1;
        let mut d1: u64 = if checkpoint >= v1 {
            checkpoint - v1
        } else {
            v1 - checkpoint
        };
        let mut v2: u64 = gap + STEP * i;
        i += 1;
        let mut d2: u64 = if checkpoint >= v2 {
            checkpoint - v2
        } else {
            v2 - checkpoint
        };
        let mut v3: u64 = gap + STEP * i;
        i += 1;
        let mut d3: u64 = if checkpoint >= v3 {
            checkpoint - v3
        } else {
            v3 - checkpoint
        };

        let t64: u64;
        loop {
            // monotonically increasing
            if d3 >= d2 && d2 >= d1 {
                t64 = v1;
                break;
            }
            // decrement then increment
            if d3 >= d2 && d2 <= d1 {
                t64 = v2;
                break;
            }
            v1 = v2;
            d1 = d2;
            v2 = v3;
            d2 = d3;
            v3 = gap + STEP * i;
            d3 = if checkpoint >= v3 {
                checkpoint - v3
            } else {
                v3 - checkpoint
            };

            i += 1;
        }

        t64
    }

    #[allow(dead_code)]
    pub fn raw_value(&self) -> u32 {
        self.raw_value
    }
}

impl ops::Add<WrappingInt32> for WrappingInt32 {
    type Output = WrappingInt32;

    fn add(self, rhs: WrappingInt32) -> Self::Output {
        WrappingInt32::new(self.raw_value() + rhs.raw_value())
    }
}
impl ops::Sub<WrappingInt32> for WrappingInt32 {
    type Output = WrappingInt32;

    fn sub(self, rhs: WrappingInt32) -> Self::Output {
        WrappingInt32::new(self.raw_value() - rhs.raw_value())
    }
}
impl ops::Sub<u32> for WrappingInt32 {
    type Output = WrappingInt32;

    fn sub(self, rhs: u32) -> Self::Output {
        WrappingInt32::new(self.raw_value() - rhs)
    }
}

impl PartialEq<Self> for WrappingInt32 {
    fn eq(&self, other: &Self) -> bool {
        self.raw_value() == other.raw_value()
    }
}
impl Eq for WrappingInt32 {}

impl std::fmt::Display for WrappingInt32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({})", self.raw_value)
    }
}
