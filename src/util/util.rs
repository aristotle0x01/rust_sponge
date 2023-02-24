use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

pub fn system_call(attempt: &str, return_value: i32, errno_mask: i32) -> i32 {
    // let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
    let d = std::io::Error::last_os_error();
    let errno = d.raw_os_error().unwrap_or(0);
    if return_value >= 0 || errno == errno_mask {
        return_value
    } else {
        panic!(
            "{}",
            format!(
                "libc::{} failed with return val:{}, errno:{} {:?}",
                attempt, return_value, errno, d
            )
        )
    }
}

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

        (!ret) as u16
    }

    #[allow(dead_code)]
    pub fn add(&mut self, data: &[u8]) {
        let bytes = data;
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

// the number of milliseconds since the program started
static mut PROGRAM_START: Option<Instant> = None;
static mut PROGRAM_STARTED: AtomicBool = AtomicBool::new(false);
pub fn timestamp_ms() -> u64 {
    let duration = unsafe {
        if !PROGRAM_STARTED.load(Ordering::SeqCst) && PROGRAM_START.is_none() {
            PROGRAM_STARTED.store(true, Ordering::SeqCst);
            PROGRAM_START = Some(Instant::now());
        }

        PROGRAM_START.unwrap().elapsed()
    };

    duration.as_millis() as u64
}
