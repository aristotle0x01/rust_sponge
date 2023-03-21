use crate::tcp_helpers::arp_message::ARPMessage;
use crate::tcp_helpers::ethernet_frame::EthernetFrame;
use crate::tcp_helpers::ethernet_header::{
    to_string, EthernetAddress, EthernetHeader, ETHERNET_BROADCAST,
};
use crate::tcp_helpers::ipv4_header::IPv4Header;
use crate::util::buffer::Buffer;
use crate::util::parser::ParseResult;
use crate::{InternetDatagram, SizeT};
use std::collections::{BTreeMap, VecDeque};
use std::net::Ipv4Addr;

#[derive(Debug)]
#[allow(dead_code)]
pub struct NetworkInterface {
    ethernet_address: EthernetAddress,
    ip_address: Ipv4Addr,
    frames_out: VecDeque<EthernetFrame>,
    frames_need_fill: BTreeMap<u32, Vec<EthernetFrame>>,
    ip_mac_cache: BTreeMap<u32, (SizeT, EthernetAddress)>,
    arp_request_in_flight: BTreeMap<u32, SizeT>,
    ms_total_tick: SizeT,
}
impl NetworkInterface {
    const GAP_30S: SizeT = 30 * 1000;
    const GAP_5S: SizeT = 5 * 1000;

    #[allow(dead_code)]
    pub fn new(ether_addr: EthernetAddress, ip_addr: Ipv4Addr) -> NetworkInterface {
        eprintln!(
            "DEBUG: Network interface has Ethernet address {} and IP address {}",
            to_string(&ether_addr),
            ip_addr.to_string()
        );
        NetworkInterface {
            ethernet_address: ether_addr,
            ip_address: ip_addr,
            frames_out: Default::default(),
            frames_need_fill: Default::default(),
            ip_mac_cache: Default::default(),
            arp_request_in_flight: Default::default(),
            ms_total_tick: 0,
        }
    }

    #[allow(dead_code)]
    pub fn send_datagram(&mut self, dgram: InternetDatagram, next_hop: &Ipv4Addr) {
        let next_hop_ip = u32::from(next_hop.clone());

        let mut frame = EthernetFrame {
            header: EthernetHeader::new(),
            payload: Buffer::new(dgram.serialize()),
        };
        frame.header_mut().src = self.ethernet_address.clone();
        frame.header_mut().pro_type = EthernetHeader::TYPE_IPV4;

        if let Some((_, t2_)) = self.ip_mac_cache.get(&next_hop_ip) {
            frame.header_mut().dst = t2_.clone();
            self.frames_out.push_back(frame);
            return;
        }

        if let Some(list) = self.frames_need_fill.get_mut(&next_hop_ip) {
            list.push(frame);
        } else {
            let mut list: Vec<EthernetFrame> = Vec::new();
            list.push(frame);
            self.frames_need_fill.insert(next_hop_ip, list);
        }

        if let Some(_) = self.arp_request_in_flight.get(&next_hop_ip) {
            return;
        }

        let mut arp = ARPMessage::new();
        arp.opcode = ARPMessage::OPCODE_REQUEST;
        arp.sender_ip_address = u32::from(self.ip_address);
        arp.sender_ethernet_address = self.ethernet_address;
        arp.target_ip_address = next_hop_ip;
        let mut arp_frame = EthernetFrame::new();
        arp_frame.payload = Buffer::new(arp.serialize());
        arp_frame.header_mut().src = self.ethernet_address;
        arp_frame.header_mut().dst = ETHERNET_BROADCAST;
        arp_frame.header_mut().pro_type = EthernetHeader::TYPE_ARP;
        self.frames_out.push_back(arp_frame);

        self.arp_request_in_flight
            .insert(next_hop_ip, self.ms_total_tick);
    }

    #[allow(dead_code)]
    pub fn recv_frame(&mut self, frame: &EthernetFrame) -> Option<InternetDatagram> {
        if frame.header().dst != ETHERNET_BROADCAST && frame.header().dst != self.ethernet_address {
            return None;
        }

        if frame.header().pro_type == EthernetHeader::TYPE_IPV4 {
            let mut datagram = InternetDatagram::new(IPv4Header::new(), frame.payload.clone());
            let r = datagram.parse(0);
            return if r == ParseResult::NoError {
                Some(datagram)
            } else {
                None
            };
        }

        if frame.header().pro_type != EthernetHeader::TYPE_ARP {
            return None;
        }

        let mut arp = ARPMessage::new();
        let r = arp.parse(frame.payload().str().to_vec());
        if r != ParseResult::NoError {
            return None;
        }

        if arp.opcode == ARPMessage::OPCODE_REQUEST
            && arp.target_ip_address == u32::from(self.ip_address)
        {
            let mut reply = ARPMessage::new();
            reply.opcode = ARPMessage::OPCODE_REPLY;
            reply.sender_ethernet_address = self.ethernet_address;
            reply.sender_ip_address = u32::from(self.ip_address);
            reply.target_ip_address = arp.sender_ip_address;
            reply.target_ethernet_address = arp.sender_ethernet_address;

            let mut reply_frame = EthernetFrame::new();
            reply_frame.payload = Buffer::new(reply.serialize());
            reply_frame.header_mut().src = self.ethernet_address;
            reply_frame.header_mut().dst = frame.header().src;
            reply_frame.header_mut().pro_type = EthernetHeader::TYPE_ARP;
            self.frames_out.push_back(reply_frame);
        }

        self.ip_mac_cache.insert(
            arp.sender_ip_address,
            (self.ms_total_tick, arp.sender_ethernet_address),
        );

        self.frames_need_fill.retain(|ip, list| {
            return if let Some((_, ether_addr)) = self.ip_mac_cache.get(ip) {
                while !list.is_empty() {
                    let mut l = list.remove(0);
                    l.header_mut().dst = ether_addr.clone();
                    self.frames_out.push_back(l);
                }
                false
            } else {
                true
            };
        });

        None
    }

    #[allow(dead_code)]
    pub fn tick(&mut self, ms_since_last_tick: SizeT) {
        self.ms_total_tick += ms_since_last_tick;

        self.ip_mac_cache
            .retain(|_, (t, _)| (*t + NetworkInterface::GAP_30S) > self.ms_total_tick);
        self.arp_request_in_flight
            .retain(|_, t| (*t + NetworkInterface::GAP_5S) > self.ms_total_tick);
    }

    #[allow(dead_code)]
    pub fn frames_out(&self) -> &VecDeque<EthernetFrame> {
        &self.frames_out
    }

    #[allow(dead_code)]
    pub fn frames_out_mut(&mut self) -> &mut VecDeque<EthernetFrame> {
        &mut self.frames_out
    }
}
