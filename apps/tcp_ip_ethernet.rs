use rand::{thread_rng, Rng};
use rust_sponge::tcp_helpers::ethernet_header::EthernetAddress;
use rust_sponge::tcp_helpers::tcp_config::{FdAdapterConfig, TCPConfig};
use rust_sponge::tcp_helpers::tcp_sponge_socket::TCPSpongeSocket;
use rust_sponge::tcp_helpers::tuntap_adapter::TCPOverIPv4OverEthernetAdapter;
use rust_sponge::util::tun::TapFD;
use std::env;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::process::exit;
use std::str::FromStr;

mod bidirectional_stream_copy;
use crate::bidirectional_stream_copy::bidirectional_stream_copy_sponge;

pub const TAP_DFLT: &'static str = "tap10\0";
pub const LOCAL_ADDRESS_DFLT: &'static str = "169.254.10.9";
pub const GATEWAY_DFLT: &'static str = "169.254.10.1";

fn show_usage(argv0: &str, msg: &str) {
    println!("Usage: {} [options] <host> <port>\n", argv0);
    print!("   Option                                                          Default\n");
    print!("   --                                                              --\n\n");

    print!(
        "   -a <addr>       Set IP source address (client mode only)           {}\n\n",
        LOCAL_ADDRESS_DFLT
    );
    print!("   -s <port>       Set TCP source port (client mode only)              (random)\n\n");
    print!(
        "   -n <addr>       Set IP next-hop address           {}\n\n",
        GATEWAY_DFLT
    );

    print!(
        "   -w <winsz>      Use a window of <winsz> bytes                   {}\n\n",
        TCPConfig::MAX_PAYLOAD_SIZE
    );

    print!(
        "   -t <tmout>      Set rt_timeout to tmout                         {}\n\n",
        TCPConfig::TIMEOUT_DFLT
    );

    print!(
        "   -d <tapdev>     Connect to tap <tapdev>                         {}\n\n",
        TAP_DFLT
    );

    print!("   -h              Show this message.\n\n");

    if !msg.is_empty() {
        print!("{}", msg);
    }
    println!();
}

fn check_argc(argc: i32, argv: &Vec<String>, curr: i32, err: &str) {
    if (curr + 3) >= argc {
        show_usage(argv[0].as_str(), err);
        exit(1);
    }
}

fn get_config(argc: i32, argv: &Vec<String>) -> (TCPConfig, FdAdapterConfig, SocketAddrV4, String, bool) {
    let mut c_fsm = TCPConfig::default();
    let mut c_filt = FdAdapterConfig {
        source: SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0),
        destination: SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0),
        loss_rate_dn: 0,
        loss_rate_up: 0,
    };
    let mut tapdev = String::from(TAP_DFLT);

    let mut curr = 1;

    let mut source_address = String::from(LOCAL_ADDRESS_DFLT);
    let mut source_port = String::from(thread_rng().gen_range(1000..u16::MAX).to_string());
    let mut next_hop_address = String::from(GATEWAY_DFLT);

    let mut listen = false;
    while (argc - curr) > 2 {
        let v = argv.get(curr as usize).unwrap().as_str();
        if v.eq("-l") {
            listen = true;
            curr += 1;
            eprintln!("listen ****");
        } else if v.eq("-a") {
            check_argc(argc, argv, curr, "ERROR: -a requires one argument.");
            source_address = argv[(curr + 1) as usize].to_string();
            curr += 2;
        } else if v.eq("-s") {
            check_argc(argc, argv, curr, "ERROR: -s requires one argument.");
            source_port = argv[(curr + 1) as usize].to_string();
            curr += 2;
        } else if v.eq("-n") {
            check_argc(argc, argv, curr, "ERROR: -n requires one argument.");
            next_hop_address = argv[(curr + 1) as usize].to_string();
            curr += 2;
        } else if v.eq("-w") {
            check_argc(argc, argv, curr, "ERROR: -w requires one argument.");
            c_fsm.recv_capacity = argv[(curr + 1) as usize].as_str().parse().unwrap();
            curr += 2;
        } else if v.eq("-t") {
            check_argc(argc, argv, curr, "ERROR: -t requires one argument.");
            c_fsm.rt_timeout = argv[(curr + 1) as usize].as_str().parse().unwrap();
            curr += 2;
        } else if v.eq("-d") {
            check_argc(argc, argv, curr, "ERROR: -d requires one argument.");
            tapdev = argv[(curr + 1) as usize].to_string() + "\0";
            curr += 2;
        } else if v.eq("-h") {
            show_usage(argv[0].as_str(), "");
            exit(0);
        } else {
            show_usage(
                argv[0].as_str(),
                format!("ERROR: unrecognized option {}", argv[curr as usize]).as_str(),
            );
            exit(1);
        }
    }

    if listen {
        c_filt
            .source
            .set_ip(Ipv4Addr::from_str(source_address.as_str()).unwrap());
        c_filt
            .source
            .set_port(argv[(curr + 1) as usize].as_str().parse().unwrap());
        assert_ne!(
            c_filt.source.port(),
            0,
            "ERROR: listen port cannot be zero in server mode."
        );
    } else {
        c_filt
            .destination
            .set_ip(Ipv4Addr::from_str(argv[curr as usize].as_str()).unwrap());
        c_filt
            .destination
            .set_port(argv[(curr + 1) as usize].parse().unwrap());
        c_filt
            .source
            .set_ip(Ipv4Addr::from_str(source_address.as_str()).unwrap());
        c_filt.source.set_port(source_port.parse().unwrap());
    }

    let next_hop = SocketAddrV4::new(Ipv4Addr::from_str(next_hop_address.as_str()).unwrap(), 0);

    (c_fsm, c_filt, next_hop, tapdev, listen)
}

// tshark -f "src 169.254.10.3 or dst 169.254.10.3" -i tap10
// 1) test 1
// ./target/debug/tcp_native -l 169.254.10.1 19121
// ./target/debug/tcp_ip_ethernet  -t 12  -d tap10 -a 169.254.10.3   169.254.10.1  19121
// 2) test 2
// ./target/debug/tcp_ip_ethernet  -t 12  -d tap11 -a 169.254.11.3  -l 169.254.11.3  19121
// ./target/debug/tcp_ip_ethernet  -t 12  -d tap10 -a 169.254.10.3   169.254.11.3  19121
fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() < 3 {
        show_usage(args[0].as_str(), "ERROR: required arguments are missing.");
        exit(1);
    }

    let mut local_ethernet_address = EthernetAddress::default();
    for b in local_ethernet_address.iter_mut() {
        *b = (thread_rng().gen_range(0..u32::MAX) % 256) as u8;
    }
    local_ethernet_address[0] |= 0x02u8;
    local_ethernet_address[0] &= 0xfeu8;

    let (c_fsm, c_filt, next_hop, tap_dev_name, listen) = get_config(args.len() as i32, &args);
    eprintln!(
        "tcp:{}, adapter:dst=>{}, src=>{}, next_hop=>{}, tap=>{}",
        c_fsm.clone().to_string(),
        c_filt.clone().destination.to_string(),
        c_filt.clone().source.to_string(),
        next_hop.to_string(),
        tap_dev_name.as_str()
    );

    let tap_fd = TapFD::new(tap_dev_name.as_str());
    let mut tcp_socket = TCPSpongeSocket::new(TCPOverIPv4OverEthernetAdapter::new(
        tap_fd,
        local_ethernet_address,
        c_filt.source.ip().clone(),
        next_hop.ip().clone(),
    ));
    if listen {
        tcp_socket.listen_and_accept(&c_fsm, c_filt);
    } else {
        tcp_socket.connect(&c_fsm, c_filt);
    }

    bidirectional_stream_copy_sponge(&mut tcp_socket);
    tcp_socket.wait_until_closed();
}
