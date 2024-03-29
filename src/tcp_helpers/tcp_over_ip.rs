use crate::tcp_helpers::fd_adapter::FdAdapterBase;
use crate::tcp_helpers::ipv4_header::IPv4Header;
use crate::tcp_helpers::tcp_segment::TCPSegment;
use crate::util::buffer::Buffer;
use crate::{InternetDatagram, SizeT};
use std::net::{Ipv4Addr, SocketAddrV4};

#[derive(Debug)]
pub struct TCPOverIPv4Adapter {
    pub fd_adapter_base: FdAdapterBase,
}
impl TCPOverIPv4Adapter {
    #[allow(dead_code)]
    pub fn new() -> TCPOverIPv4Adapter {
        TCPOverIPv4Adapter {
            fd_adapter_base: FdAdapterBase::new(),
        }
    }

    #[allow(dead_code)]
    pub fn unwrap_tcp_in_ip(&mut self, ip_dgram: InternetDatagram) -> Option<TCPSegment> {
        let c_s_ip: u32 = u32::from(self.fd_adapter_base.config().source.ip().clone());
        if !self.fd_adapter_base.listening() && ip_dgram.header().dst != c_s_ip {
            return None;
        }

        let c_d_ip: u32 = u32::from(self.fd_adapter_base.config().destination.ip().clone());
        if !self.fd_adapter_base.listening() && ip_dgram.header().src != c_d_ip {
            return None;
        }

        if ip_dgram.header().proto != IPv4Header::PROTO_TCP {
            return None;
        }

        let pseudo_cksum = ip_dgram.header().pseudo_cksum();
        let ip_dgram_dst = ip_dgram.header().dst;
        let ip_dgram_src = ip_dgram.header().src;
        let payload_ = ip_dgram.payload;

        let ret = TCPSegment::parse_new(payload_, pseudo_cksum);
        if ret.is_err() {
            return None;
        }
        let tcp_seg = ret.ok().unwrap();

        if tcp_seg.header().dport != self.fd_adapter_base.config().source.port() {
            return None;
        }

        if self.fd_adapter_base.listening() {
            if tcp_seg.header().syn && !tcp_seg.header().rst {
                self.fd_adapter_base.config_mut().source = SocketAddrV4::new(
                    Ipv4Addr::from(ip_dgram_dst),
                    self.fd_adapter_base.config().source.port(),
                );
                self.fd_adapter_base.config_mut().destination =
                    SocketAddrV4::new(Ipv4Addr::from(ip_dgram_src), tcp_seg.header().sport);
                self.fd_adapter_base.set_listening(false);
            } else {
                return None;
            }
        }

        if tcp_seg.header().sport != self.fd_adapter_base.config().destination.port() {
            return None;
        }

        Some(tcp_seg)
    }

    #[allow(dead_code)]
    pub fn wrap_tcp_in_ip(&mut self, seg: &mut TCPSegment) -> InternetDatagram {
        seg.header_mut().sport = self.fd_adapter_base.config().source.port();
        seg.header_mut().dport = self.fd_adapter_base.config().destination.port();

        let mut header = IPv4Header::new();
        header.src = u32::from(self.fd_adapter_base.config().source.ip().clone());
        header.dst = u32::from(self.fd_adapter_base.config().destination.ip().clone());
        header.len =
            ((header.hlen * 4 + seg.header().doff * 4) as SizeT + seg.payload().size()) as u16;

        let check_sum = header.pseudo_cksum();
        InternetDatagram::new(header, Buffer::new(seg.serialize(check_sum)))
    }
}
