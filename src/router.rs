use crate::network_interface::NetworkInterface;
use crate::tcp_helpers::ethernet_frame::EthernetFrame;
use crate::{InternetDatagram, SizeT};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::net::Ipv4Addr;

#[derive(Debug)]
pub struct AsyncNetworkInterface {
    intf: NetworkInterface,
    datagrams_out: VecDeque<InternetDatagram>,
}
impl AsyncNetworkInterface {
    #[allow(dead_code)]
    pub fn new(intf_: NetworkInterface) -> AsyncNetworkInterface {
        AsyncNetworkInterface {
            intf: intf_,
            datagrams_out: Default::default(),
        }
    }

    #[allow(dead_code)]
    pub fn send_datagram(&mut self, dgram: InternetDatagram, next_hop: &Ipv4Addr) {
        self.intf.send_datagram(dgram, next_hop);
    }

    #[allow(dead_code)]
    pub fn recv_frame(&mut self, frame: &EthernetFrame) {
        let opt_dgram = self.intf.recv_frame(frame);
        if opt_dgram.is_some() {
            self.datagrams_out.push_back(opt_dgram.unwrap());
        }
    }

    #[allow(dead_code)]
    pub fn tick(&mut self, ms_since_last_tick: SizeT) {
        self.intf.tick(ms_since_last_tick);
    }

    #[allow(dead_code)]
    pub fn frames_out(&self) -> &VecDeque<EthernetFrame> {
        self.intf.frames_out()
    }

    #[allow(dead_code)]
    pub fn frames_out_mut(&mut self) -> &mut VecDeque<EthernetFrame> {
        self.intf.frames_out_mut()
    }

    #[allow(dead_code)]
    pub fn datagrams_out(&self) -> &VecDeque<InternetDatagram> {
        &self.datagrams_out
    }

    #[allow(dead_code)]
    pub fn datagrams_out_mut(&mut self) -> &mut VecDeque<InternetDatagram> {
        &mut self.datagrams_out
    }
}

#[derive(Debug)]
pub struct Router {
    intfs: Vec<AsyncNetworkInterface>,
    // <prefix_length, <route_prefix, (Option<next_hop>, interface_num)>)
    route_map: BTreeMap<u8, HashMap<u32, (Option<Ipv4Addr>, SizeT)>>,
}
impl Router {
    #[allow(dead_code)]
    pub fn new() -> Router {
        Router {
            intfs: Default::default(),
            route_map: Default::default(),
        }
    }

    #[allow(dead_code)]
    pub fn add_route(
        &mut self,
        prefix: u32,
        prefix_len: u8,
        next_hop: Option<Ipv4Addr>,
        interface_num: SizeT,
    ) {
        eprintln!(
            "DEBUG: adding route {}/{} => {} on interface {}",
            Ipv4Addr::from(prefix).to_string(),
            prefix_len,
            if next_hop.is_some() {
                next_hop.unwrap().to_string()
            } else {
                "(direct)".to_string()
            },
            interface_num
        );

        if self.route_map.contains_key(&prefix_len) {
            self.route_map
                .get_mut(&prefix_len)
                .unwrap()
                .insert(prefix, (next_hop, interface_num));
        } else {
            let mut route_map: HashMap<u32, (Option<Ipv4Addr>, SizeT)> = HashMap::new();
            route_map.insert(prefix, (next_hop, interface_num));
            self.route_map.insert(prefix_len, route_map);
        }
    }

    #[allow(dead_code)]
    fn route_one_datagram(&mut self, mut dgram: InternetDatagram) {
        if dgram.header().ttl <= 1 {
            return;
        }
        dgram.header_mut().ttl = dgram.header_mut().ttl - 1;

        let dst = dgram.header().dst;
        let found = self.route_map.iter().rev().find_map(|it| {
            let prefix_length = it.0;
            let mappings = it.1;

            let shift_length = 32 - prefix_length;

            for it_0 in mappings.iter() {
                let route_prefix = it_0.0.clone();
                let (next_hop, interface_num) = it_0.1;

                let r: u32 = (((dst as u64) ^ (route_prefix as u64)) >> shift_length) as u32;
                if r != 0 {
                    continue;
                }

                return Some((interface_num.clone(), next_hop.clone()));
            }

            None
        });
        match found {
            Some((interface_num, next_hop)) => {
                let next = next_hop.unwrap_or(Ipv4Addr::from(dst));
                self.interface_mut(interface_num)
                    .send_datagram(dgram, &next);
            }
            None => {}
        }
    }

    #[allow(dead_code)]
    pub fn route(&mut self) {
        let mut t: VecDeque<InternetDatagram> = VecDeque::new();
        for if_ in self.intfs.iter_mut() {
            let queue = if_.datagrams_out_mut();
            t.append(queue);
            assert!(queue.is_empty());
        }

        while !t.is_empty() {
            self.route_one_datagram(t.pop_front().unwrap());
        }
    }

    #[allow(dead_code)]
    pub fn add_interface(&mut self, interface_: AsyncNetworkInterface) -> SizeT {
        self.intfs.push(interface_);
        self.intfs.len() - 1
    }

    #[allow(dead_code)]
    pub fn add_interface_at(&mut self, n: SizeT, interface_: AsyncNetworkInterface) {
        self.intfs.insert(n, interface_);
    }

    #[allow(dead_code)]
    pub fn rm_interface(&mut self, n: SizeT) -> AsyncNetworkInterface {
        self.intfs.remove(n)
    }

    #[allow(dead_code)]
    pub fn interface_mut(&mut self, n: SizeT) -> &mut AsyncNetworkInterface {
        self.intfs.get_mut(n).unwrap()
    }
}
