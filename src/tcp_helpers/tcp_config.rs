use crate::wrapping_integers::WrappingInt32;
use crate::SizeT;

#[derive(Debug)]
pub struct TCPConfig {
    pub rt_timeout: u16,
    pub recv_capacity: SizeT,
    pub send_capacity: SizeT,
    pub fixed_isn: Option<WrappingInt32>,
}
impl TCPConfig {
    pub const DEFAULT_CAPACITY: SizeT = 64000 as SizeT;
    pub const MAX_PAYLOAD_SIZE: SizeT = 1452 as SizeT;
    pub const TIMEOUT_DFLT: u16 = 1000;
    pub const MAX_RETX_ATTEMPTS: u32 = 8;
}
impl Default for TCPConfig {
    fn default() -> TCPConfig {
        TCPConfig {
            rt_timeout: TCPConfig::TIMEOUT_DFLT,
            recv_capacity: TCPConfig::DEFAULT_CAPACITY,
            send_capacity: TCPConfig::DEFAULT_CAPACITY,
            fixed_isn: None,
        }
    }
}
// let p1 = TCPConfig { ..Default::default() };
