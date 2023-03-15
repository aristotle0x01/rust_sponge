use log::error;
use rust_sponge::util::socket::{AsSocket, AsSocketMut, TCPSocket};
use std::env;

mod bidirectional_stream_copy;
use crate::bidirectional_stream_copy::bidirectional_stream_copy;

fn show_usage(argv0: &str) {
    error!("Usage: {} [-l] <host> <port>\n\n  -l specifies listen mode; <host>:<port> is the listening address.\n", argv0);
}

// cargo build --example tcp_native
// target/debug/apps/tcp_native "127.0.0.1" "1234"
// target/debug/apps/tcp_native "-l" "127.0.0.1" "1234"
// cargo run --example tcp_native
fn main() {
    let args: Vec<_> = env::args().collect();
    let mut err = false;
    if args.len() < 3 {
        err = true;
    }
    let server_mode = args[1].eq("-l");
    if server_mode && args.len() < 4 {
        err = true;
    }
    if err {
        show_usage(args[0].as_str());
        return;
    }

    let mut socket = if server_mode {
        let mut listening_socket = TCPSocket::new();
        listening_socket.set_reuseaddr();
        listening_socket.bind(args[2].as_str(), args[3].parse::<u16>().unwrap());
        listening_socket.listen(16);
        listening_socket.accept()
    } else {
        let connecting_socket = TCPSocket::new();
        connecting_socket.connect(args[1].as_str(), args[2].parse::<u16>().unwrap());
        connecting_socket
    };

    bidirectional_stream_copy(&mut socket);
}
