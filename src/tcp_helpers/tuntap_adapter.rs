use crate::tcp_helpers::fd_adapter::{AsFdAdapterBase, AsFdAdapterBaseMut, FdAdapterBase};
use crate::tcp_helpers::ipv4_header::IPv4Header;
use crate::tcp_helpers::tcp_over_ip::TCPOverIPv4Adapter;
use crate::tcp_helpers::tcp_segment::TCPSegment;
use crate::util::buffer::Buffer;
use crate::util::file_descriptor::{AsFileDescriptor, AsFileDescriptorMut, FileDescriptor};
use crate::util::parser::ParseResult;
use crate::util::tun::TunFD;
use crate::InternetDatagram;

#[derive(Debug)]
pub struct TCPOverIPv4OverTunFdAdapter {
    ip_adapter: TCPOverIPv4Adapter,
    tun: TunFD,
}
impl AsFileDescriptor for TCPOverIPv4OverTunFdAdapter {
    fn as_file_descriptor(&self) -> &FileDescriptor {
        self.tun.as_file_descriptor()
    }
}
impl AsFileDescriptorMut for TCPOverIPv4OverTunFdAdapter {
    fn as_file_descriptor_mut(&mut self) -> &mut FileDescriptor {
        self.tun.as_file_descriptor_mut()
    }
}
impl AsFdAdapterBase for TCPOverIPv4OverTunFdAdapter {
    fn as_fd_adapter_base(&self) -> &FdAdapterBase {
        &self.ip_adapter.fd_adapter_base
    }
}
impl AsFdAdapterBaseMut for TCPOverIPv4OverTunFdAdapter {
    fn as_fd_adapter_base_mut(&mut self) -> &mut FdAdapterBase {
        &mut self.ip_adapter.fd_adapter_base
    }

    fn read_adp(&mut self) -> Option<TCPSegment> {
        let mut ip_dgram = InternetDatagram::new(IPv4Header::new(), Buffer::new(vec![]));
        let t = self.tun.read(u32::MAX);
        if ip_dgram.parse(&Buffer::from(t), 0) != ParseResult::NoError {
            None
        } else {
            self.ip_adapter.unwrap_tcp_in_ip(&ip_dgram)
        }
    }

    fn write_adp(&mut self, seg: &mut TCPSegment) {
        self.tun.write(
            &String::from_utf8_lossy(self.ip_adapter.wrap_tcp_in_ip(seg).serialize().as_slice()).to_string(),
            true,
        );
    }
}
impl TCPOverIPv4OverTunFdAdapter {
    #[allow(dead_code)]
    pub fn new(tun_: TunFD) -> TCPOverIPv4OverTunFdAdapter {
        TCPOverIPv4OverTunFdAdapter {
            ip_adapter: TCPOverIPv4Adapter::new(),
            tun: tun_,
        }
    }

    #[allow(dead_code)]
    pub fn tun(&self) -> &TunFD {
        &self.tun
    }

    #[allow(dead_code)]
    pub fn tun_mut(&mut self) -> &mut TunFD {
        &mut self.tun
    }
}
