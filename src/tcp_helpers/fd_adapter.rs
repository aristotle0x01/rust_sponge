use crate::tcp_helpers::tcp_config::FdAdapterConfig;
use crate::tcp_helpers::tcp_segment::TCPSegment;
use crate::util::buffer::Buffer;
use crate::util::file_descriptor::{AsFileDescriptor, AsFileDescriptorMut, FileDescriptor};
use crate::util::socket::UDPSocket;
use crate::SizeT;
use std::net::{Ipv4Addr, SocketAddrV4};

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
                source: SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0),
                destination: SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0),
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

pub trait AsFdAdapterBase {
    fn as_fd_adapter_base(&self) -> &FdAdapterBase;

    fn listening(&self) -> bool {
        self.as_fd_adapter_base().listening()
    }

    fn config(&self) -> &FdAdapterConfig {
        self.as_fd_adapter_base().config()
    }
}
pub trait AsFdAdapterBaseMut: AsFdAdapterBase {
    fn as_fd_adapter_base_mut(&mut self) -> &mut FdAdapterBase;

    fn set_listening(&mut self, _listen: bool) {
        self.as_fd_adapter_base_mut().set_listening(_listen);
    }

    fn config_mut(&mut self) -> &mut FdAdapterConfig {
        self.as_fd_adapter_base_mut().config_mut()
    }

    fn set_config(&mut self, conf: FdAdapterConfig) {
        let mut t = self.as_fd_adapter_base_mut().config_mut();
        t.loss_rate_dn = conf.loss_rate_dn;
        t.loss_rate_up = conf.loss_rate_up;
        t.source = conf.source;
        t.destination = conf.destination;
    }

    fn tick(&mut self, _t: SizeT) {
        self.as_fd_adapter_base_mut().tick(_t);
    }

    fn read_adp(&mut self) -> Option<TCPSegment>;

    fn write_adp(&mut self, seg: &mut TCPSegment);
}

#[derive(Debug)]
pub struct TCPOverUDPSocketAdapter {
    fd_adapter_base: FdAdapterBase,
    sock: UDPSocket,
}
impl AsFileDescriptor for TCPOverUDPSocketAdapter {
    fn as_file_descriptor(&self) -> &FileDescriptor {
        self.sock.as_file_descriptor()
    }
}
impl AsFileDescriptorMut for TCPOverUDPSocketAdapter {
    fn as_file_descriptor_mut(&mut self) -> &mut FileDescriptor {
        self.sock.as_file_descriptor_mut()
    }
}
impl AsFdAdapterBase for TCPOverUDPSocketAdapter {
    fn as_fd_adapter_base(&self) -> &FdAdapterBase {
        &self.fd_adapter_base
    }
}
impl AsFdAdapterBaseMut for TCPOverUDPSocketAdapter {
    fn as_fd_adapter_base_mut(&mut self) -> &mut FdAdapterBase {
        &mut self.fd_adapter_base
    }

    fn read_adp(&mut self) -> Option<TCPSegment> {
        let (source_address, payload) = self.sock.recv(65536);

        let b =
            FdAdapterConfig::eq_to_sockaddr(&source_address, &self.config().destination);
        if !self.listening() && !b {
            return None;
        }

        let ret = TCPSegment::parse_new(Buffer::new(payload), 0);
        if ret.is_err() {
            return None;
        }
        let seg = ret.ok().unwrap();

        if self.listening() {
            if seg.header().syn && !seg.header().rst {
                self.config_mut().destination =
                    FdAdapterConfig::from_sockaddr(&source_address);
                self.set_listening(false);
            } else {
                return None;
            }
        }

        Some(seg)
    }

    fn write_adp(&mut self, seg: &mut TCPSegment) {
        seg.header_mut().sport = self.config().source.port();
        seg.header_mut().dport = self.config().destination.port();

        self.sock.sendto(
            &self.fd_adapter_base.cfg.destination,
            &mut seg.serialize(0),
        );
    }
}
impl TCPOverUDPSocketAdapter {
    #[allow(dead_code)]
    pub fn new(sock: UDPSocket) -> TCPOverUDPSocketAdapter {
        TCPOverUDPSocketAdapter {
            fd_adapter_base: FdAdapterBase::new(),
            sock,
        }
    }

    #[allow(dead_code)]
    pub fn udp_sock(&self) -> &UDPSocket {
        &self.sock
    }

    #[allow(dead_code)]
    pub fn udp_sock_mut(&mut self) -> &mut UDPSocket {
        &mut self.sock
    }
}
