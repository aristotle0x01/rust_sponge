use rand::{thread_rng, Rng};
use rust_sponge::network_interface::NetworkInterface;
use rust_sponge::router::{AsyncNetworkInterface, Router};
use rust_sponge::tcp_helpers::arp_message::ARPMessage;
use rust_sponge::tcp_helpers::ethernet_frame::EthernetFrame;
use rust_sponge::tcp_helpers::ethernet_header::{EthernetAddress, EthernetHeader};
use rust_sponge::tcp_helpers::ipv4_header::IPv4Header;
use rust_sponge::util::buffer::Buffer;
use rust_sponge::util::parser::ParseResult;
use rust_sponge::{InternetDatagram, SizeT};
use std::collections::{HashMap, VecDeque};
use std::net::Ipv4Addr;
use std::str::FromStr;
use rust_sponge::util::util::InternetChecksum;

fn main() {
    network_simulator();
}

fn random_host_ethernet_address() -> EthernetAddress {
    let mut addr = EthernetAddress::default();
    for b in addr.iter_mut() {
        *b = (thread_rng().gen_range(0..u32::MAX) % 256) as u8;
    }
    addr[0] |= 0x02u8;
    addr[0] &= 0xfeu8;

    addr
}

fn random_router_ethernet_address() -> EthernetAddress {
    let mut addr = EthernetAddress::default();
    for b in addr.iter_mut() {
        *b = (thread_rng().gen_range(0..u32::MAX) % 256) as u8;
    }
    addr[0] = 0x02u8;
    addr[1] = 0;
    addr[2] = 0;

    addr
}

fn summary(frame: &EthernetFrame) -> String {
    let mut ret = String::new();
    ret.push_str(frame.header().to_string().as_str());

    match frame.header().pro_type {
        EthernetHeader::TYPE_IPV4 => {
            let mut dgram = InternetDatagram::new(IPv4Header::new(), frame.payload().clone());
            let result = dgram.parse(0);
            if result == ParseResult::NoError {
                ret.push_str(" ");
                ret.push_str(dgram.header().summary().as_str());
                ret.push_str(" payload=\"");
                ret.push_str(String::from_utf8_lossy(dgram.payload().str()).as_ref());
                ret.push_str("\"");
            } else {
                ret.push_str(" (bad IPv4)");
            }
        }
        EthernetHeader::TYPE_ARP => {
            let mut arp = ARPMessage::new();
            let result = arp.parse(frame.payload().str().to_vec());
            if result == ParseResult::NoError {
                ret.push_str(" ");
                ret.push_str(arp.to_string().as_str());
            } else {
                ret.push_str(" (bad ARP)");
            }
        }
        _ => {}
    }

    ret
}

#[derive(Debug)]
pub struct Host {
    name: String,
    my_address: Ipv4Addr,
    interface: AsyncNetworkInterface,
    next_hop: Ipv4Addr,
    expecting_to_receive: Vec<InternetDatagram>,
}
impl Host {
    #[allow(dead_code)]
    pub fn new(name_: String, my_addr: Ipv4Addr, next: Ipv4Addr) -> Host {
        Host {
            name: name_,
            my_address: my_addr.clone(),
            interface: AsyncNetworkInterface::new(NetworkInterface::new(
                random_host_ethernet_address(),
                my_addr,
            )),
            next_hop: next,
            expecting_to_receive: vec![],
        }
    }

    fn expecting(&self, expected: &InternetDatagram) -> bool {
        for x in self.expecting_to_receive.iter() {
            if x.serialize() == expected.serialize() {
                return true;
            }
        }

        return false;
    }

    fn remove_expectation(&mut self, expected: &InternetDatagram) {
        for i_ in 0..self.expecting_to_receive.len() {
            if self.expecting_to_receive[i_].serialize() == expected.serialize() {
                self.expecting_to_receive.remove(i_);
                return;
            }
        }
    }

    #[allow(dead_code)]
    pub fn send_to(&mut self, destination: &Ipv4Addr, ttl: u8) -> InternetDatagram {
        let payload = Buffer::from(format!(
            "random payload: {}{}{}",
            "{",
            thread_rng().gen_range(0..u32::MAX),
            "}"
        ));
        let mut header = IPv4Header::new();
        header.src = u32::from(self.my_address);
        header.dst = u32::from(destination.clone());
        header.len = ((header.hlen * 4) as SizeT + payload.size()) as u16;
        header.ttl = ttl;
        let dgram = InternetDatagram::new(
            header,
            payload,
        );
        self.interface.send_datagram(dgram.clone(), &self.next_hop);

        // awkward way to calc cksum here
        let mut rd = dgram.serialize();
        let mut rgram = InternetDatagram::new(IPv4Header::new(), Buffer::new(rd));
        assert_eq!(rgram.parse(0), ParseResult::NoError);
        eprintln!(
            "Host {} trying to send datagram (with next hop = {}): {} payload=\"{}\"",
            self.name,
            self.next_hop.to_string(),
            rgram.header().summary(),
            String::from_utf8_lossy(rgram.payload.str())
        );
        return rgram;
    }

    #[allow(dead_code)]
    pub fn address(&self) -> &Ipv4Addr {
        &self.my_address
    }

    #[allow(dead_code)]
    pub fn interface_mut(&mut self) -> &mut AsyncNetworkInterface {
        &mut self.interface
    }

    #[allow(dead_code)]
    pub fn expect(&mut self, expected: InternetDatagram) {
        self.expecting_to_receive.push(expected);
    }

    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[allow(dead_code)]
    pub fn check(&mut self) {
        while !self.interface.datagrams_out().is_empty() {
            let dgram_received = self.interface.datagrams_out_mut().pop_front().unwrap();
            assert!(
                self.expecting(&dgram_received),
                "{}",
                format!(
                    "Host {} received unexpected Internet datagram: {} payload=\"{}\"",
                    self.name,
                    dgram_received.header().summary(),
                    String::from_utf8_lossy(dgram_received.payload.str())
                )
            );
            self.remove_expectation(&dgram_received);
        }

        if !self.expecting_to_receive.is_empty() {
            let expected = self.expecting_to_receive.first().unwrap();
            assert!(
                false,
                "{}",
                format!(
                    "Host {}  did NOT receive an expected Internet datagram: {} payload=\"{}\"",
                    self.name,
                    expected.header().summary(),
                    String::from_utf8_lossy(expected.payload.str())
                )
            );
        }
    }
}

#[derive(Debug)]
pub struct Network {
    router: Router,
    default_id: SizeT,
    eth0_id: SizeT,
    eth1_id: SizeT,
    eth2_id: SizeT,
    uun3_id: SizeT,
    hs4_id: SizeT,
    mit5_id: SizeT,
    hosts: HashMap<String, Host>,
}
impl Network {
    #[allow(dead_code)]
    pub fn new() -> Network {
        let mut net = Network {
            router: Router::new(),
            default_id: 0,
            eth0_id: 0,
            eth1_id: 0,
            eth2_id: 0,
            uun3_id: 0,
            hs4_id: 0,
            mit5_id: 0,
            hosts: Default::default(),
        };
        net.default_id =
            net.router
                .add_interface(AsyncNetworkInterface::new(NetworkInterface::new(
                    random_router_ethernet_address(),
                    Ipv4Addr::from_str("171.67.76.46").unwrap(),
                )));
        net.eth0_id = net
            .router
            .add_interface(AsyncNetworkInterface::new(NetworkInterface::new(
                random_router_ethernet_address(),
                Ipv4Addr::from_str("10.0.0.1").unwrap(),
            )));
        net.eth1_id = net
            .router
            .add_interface(AsyncNetworkInterface::new(NetworkInterface::new(
                random_router_ethernet_address(),
                Ipv4Addr::from_str("172.16.0.1").unwrap(),
            )));
        net.eth2_id = net
            .router
            .add_interface(AsyncNetworkInterface::new(NetworkInterface::new(
                random_router_ethernet_address(),
                Ipv4Addr::from_str("192.168.0.1").unwrap(),
            )));
        net.uun3_id = net
            .router
            .add_interface(AsyncNetworkInterface::new(NetworkInterface::new(
                random_router_ethernet_address(),
                Ipv4Addr::from_str("198.178.229.1").unwrap(),
            )));
        net.hs4_id = net
            .router
            .add_interface(AsyncNetworkInterface::new(NetworkInterface::new(
                random_router_ethernet_address(),
                Ipv4Addr::from_str("143.195.0.2").unwrap(),
            )));
        net.mit5_id = net
            .router
            .add_interface(AsyncNetworkInterface::new(NetworkInterface::new(
                random_router_ethernet_address(),
                Ipv4Addr::from_str("128.30.76.255").unwrap(),
            )));

        net.hosts.insert(
            "applesauce".to_string(),
            Host::new(
                "applesauce".to_string(),
                Ipv4Addr::from_str("10.0.0.2").unwrap(),
                Ipv4Addr::from_str("10.0.0.1").unwrap(),
            ),
        );
        net.hosts.insert(
            "default_router".to_string(),
            Host::new(
                "default_router".to_string(),
                Ipv4Addr::from_str("171.67.76.1").unwrap(),
                Ipv4Addr::from_str("0.0.0.0").unwrap(),
            ),
        );
        net.hosts.insert(
            "cherrypie".to_string(),
            Host::new(
                "cherrypie".to_string(),
                Ipv4Addr::from_str("192.168.0.2").unwrap(),
                Ipv4Addr::from_str("192.168.0.1").unwrap(),
            ),
        );
        net.hosts.insert(
            "hs_router".to_string(),
            Host::new(
                "hs_router".to_string(),
                Ipv4Addr::from_str("143.195.0.1").unwrap(),
                Ipv4Addr::from_str("0.0.0.0").unwrap(),
            ),
        );
        net.hosts.insert(
            "dm42".to_string(),
            Host::new(
                "dm42".to_string(),
                Ipv4Addr::from_str("198.178.229.42").unwrap(),
                Ipv4Addr::from_str("198.178.229.1").unwrap(),
            ),
        );
        net.hosts.insert(
            "dm43".to_string(),
            Host::new(
                "dm43".to_string(),
                Ipv4Addr::from_str("198.178.229.43").unwrap(),
                Ipv4Addr::from_str("198.178.229.1").unwrap(),
            ),
        );

        net.router.add_route(
            u32::from(Ipv4Addr::from_str("0.0.0.0").unwrap()),
            0,
            Some(net.hosts["default_router"].address().clone()),
            net.default_id,
        );
        net.router.add_route(
            u32::from(Ipv4Addr::from_str("10.0.0.0").unwrap()),
            8,
            None,
            net.eth0_id,
        );
        net.router.add_route(
            u32::from(Ipv4Addr::from_str("172.16.0.0").unwrap()),
            16,
            None,
            net.eth1_id,
        );
        net.router.add_route(
            u32::from(Ipv4Addr::from_str("192.168.0.0").unwrap()),
            24,
            None,
            net.eth2_id,
        );
        net.router.add_route(
            u32::from(Ipv4Addr::from_str("198.178.229.0").unwrap()),
            24,
            None,
            net.uun3_id,
        );
        net.router.add_route(
            u32::from(Ipv4Addr::from_str("143.195.0.0").unwrap()),
            17,
            Some(net.hosts["hs_router"].address().clone()),
            net.hs4_id,
        );
        net.router.add_route(
            u32::from(Ipv4Addr::from_str("143.195.128.0").unwrap()),
            18,
            Some(net.hosts["hs_router"].address().clone()),
            net.hs4_id,
        );
        net.router.add_route(
            u32::from(Ipv4Addr::from_str("143.195.192.0").unwrap()),
            19,
            Some(net.hosts["hs_router"].address().clone()),
            net.hs4_id,
        );
        net.router.add_route(
            u32::from(Ipv4Addr::from_str("128.30.76.255").unwrap()),
            16,
            Some(Ipv4Addr::from_str("128.30.0.1").unwrap()),
            net.mit5_id,
        );

        net
    }

    fn exchange_frames(
        &mut self,
        x_name: &str,
        x: &mut AsyncNetworkInterface,
        y_name: &str,
        y: &mut AsyncNetworkInterface,
    ) {
        let mut x_frames = x.frames_out().clone();
        let mut y_frames = y.frames_out().clone();

        self.deliver(x_name, &x_frames, y_name, y);
        self.deliver(y_name, &y_frames, x_name, x);

        self.clear(&mut x_frames, x.frames_out_mut());
        self.clear(&mut y_frames, y.frames_out_mut());
    }

    fn exchange_frames_xyz(
        &mut self,
        x_name: &str,
        x: &mut AsyncNetworkInterface,
        y_name: &str,
        y: &mut AsyncNetworkInterface,
        z_name: &str,
        z: &mut AsyncNetworkInterface,
    ) {
        let mut x_frames = x.frames_out().clone();
        let mut y_frames = y.frames_out().clone();
        let mut z_frames = z.frames_out().clone();

        self.deliver(x_name, &x_frames, y_name, y);
        self.deliver(x_name, &x_frames, z_name, z);

        self.deliver(y_name, &y_frames, x_name, x);
        self.deliver(y_name, &y_frames, z_name, z);

        self.deliver(z_name, &z_frames, x_name, x);
        self.deliver(z_name, &z_frames, y_name, y);

        self.clear(&mut x_frames, x.frames_out_mut());
        self.clear(&mut y_frames, y.frames_out_mut());
        self.clear(&mut z_frames, z.frames_out_mut());
    }

    fn clear(&self, x: &mut VecDeque<EthernetFrame>, y: &mut VecDeque<EthernetFrame>) {
        while !x.is_empty() {
            x.pop_front();
            y.pop_front();
        }
    }

    fn deliver(
        &self,
        src_name: &str,
        src: &VecDeque<EthernetFrame>,
        dst_name: &str,
        dst: &mut AsyncNetworkInterface,
    ) {
        for i_ in 0..src.len() {
            eprintln!(
                "Transferring frame from {} to {}: {}",
                src_name,
                dst_name,
                summary(&src[i_])
            );
            dst.recv_frame(&src[i_]);
        }
    }

    #[allow(dead_code)]
    pub fn simulate_physical_connections(&mut self) {
        {
            let mut if1_ = self.router.rm_interface(self.default_id);
            let mut host_ = self.rm_host("default_router");
            self.exchange_frames(
                "router.default",
                &mut if1_,
                "default_router",
                host_.interface_mut(),
            );
            self.router.add_interface_at(self.default_id, if1_);
            self.add_host("default_router", host_);
        }
        {
            let mut if1_ = self.router.rm_interface(self.eth0_id);
            let mut host_ = self.rm_host("applesauce");
            self.exchange_frames(
                "router.eth0",
                &mut if1_,
                "applesauce",
                host_.interface_mut(),
            );
            self.router.add_interface_at(self.eth0_id, if1_);
            self.add_host("applesauce", host_);
        }
        {
            let mut if1_ = self.router.rm_interface(self.eth2_id);
            let mut host_ = self.rm_host("cherrypie");
            self.exchange_frames(
                "router.eth2",
                &mut if1_,
                "cherrypie",
                host_.interface_mut(),
            );
            self.router.add_interface_at(self.eth2_id, if1_);
            self.add_host("cherrypie", host_);
        }
        {
            let mut if1_ = self.router.rm_interface(self.hs4_id);
            let mut host_ = self.rm_host("hs_router");
            self.exchange_frames(
                "router.hs4",
                &mut if1_,
                "hs_router",
                host_.interface_mut(),
            );
            self.router.add_interface_at(self.hs4_id, if1_);
            self.add_host("hs_router", host_);
        }
        {
            let mut if1_ = self.router.rm_interface(self.uun3_id);
            let mut host_ = self.rm_host("dm42");
            let mut host2_ = self.rm_host("dm43");
            self.exchange_frames_xyz(
                "router.uun3",
                &mut if1_,
                "dm42",
                host_.interface_mut(),
                "dm43",
                host2_.interface_mut(),
            );
            self.router.add_interface_at(self.uun3_id, if1_);
            self.add_host("dm42", host_);
            self.add_host("dm43", host2_);
        }
    }

    #[allow(dead_code)]
    pub fn simulate(&mut self) {
        for i_ in 0..256 {
            self.router.route();
            self.simulate_physical_connections();
        }

        for (f, s) in self.hosts.iter_mut() {
            s.check();
        }
    }

    #[allow(dead_code)]
    pub fn host(&mut self, name: &str) -> &mut Host {
        assert!(self.hosts.contains_key(name), "unknown host: {}", name);
        let shost = self.hosts.get_mut(name).unwrap();
        assert_eq!(shost.name(), name, "invalid host: {}", name);

        shost
    }

    #[allow(dead_code)]
    pub fn rm_host(&mut self, name: &str) -> Host {
        assert!(self.hosts.contains_key(name), "unknown host: {}", name);
        let shost = self.hosts.remove(name).unwrap();
        assert_eq!(shost.name(), name, "invalid host: {}", name);

        shost
    }

    #[allow(dead_code)]
    pub fn add_host(&mut self, name: &str, h: Host) {
        assert!(!self.hosts.contains_key(name), "host: {} exists", name);
        self.hosts.insert(name.to_string(), h);
    }
}

fn network_simulator() {
    // https://stackoverflow.com/questions/69981449/how-do-i-print-colored-text-to-the-terminal-in-rust
    let green = "\x1b[32;1m";
    let normal = "\x1b[m";

    eprintln!("{}Constructing network.{}", green, normal);

    let mut network = Network::new();
    eprintln!(
        "{}Testing traffic between two ordinary hosts (applesauce to cherrypie)...{}",
        green, normal
    );
    {
        let d = network.host("cherrypie").address().clone();
        let mut dgram_sent = network.host("applesauce").send_to(&d, 64);
        dgram_sent.header_mut().ttl -= 1;
        network.host("cherrypie").expect(dgram_sent);
        network.simulate();
    }

    eprintln!(
        "{}Testing traffic between two ordinary hosts (cherrypie to applesauce)...{}",
        green, normal
    );
    {
        let d = network.host("applesauce").address().clone();
        let mut dgram_sent = network.host("cherrypie").send_to(&d, 64);
        dgram_sent.header_mut().ttl -= 1;
        network.host("applesauce").expect(dgram_sent);
        network.simulate();
    }

    eprintln!(
        "{}Success! Testing applesauce sending to the Internet.{}",
        green, normal
    );
    {
        let mut dgram_sent = network
            .host("applesauce")
            .send_to(&Ipv4Addr::from_str("1.2.3.4").unwrap(), 64);
        dgram_sent.header_mut().ttl -= 1;
        network.host("default_router").expect(dgram_sent);
        network.simulate();
    }

    eprintln!(
        "{}Success! Testing sending to the HS network and Internet.{}",
        green, normal
    );
    {
        let mut dgram_sent = network
            .host("applesauce")
            .send_to(&Ipv4Addr::from_str("143.195.131.17").unwrap(), 64);
        dgram_sent.header_mut().ttl -= 1;
        network.host("hs_router").expect(dgram_sent);
        network.simulate();

        dgram_sent = network
            .host("cherrypie")
            .send_to(&Ipv4Addr::from_str("143.195.193.52").unwrap(), 64);
        dgram_sent.header_mut().ttl -= 1;
        network.host("hs_router").expect(dgram_sent);
        network.simulate();

        dgram_sent = network
            .host("cherrypie")
            .send_to(&Ipv4Addr::from_str("143.195.223.255").unwrap(), 64);
        dgram_sent.header_mut().ttl -= 1;
        network.host("hs_router").expect(dgram_sent);
        network.simulate();

        dgram_sent = network
            .host("cherrypie")
            .send_to(&Ipv4Addr::from_str("143.195.224.0").unwrap(), 64);
        dgram_sent.header_mut().ttl -= 1;
        network.host("default_router").expect(dgram_sent);
        network.simulate();
    }

    eprintln!(
        "{}Success! Testing two hosts on the same network (dm42 to dm43)...{}",
        green, normal
    );
    {
        let d = &network.host("dm43").address().clone();
        let mut dgram_sent = network.host("dm42").send_to(d, 64);
        dgram_sent.header_mut().ttl -= 1;
        network.host("dm43").expect(dgram_sent);
        network.simulate();
    }

    eprintln!("{}Success! Testing TTL expiration...{}", green, normal);
    {
        let mut dgram_sent = network
            .host("applesauce")
            .send_to(&Ipv4Addr::from_str("1.2.3.4").unwrap(), 1);
        network.simulate();

        dgram_sent = network
            .host("applesauce")
            .send_to(&Ipv4Addr::from_str("1.2.3.4").unwrap(), 0);
        network.simulate();
    }

    eprintln!("\n\n\033[32;1mCongratulations! All datagrams were routed successfully.\033[m");
}
