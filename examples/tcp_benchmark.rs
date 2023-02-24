use rand::Rng;
use rust_sponge::tcp_connection::TCPConnection;
use rust_sponge::tcp_helpers::tcp_config::TCPConfig;
use rust_sponge::tcp_helpers::tcp_segment::TCPSegment;
use rust_sponge::util::buffer::Buffer;
use rust_sponge::SizeT;
use std::cmp::min;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::time::Instant;

// todo: has not met the minimal performance requirement yet

const len: SizeT = 100 * 1024 * 1024;
const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

fn main() {
    main_loop(false);
    main_loop(true);
}

fn main_loop(reorder: bool) {
    let config = TCPConfig {
        ..Default::default()
    };

    let mut x = TCPConnection::new(config.clone());
    let mut y = TCPConnection::new(config.clone());

    // let string_to_send: String = (0..len)
    //     .map(|_| {
    //         let idx = rand::thread_rng().gen_range(0..CHARSET.len());
    //         CHARSET[idx] as char
    //     })
    //     .collect();
    // https://users.rust-lang.org/t/fill-string-with-repeated-character/1121
    let string_to_send: String = (0..len).map(|_| "x").collect::<String>();

    let mut bytes_to_send = Buffer::new(string_to_send.as_bytes().to_vec());
    x.connect();
    y.end_input_stream();

    let mut x_closed = false;
    let mut string_received = String::with_capacity(len);

    let first_time = Instant::now();

    while !y.inbound_stream().eof() {
        x_closed = loop_(
            &mut x,
            &mut y,
            x_closed,
            reorder,
            &mut bytes_to_send,
            &mut string_received,
        );
    }

    assert_eq!(
        string_received.clone(),
        string_to_send.clone(),
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
        x_closed = loop_(
            &mut x,
            &mut y,
            x_closed,
            reorder,
            &mut bytes_to_send,
            &mut string_received,
        );
    }
}

fn move_segments(x: &mut TCPConnection, y: &mut TCPConnection, reorder: bool) {
    let mut segments: Vec<Arc<Mutex<TCPSegment>>> = Vec::new();

    while !x.segments_out_mut().is_empty() {
        segments.push(x.segments_out_mut().pop_front().unwrap());
    }

    if reorder {
        for s in segments.iter_mut().rev() {
            let t_ = s.lock().unwrap();
            y.segment_received(t_.deref());
        }
    } else {
        for s in segments {
            let t_ = s.lock().unwrap();
            y.segment_received(t_.deref());
        }
    }
}

fn loop_(
    x: &mut TCPConnection,
    y: &mut TCPConnection,
    x_closed: bool,
    reorder: bool,
    bytes_to_send: &mut Buffer,
    string_received: &mut String,
) -> bool {
    let mut ret = x_closed;

    while bytes_to_send.size() > 0 && x.remaining_outbound_capacity() > 0 {
        let want = min(x.remaining_outbound_capacity(), bytes_to_send.size());
        let written = x.write(&String::from_utf8(bytes_to_send.str()[0..want].to_owned()).unwrap());
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
        string_received.push_str(y.inbound_stream_mut().read(available_output).as_str());
    }

    x.tick(1000);
    y.tick(1000);

    ret
}
