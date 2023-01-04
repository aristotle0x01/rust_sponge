use crate::tcp_helpers::tcp_config::FdAdapterConfig;
use crate::SizeT;
use std::net::SocketAddrV4;
use std::str::FromStr;

#[derive(Debug)]
pub struct FdAdapterBase {
    cfg: FdAdapterConfig,
    listen: bool,
}
impl FdAdapterBase {
    #[allow(dead_code)]
    pub fn new() -> FdAdapterBase {
        FdAdapterBase {
            cfg: FdAdapterConfig {
                source: SocketAddrV4::from_str("127.0.0.1:80").unwrap(),
                destination: SocketAddrV4::from_str("127.0.0.1:80").unwrap(),
                loss_rate_dn: 0,
                loss_rate_up: 0,
            },
            listen: false,
        }
    }

    #[allow(dead_code)]
    pub fn set_listening(&mut self, l: bool) {
        self.listen = l;
    }

    #[allow(dead_code)]
    pub fn listening(&self) -> bool {
        self.listen
    }

    #[allow(dead_code)]
    pub fn config(&self) -> &FdAdapterConfig {
        &self.cfg
    }

    #[allow(dead_code)]
    pub fn config_mut(&mut self) -> &mut FdAdapterConfig {
        &mut self.cfg
    }

    #[allow(dead_code)]
    pub fn tick(&mut self, _t: SizeT) {}
}
