use rust_sponge::tcp_helpers::fd_adapter::TCPOverUDPSocketAdapter;
use rust_sponge::tcp_helpers::lossy_fd_adapter::LossyFdAdapter;
use rust_sponge::tcp_helpers::tcp_config::{FdAdapterConfig, TCPConfig};
use rust_sponge::tcp_helpers::tcp_sponge_socket::TCPSpongeSocket;
use rust_sponge::util::socket::{AsSocket, UDPSocket};
use std::env;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::process::exit;
use std::str::FromStr;

mod bidirectional_stream_copy;
use crate::bidirectional_stream_copy::bidirectional_stream_copy_sponge;

pub const DPORT_DFLT: u16 = 1440;

fn show_usage(argv0: &str, msg: &str) {
    println!("Usage: {} [options] <host> <port>\n", argv0);
    print!("   Option                                                          Default\n");
    print!("   --                                                              --\n\n");
    print!("   -l              Server (listen) mode.                           (client mode)\n");
    print!("                   In server mode, <host>:<port> is the address to bind.\n\n");
    print!(
        "   -w <winsz>      Use a window of <winsz> bytes                   {}\n\n",
        TCPConfig::MAX_PAYLOAD_SIZE
    );
    print!(
        "   -t <tmout>      Set rt_timeout to tmout                         {}\n\n",
        TCPConfig::TIMEOUT_DFLT
    );
    print!("   -Lu <loss>      Set uplink loss to <rate> (float in 0..1)       (no loss)\n");
    print!("   -Ld <loss>      Set downlink loss to <rate> (float in 0..1)     (no loss)\n\n");
    print!("   -h              Show this message and quit.\n\n");

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

fn get_config(argc: i32, argv: &Vec<String>) -> (TCPConfig, FdAdapterConfig, bool) {
    let mut c_fsm = TCPConfig::default();
    let mut c_filt = FdAdapterConfig {
        source: SocketAddrV4::from_str("").unwrap(),
        destination: SocketAddrV4::from_str("").unwrap(),
        loss_rate_dn: 0,
        loss_rate_up: 0,
    };

    let mut curr = 1;
    let mut listen = false;

    while (argc - curr) > 2 {
        let v = argv.get(curr as usize).unwrap().as_str();
        if v.eq("-l") {
            listen = true;
            curr += 1;
        } else if v.eq("-w") {
            check_argc(argc, argv, curr, "ERROR: -w requires one argument.");
            c_fsm.recv_capacity = argv[(curr + 1) as usize].as_str().parse().unwrap();
            curr += 2;
        } else if v.eq("-t") {
            check_argc(argc, argv, curr, "ERROR: -t requires one argument.");
            c_fsm.rt_timeout = argv[(curr + 1) as usize].as_str().parse().unwrap();
            curr += 2;
        } else if v.eq("-Lu") {
            check_argc(argc, argv, curr, "ERROR: -Lu requires one argument.");
            let lossrate: f32 = argv[(curr + 1) as usize].as_str().parse().unwrap();
            c_filt.loss_rate_up = (u16::MAX as f32 * lossrate) as u16;
            curr += 2;
        } else if v.eq("-Ld") {
            check_argc(argc, argv, curr, "ERROR: -Ld requires one argument.");
            let lossrate: f32 = argv[(curr + 1) as usize].as_str().parse().unwrap();
            c_filt.loss_rate_dn = (u16::MAX as f32 * lossrate) as u16;
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
            .set_ip(Ipv4Addr::from_str("127.0.0.1").unwrap());
        c_filt
            .source
            .set_port(argv[(argc - 1) as usize].parse().unwrap());
    } else {
        c_filt
            .destination
            .set_ip(Ipv4Addr::from_str(argv[(argc - 2) as usize].as_str()).unwrap());
        c_filt
            .destination
            .set_port(argv[(argc - 1) as usize].parse().unwrap());
    }

    (c_fsm, c_filt, listen)
}

// cargo build --example tcp_udp
// target/debug/apps/tcp_udp -t 12 -w 1452
// cargo run --example tcp_udp
fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() < 3 {
        show_usage(args[0].as_str(), "ERROR: required arguments are missing.");
        exit(1);
    }

    let (c_fsm, c_filt, listen) = get_config(args.len() as i32, &args);

    let udp_sock = UDPSocket::new();
    if listen {
        udp_sock.bind(
            c_filt.source.ip().to_string().as_str(),
            c_filt.source.port(),
        );
    }

    let mut tcp_socket =
        TCPSpongeSocket::new(LossyFdAdapter::new(TCPOverUDPSocketAdapter::new(udp_sock)));
    if listen {
        tcp_socket.listen_and_accept(&c_fsm, c_filt);
    } else {
        tcp_socket.connect(&c_fsm, c_filt);
    }

    bidirectional_stream_copy_sponge(&mut tcp_socket);
    tcp_socket.wait_until_closed();
}
