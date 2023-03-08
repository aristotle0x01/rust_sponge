use rust_sponge::tcp_connection::TCPConnection;
use rust_sponge::tcp_helpers::tcp_config::TCPConfig;
use rust_sponge::tcp_helpers::tcp_segment::TCPSegment;
use rust_sponge::util::buffer::Buffer;
use rust_sponge::SizeT;
use std::cmp::min;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::ops::Deref;
use std::time::Instant;

// todo: has not met the minimal performance requirement yet

const len: SizeT = 100 * 1024 * 1024;
const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

// #[bench]
fn main() {
    main_loop(false);
    main_loop(true);
}

fn main_loop(reorder: bool) {
    let config = TCPConfig {
        ..Default::default()
    };

    let mut x = TCPConnection::new2(config.clone(), "x".to_string());
    let mut y = TCPConnection::new2(config.clone(), "y".to_string());

    // let string_to_send: String = (0..len)
    //     .map(|_| {
    //         let idx = rand::thread_rng().gen_range(0..CHARSET.len());
    //         CHARSET[idx] as char
    //     })
    //     .collect();
    // https://users.rust-lang.org/t/fill-string-with-repeated-character/1121
    // let string_to_send = (0..len)
    //     .map(|_| "x")
    //     .collect::<String>()
    //     .as_bytes()
    //     .to_vec();
    let string_to_send = vec![49u8; len];
    let sent_hash = seahash::hash(&string_to_send);

    let mut bytes_to_send = Buffer::new(string_to_send);
    x.connect();
    y.end_input_stream();

    let mut x_closed = false;
    let mut string_received = Vec::with_capacity(len);

    let first_time = Instant::now();

    while !y.inbound_stream().eof() {
        let b = x_closed;
        x_closed = loop_(
            &mut x,
            &mut y,
            b,
            reorder,
            &mut bytes_to_send,
            &mut string_received,
        );
    }

    assert_eq!(
        len,
        string_received.len(),
        "strings sent vs. received len don't match"
    );
    let recv_hash = seahash::hash(&string_received);
    assert_eq!(
        sent_hash, recv_hash,
        "strings sent vs. received don't match"
    );

    let duration = first_time.elapsed();

    let gigabits_per_second = len as f64 * 8.0 / duration.as_nanos() as f64;
    println!(
        "CPU-limited throughput {} {} Gbit/s",
        if reorder {
            " with reordering: "
        } else {
            "                : "
        },
        gigabits_per_second
    );

    while x.active() || y.active() {
        let b = x_closed;
        x_closed = loop_(
            &mut x,
            &mut y,
            b,
            reorder,
            &mut bytes_to_send,
            &mut string_received,
        );
    }
}

fn move_segments(x: &mut TCPConnection, y: &mut TCPConnection, reorder: bool) {
    if reorder {
        while !x.segments_out_mut().is_empty() {
            let t = x.segments_out_mut().back().unwrap();
            y.segment_received(t);
            x.segments_out_mut().pop_back();
        }
    } else {
        while !x.segments_out_mut().is_empty() {
            let t = x.segments_out_mut().front().unwrap();
            // eprintln!("xout: {}", t.header().summary());
            y.segment_received(t);
            x.segments_out_mut().pop_front();
        }
    }
}

fn loop_(
    x: &mut TCPConnection,
    y: &mut TCPConnection,
    x_closed: bool,
    reorder: bool,
    bytes_to_send: &mut Buffer,
    mut string_received: &mut Vec<u8>,
) -> bool {
    let mut ret = x_closed;

    while bytes_to_send.size() > 0 && x.remaining_outbound_capacity() > 0 {
        let want = min(x.remaining_outbound_capacity(), bytes_to_send.size());
        let written = x.write(&bytes_to_send.str()[0..want]);

        assert_eq!(
            want,
            written,
            "{}",
            format!("want = {}, written = {}", want, written)
        );
        bytes_to_send.remove_prefix(written);
    }

    if bytes_to_send.size() == 0 && !ret {
        x.end_input_stream();
        ret = true;
    }

    move_segments(x, y, reorder);
    move_segments(y, x, false);

    let available_output = y.inbound_stream().buffer_size();
    if available_output > 0 {
        string_received.extend_from_slice(y.inbound_stream_mut().read(available_output).as_slice());
    }

    x.tick(1000);
    y.tick(1000);

    ret
}
