use crate::wrapping_integers::WrappingInt32;
use crate::SizeT;
use std::fmt;
use std::net::SocketAddrV4;

#[derive(Debug, Copy, Clone)]
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
impl fmt::Display for TCPConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(rt_timeout:{}, recv_capacity:{}, send_capacity:{}, isn:{})",
            self.rt_timeout,
            self.recv_capacity,
            self.send_capacity,
            if self.fixed_isn.is_some() {
                format!("{}", self.fixed_isn.unwrap())
            } else {
                "None".to_string()
            }
        )
    }
}
// let p1 = TCPConfig { ..Default::default() };

#[derive(Debug, Copy, Clone)]
pub struct FdAdapterConfig {
    pub source: SocketAddrV4,
    pub destination: SocketAddrV4,
    pub loss_rate_dn: u16,
    pub loss_rate_up: u16,
}
