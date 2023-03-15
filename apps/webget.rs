use rust_sponge::tcp_helpers::tcp_sponge_socket::{AsLocalStreamSocketMut, CS144TCPSocket};
use rust_sponge::util::file_descriptor::{AsFileDescriptor, AsFileDescriptorMut};
use std::env;
use std::net::ToSocketAddrs;
use std::str;

// ./webget cs144.keithw.org /hello

fn main() {
    let args: Vec<String> = env::args().collect();
    let argc = env::args().len();
    if argc <= 0 {
        panic!("without arguments!");
    }

    if argc != 3 {
        println!("Usage: {} HOST PATH\n", args[0]);
        println!("\tExample: {} stanford.edu /class/cs144\n", args[0]);
        panic!("arguments not equall to three!");
    }

    let host = &args[1];
    let url = &args[2];

    get_url(host, url);
}

fn get_url(host: &str, url: &str) {
    println!("host is {}", host);
    println!("url is {}", url);

    // must do the resolve
    let server_details = host.to_owned() + ":80";
    let server: Vec<_> = server_details
        .to_socket_addrs()
        .expect("Unable to resolve domain")
        .collect();
    println!("resolved: {:?}", server);

    let mut socket = CS144TCPSocket::new();
    socket.connect(server[0].ip().to_string().as_str(), server[0].port());
    {
        // using block to let lock auto drop, otherwise it would hang on lock in fn wait_until_closed

        let sock = socket.as_socket_mut();
        let mut guard = sock.lock().unwrap();
        let s1 = format!("GET {} HTTP/1.1\r\n", url);
        guard.write(s1.as_bytes(), true);
        let s2 = format!("Host: {}\r\n", host);
        guard.write(s2.as_bytes(), true);
        guard.write(b"Connection: close\r\n", true);
        guard.write(b"\r\n", true);

        while !guard.closed() && !guard.eof() {
            let r = guard.read(1024);
            let text = str::from_utf8(r.as_ref()).unwrap();
            println!("{}", text);
        }
    }

    socket.wait_until_closed();

    println!("Terminated.");
}
