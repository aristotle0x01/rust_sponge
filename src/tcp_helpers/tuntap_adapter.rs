use crate::network_interface::NetworkInterface;
use crate::tcp_helpers::ethernet_frame::EthernetFrame;
use crate::tcp_helpers::ethernet_header::EthernetAddress;
use crate::tcp_helpers::fd_adapter::{AsFdAdapterBase, AsFdAdapterBaseMut, FdAdapterBase};
use crate::tcp_helpers::ipv4_header::IPv4Header;
use crate::tcp_helpers::tcp_over_ip::TCPOverIPv4Adapter;
use crate::tcp_helpers::tcp_segment::TCPSegment;
use crate::util::buffer::Buffer;
use crate::util::file_descriptor::{AsFileDescriptor, AsFileDescriptorMut, FileDescriptor};
use crate::util::parser::ParseResult;
use crate::util::tun::{TapFD, TunFD};
use crate::{InternetDatagram, SizeT};
use std::net::Ipv4Addr;

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
        let t = self.tun.read(u32::MAX);
        let mut ip_dgram = InternetDatagram::new(IPv4Header::new(), Buffer::new(t));
        if ip_dgram.parse(0) != ParseResult::NoError {
            None
        } else {
            self.ip_adapter.unwrap_tcp_in_ip(ip_dgram)
        }
    }

    fn write_adp(&mut self, seg: &mut TCPSegment) {
        self.tun.write(
            self.ip_adapter.wrap_tcp_in_ip(seg).serialize().as_slice(),
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

#[derive(Debug)]
pub struct TCPOverIPv4OverEthernetAdapter {
    ip_adapter: TCPOverIPv4Adapter,
    tap: TapFD,
    interface: NetworkInterface,
    next_hop: Ipv4Addr,
}
impl AsFileDescriptor for TCPOverIPv4OverEthernetAdapter {
    fn as_file_descriptor(&self) -> &FileDescriptor {
        self.tap.as_file_descriptor()
    }
}
impl AsFileDescriptorMut for TCPOverIPv4OverEthernetAdapter {
    fn as_file_descriptor_mut(&mut self) -> &mut FileDescriptor {
        self.tap.as_file_descriptor_mut()
    }
}
impl AsFdAdapterBase for TCPOverIPv4OverEthernetAdapter {
    fn as_fd_adapter_base(&self) -> &FdAdapterBase {
        &self.ip_adapter.fd_adapter_base
    }
}
impl AsFdAdapterBaseMut for TCPOverIPv4OverEthernetAdapter {
    fn as_fd_adapter_base_mut(&mut self) -> &mut FdAdapterBase {
        &mut self.ip_adapter.fd_adapter_base
    }

    fn read_adp(&mut self) -> Option<TCPSegment> {
        let mut frame = EthernetFrame::new();

        let r = frame.parse(self.tap.read(u32::MAX));
        if r != ParseResult::NoError {
            return None;
        }

        let ip_dgram = self.interface.recv_frame(&frame);

        self.send_pending();

        if ip_dgram.is_some() {
            return self.ip_adapter.unwrap_tcp_in_ip(ip_dgram.unwrap());
        }

        None
    }

    fn write_adp(&mut self, seg: &mut TCPSegment) {
        let ip_dgram = self.ip_adapter.wrap_tcp_in_ip(seg);
        self.interface.send_datagram(ip_dgram, &self.next_hop);

        self.send_pending();
    }
}
impl TCPOverIPv4OverEthernetAdapter {
    #[allow(dead_code)]
    pub fn new(
        tap_: TapFD,
        eth_address: EthernetAddress,
        ip_address: Ipv4Addr,
        next_hop_: Ipv4Addr,
    ) -> TCPOverIPv4OverEthernetAdapter {
        let mut t = TCPOverIPv4OverEthernetAdapter {
            ip_adapter: TCPOverIPv4Adapter::new(),
            tap: tap_,
            interface: NetworkInterface::new(eth_address, ip_address),
            next_hop: next_hop_,
        };

        let dummy = EthernetFrame::new();
        t.tap_mut().write(dummy.serialize().as_slice(), true);

        t
    }

    #[allow(dead_code)]
    fn send_pending(&mut self) {
        while !self.interface.frames_out().is_empty() {
            let frame = self.interface.frames_out_mut().pop_front().unwrap();
            self.tap.write(frame.serialize().as_slice(), true);
        }
    }

    #[allow(dead_code)]
    pub fn tick(&mut self, ms_since_last_tick: SizeT) {
        self.interface.tick(ms_since_last_tick);
        self.send_pending();
    }

    #[allow(dead_code)]
    pub fn tap(&self) -> &TapFD {
        &self.tap
    }

    #[allow(dead_code)]
    pub fn tap_mut(&mut self) -> &mut TapFD {
        &mut self.tap
    }
}
