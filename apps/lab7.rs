use rand::{thread_rng, Rng};
use rust_sponge::network_interface::NetworkInterface;
use rust_sponge::router::{AsyncNetworkInterface, Router};
use rust_sponge::tcp_helpers::arp_message::ARPMessage;
use rust_sponge::tcp_helpers::ethernet_frame::EthernetFrame;
use rust_sponge::tcp_helpers::ethernet_header::EthernetHeader;
use rust_sponge::tcp_helpers::fd_adapter::NetworkInterfaceAdapter;
use rust_sponge::tcp_helpers::ipv4_header::IPv4Header;
use rust_sponge::tcp_helpers::tcp_config::{FdAdapterConfig, TCPConfig};
use rust_sponge::tcp_helpers::tcp_segment::TCPSegment;
use rust_sponge::tcp_helpers::tcp_sponge_socket::AsLocalStreamSocketMut;
use rust_sponge::util::aeventloop::AEventLoop;
use rust_sponge::util::eventloop::Direction;
use rust_sponge::util::file_descriptor::AsFileDescriptor;
use rust_sponge::util::parser::ParseResult;
use rust_sponge::util::socket::{LocalStreamSocket, UDPSocket};
use rust_sponge::util::util::random_router_ethernet_address;
use rust_sponge::{InternetDatagram, NetworkInterfaceSpongeSocket, SizeT};
use std::net::{Ipv4Addr, SocketAddrV4, ToSocketAddrs};
use std::process::exit;
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::{Arc, Mutex};
use std::{env, thread};

mod bidirectional_stream_copy;
use crate::bidirectional_stream_copy::bidirectional_stream_copy_sponge;

// TCPSocketLab7 rt_timeout

// test: 1
// ./target/debug/lab7 server cs144.keithw.org 3000
// ./target/debug/lab7 client cs144.keithw.org 3001

// test: 2
// dd if=/dev/urandom bs=1M count=1 of=/tmp/big.txt
// ./target/debug/lab7 server cs144.keithw.org 3000 < /tmp/big.txt
// </dev/null ./target/debug/lab7 client cs144.keithw.org 3001 > /tmp/big-received.txt

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() <= 0 {
        exit(1);
    }

    if args.len() != 4 && args.len() != 5 {
        print_usage(&args[0]);
        exit(1);
    }

    if args[1] != "client" && args[1] != "server" {
        print_usage(&args[0]);
        exit(1);
    }

    // must do the resolve
    let server_details = args[2].to_owned() + ":" + &*args[3].to_string();
    let server: Vec<_> = server_details
        .to_socket_addrs()
        .expect("Unable to resolve domain")
        .collect();
    eprintln!("resolved: {:?}", server);

    program_body(
        args[1] == "client",
        server[0].ip().to_string().as_str(),
        server[0].port(),
        args.len() == 5,
    );
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
                if dgram.header().proto == IPv4Header::PROTO_TCP {
                    let r = TCPSegment::parse_new(
                        dgram.payload_mut().clone(),
                        dgram.header().pseudo_cksum(),
                    );
                    match r {
                        Ok(tcp_seg) => {
                            ret.push_str(" ");
                            ret.push_str(tcp_seg.header().summary().as_str());
                        }
                        _ => {}
                    }
                }
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

fn program_body(is_client: bool, bounce_host: &str, bounce_port: u16, debug: bool) {
    let mut internet_socket = UDPSocket::new();
    let bounce_address = SocketAddrV4::new(Ipv4Addr::from_str(bounce_host).unwrap(), bounce_port);
    internet_socket.sendto(&bounce_address, &mut b"".to_vec());
    internet_socket.sendto(&bounce_address, &mut b"".to_vec());
    internet_socket.sendto(&bounce_address, &mut b"".to_vec());

    let internet_socket_rc = Arc::new(Mutex::new(internet_socket.as_file_descriptor().clone()));

    let mut router = Router::new();
    let host_side: SizeT;
    let internet_side: SizeT;
    if is_client {
        host_side = router.add_interface(AsyncNetworkInterface::new(NetworkInterface::new(
            random_router_ethernet_address(),
            Ipv4Addr::from_str("192.168.0.1").unwrap(),
        )));
        internet_side = router.add_interface(AsyncNetworkInterface::new(NetworkInterface::new(
            random_router_ethernet_address(),
            Ipv4Addr::from_str("10.0.0.192").unwrap(),
        )));
        router.add_route(
            u32::from(Ipv4Addr::from_str("192.168.0.0").unwrap()),
            16,
            None,
            host_side,
        );
        router.add_route(
            u32::from(Ipv4Addr::from_str("10.0.0.0").unwrap()),
            8,
            None,
            internet_side,
        );
        router.add_route(
            u32::from(Ipv4Addr::from_str("172.16.0.0").unwrap()),
            12,
            Some(Ipv4Addr::from_str("10.0.0.172").unwrap()),
            internet_side,
        );
    } else {
        host_side = router.add_interface(AsyncNetworkInterface::new(NetworkInterface::new(
            random_router_ethernet_address(),
            Ipv4Addr::from_str("172.16.0.1").unwrap(),
        )));
        internet_side = router.add_interface(AsyncNetworkInterface::new(NetworkInterface::new(
            random_router_ethernet_address(),
            Ipv4Addr::from_str("10.0.0.172").unwrap(),
        )));
        router.add_route(
            u32::from(Ipv4Addr::from_str("172.16.0.0").unwrap()),
            12,
            None,
            host_side,
        );
        router.add_route(
            u32::from(Ipv4Addr::from_str("10.0.0.0").unwrap()),
            8,
            None,
            internet_side,
        );
        router.add_route(
            u32::from(Ipv4Addr::from_str("192.168.0.0").unwrap()),
            16,
            Some(Ipv4Addr::from_str("10.0.0.192").unwrap()),
            internet_side,
        );
    }

    let mut sock = if is_client {
        TCPSocketLab7::new(
            SocketAddrV4::new(Ipv4Addr::from_str("192.168.0.50").unwrap(), 0),
            Ipv4Addr::from_str("192.168.0.1").unwrap(),
        )
    } else {
        TCPSocketLab7::new(
            SocketAddrV4::new(Ipv4Addr::from_str("172.16.0.100").unwrap(), 0),
            Ipv4Addr::from_str("172.16.0.1").unwrap(),
        )
    };

    let exit_flag = Arc::new(AtomicBool::new(false));
    let exit_flag_ = exit_flag.clone();

    let router_rc = Arc::new(Mutex::new(router));

    let frame_fd_rc = Arc::new(Mutex::new(
        sock.adapter().lock().unwrap().frame_fd().clone(),
    ));

    let network_thread = thread::Builder::new()
        .name("thread1".to_string())
        .spawn(Box::new(move || {
            let mut event_loop_ = AEventLoop::new();

            // rule 1: frames from host to router
            let fd_ = frame_fd_rc.clone();
            let fd_1 = frame_fd_rc.clone();
            let router_rc_ = router_rc.clone();
            event_loop_.add_rule(
                fd_,
                Direction::In,
                Box::new(move || {
                    let mut frame = EthernetFrame::new();
                    let r = frame.parse(fd_1.lock().unwrap().read(u32::MAX));
                    if r != ParseResult::NoError {
                        return;
                    }
                    if debug {
                        eprintln!("     Host->router:     {}", summary(&frame));
                    }

                    let mut _router = router_rc_.lock().unwrap();
                    _router.interface_mut(host_side).recv_frame(&frame);
                    _router.route();
                }),
                Box::new(|| true),
                Box::new(|| {}),
            );

            // rule 2: frames from router to host
            let fd_ = frame_fd_rc.clone();
            let fd_1 = frame_fd_rc.clone();
            let router_rc_ = router_rc.clone();
            let router_rc_1_ = router_rc.clone();
            event_loop_.add_rule(
                fd_,
                Direction::Out,
                Box::new(move || {
                    let mut _router = router_rc_.lock().unwrap();
                    let f = _router.interface_mut(host_side).frames_out_mut();
                    if f.is_empty() {
                        return;
                    }
                    if debug {
                        eprintln!("     Router->host:     {}", summary(f.front().unwrap()));
                    }
                    fd_1.lock()
                        .unwrap()
                        .write(f.pop_front().unwrap().serialize().as_slice(), true);
                }),
                Box::new(move || {
                    let mut _router = router_rc_1_.lock().unwrap();
                    !_router.interface_mut(host_side).frames_out().is_empty()
                }),
                Box::new(|| {}),
            );

            // rule 3: frames from router to internet
            let fd_ = internet_socket_rc.clone();
            let router_rc_ = router_rc.clone();
            let router_rc_1_ = router_rc.clone();
            event_loop_.add_rule(
                fd_,
                Direction::Out,
                Box::new(move || {
                    let mut _router = router_rc_.lock().unwrap();
                    let f = _router.interface_mut(internet_side).frames_out_mut();
                    if f.is_empty() {
                        return;
                    }
                    if debug {
                        eprintln!("     Router->Internet:     {}", summary(f.front().unwrap()));
                    }
                    internet_socket
                        .sendto(&bounce_address, &mut f.pop_front().unwrap().serialize());
                }),
                Box::new(move || {
                    let mut _router = router_rc_1_.lock().unwrap();
                    !_router.interface_mut(internet_side).frames_out().is_empty()
                }),
                Box::new(|| {}),
            );

            // rule 4: frames from internet to router
            let fd_ = internet_socket_rc.clone();
            let internet_socket_rc_ = internet_socket_rc.clone();
            let router_rc_ = router_rc.clone();
            event_loop_.add_rule(
                fd_,
                Direction::In,
                Box::new(move || {
                    let mut frame = EthernetFrame::new();
                    let mut internet_socket_rc_1 = internet_socket_rc_.lock().unwrap();
                    let r = frame.parse(internet_socket_rc_1.read(u32::MAX));
                    if r != ParseResult::NoError {
                        return;
                    }
                    if debug {
                        eprintln!("     Internet->router:     {}", summary(&frame));
                    }
                    let mut _router = router_rc_.lock().unwrap();
                    _router.interface_mut(internet_side).recv_frame(&frame);
                    _router.route();
                }),
                Box::new(|| true),
                Box::new(|| {}),
            );

            loop {
                if event_loop_.wait_next_event(50) == rust_sponge::util::eventloop::Result::Exit {
                    eprintln!("Exiting... ");
                    return;
                }

                let mut _router = router_rc.lock().unwrap();
                _router.interface_mut(host_side).tick(50);
                _router.interface_mut(internet_side).tick(50);
                if exit_flag_.load(SeqCst) {
                    return;
                }
            }
        }))
        .unwrap();

    if is_client {
        sock.connect("172.16.0.100", 1234);
    } else {
        sock.bind("172.16.0.100", 1234);
        sock.listen_and_accept();
    }
    bidirectional_stream_copy_sponge(&mut sock);
    sock.wait_until_closed();

    eprintln!("Exiting... ");
    exit_flag.store(true, SeqCst);
    network_thread.join().expect("waiting network_thread");
    eprintln!("done.");
}

fn print_usage(argv0: &str) {
    eprintln!("Usage: {} client HOST PORT [debug]", argv0);
    eprintln!("or: {} server HOST PORT [debug]", argv0);
}

#[derive(Debug)]
pub struct TCPSocketLab7 {
    sock: NetworkInterfaceSpongeSocket,
    local_address: SocketAddrV4,
}
impl TCPSocketLab7 {
    #[allow(dead_code)]
    pub fn new(ip_address: SocketAddrV4, next_hop: Ipv4Addr) -> TCPSocketLab7 {
        TCPSocketLab7 {
            sock: NetworkInterfaceSpongeSocket::new(NetworkInterfaceAdapter::new(
                ip_address.ip().clone(),
                next_hop,
            )),
            local_address: ip_address,
        }
    }

    #[allow(dead_code)]
    pub fn connect(&mut self, _host: &str, _port: u16) {
        let s_port: u16 = thread_rng().gen_range(20000..30000);

        self.local_address = SocketAddrV4::new(self.local_address.ip().clone(), s_port);
        eprintln!(
            "DEBUG: Connecting from {}...",
            self.local_address.to_string()
        );

        let multiplexer_config = FdAdapterConfig {
            source: self.local_address.clone(),
            destination: SocketAddrV4::new(Ipv4Addr::from_str(_host).unwrap(), _port),
            loss_rate_dn: 0,
            loss_rate_up: 0,
        };
        let mut config = TCPConfig::default();
        // config.rt_timeout = 30;

        self.sock.connect(&config, multiplexer_config);
    }

    #[allow(dead_code)]
    pub fn bind(&mut self, _host: &str, _port: u16) {
        assert_eq!(
            _host,
            self.local_address.ip().to_string(),
            "Cannot bind to {}:{}",
            _host,
            _port
        );
        self.local_address = SocketAddrV4::new(self.local_address.ip().clone(), _port);
    }

    #[allow(dead_code)]
    pub fn listen_and_accept(&mut self) {
        let multiplexer_config = FdAdapterConfig {
            source: self.local_address.clone(),
            destination: SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0),
            loss_rate_dn: 0,
            loss_rate_up: 0,
        };
        let mut config = TCPConfig::default();
        // config.rt_timeout = 30;

        self.sock.listen_and_accept(&config, multiplexer_config);
    }

    #[allow(dead_code)]
    pub fn wait_until_closed(&mut self) {
        self.sock.wait_until_closed();
    }

    pub fn adapter(&self) -> Arc<Mutex<NetworkInterfaceAdapter>> {
        self.sock.datagram_adapter.clone()
    }
}
impl AsLocalStreamSocketMut for TCPSocketLab7 {
    fn as_socket_mut(&mut self) -> Arc<Mutex<LocalStreamSocket>> {
        self.sock.main_thread_data.clone()
    }
}
