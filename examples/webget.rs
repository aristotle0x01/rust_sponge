use std::env;
use std::str;
use std::net::{TcpStream, ToSocketAddrs};
use std::io::{Read, Write};

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

fn get_url(host: &String, url: &String) {
    println!("host is {}", host);
    println!("url is {}", url);

    let server_details = host.to_owned() + ":80";
    let server: Vec<_> = server_details
        .to_socket_addrs()
        .expect("Unable to resolve domain")
        .collect();
    println!("{:?}", server);

    match TcpStream::connect(server_details) {
        Ok(mut stream) => {
            println!("Successfully connected to server {}", host);
            
            let s1 = format!("GET {} HTTP/1.1\r\n", url);
            stream.write(s1.as_bytes()).unwrap();
            let s2 = format!("Host: {}\r\n", host);
            stream.write(s2.as_bytes()).unwrap();
            stream.write(b"Connection: close\r\n").unwrap();
            stream.write(b"\r\n").unwrap();

            println!("Sent Hello, awaiting reply...");

            // Array with a fixed size
            let mut rx_bytes = [0u8; 1024];
            loop {
                // Read from the current data in the TcpStream
                let bytes_read = stream.read(&mut rx_bytes);
                match bytes_read {
                    Ok(n) => if n == 0 {break;},
                    _ => {break;},
                }

                let text = str::from_utf8(&rx_bytes).unwrap();
                println!("{}", text);
            }
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Terminated.");
}