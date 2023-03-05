use rust_sponge::tcp_helpers::ipv4_datagram::IPv4Datagram;
use rust_sponge::tcp_helpers::ipv4_header::IPv4Header;
use rust_sponge::tcp_helpers::tcp_header::TCPHeader;
use rust_sponge::tcp_helpers::tcp_segment::TCPSegment;
use rust_sponge::util::buffer::Buffer;
use rust_sponge::util::file_descriptor::AsFileDescriptorMut;
use rust_sponge::util::parser::ParseResult;
use rust_sponge::util::tun::TunFD;
use rust_sponge::SizeT;

fn hexdump(d: &[u8], size: SizeT) {
    println!("hexdump:{} {}", d.len(), size);
}

// cargo run --example tun
fn main() {
    let mut tun = TunFD::new("tun144");
    loop {
        let buffer = tun.read(1024 * 1024 * 2);
        println!("\n\n***\n*** Got packet:\n***\n");
        hexdump(buffer.as_slice(), buffer.len());

        let mut ip_dgram: IPv4Datagram = IPv4Datagram::new(IPv4Header::new(), Buffer::new(buffer));
        if ip_dgram.parse(0) != ParseResult::NoError {
            println!("failed.\n");
            continue;
        }

        println!(
            "success! totlen={}, IPv4 header contents:",
            &ip_dgram.header().len
        );
        println!("{}", ip_dgram.header().to_string());

        if ip_dgram.header().proto != IPv4Header::PROTO_TCP {
            println!("\nNot TCP, skipping.");
            continue;
        }

        println!("\nAttempting to parse as a TCP segment... ");

        let pseudo_cksum = ip_dgram.header().pseudo_cksum();
        let ret = TCPSegment::parse_new(ip_dgram.payload, pseudo_cksum);
        if ret.is_err() {
            println!("failed.");
            continue;
        }
        let mut tcp_seg = ret.ok().unwrap();

        println!(
            "success! payload len={}, TCP header contents:",
            tcp_seg.payload().size()
        );
        println!("{}", tcp_seg.header().to_string());
    }
}
