use rand::Rng;
use rust_sponge::tcp_connection::TCPConnection;
use rust_sponge::tcp_helpers::tcp_config::TCPConfig;
use rust_sponge::tcp_helpers::tcp_segment::TCPSegment;
use rust_sponge::util::buffer::Buffer;
use rust_sponge::SizeT;
use std::cmp::min;
use std::rc::Rc;
use std::time::Instant;

const len: SizeT = 100 * 1024 * 1024;
const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

fn main() {
    main_loop(false);
    main_loop(true);
}

fn move_segments(
    x: &mut TCPConnection,
    y: &mut TCPConnection,
    segments: &mut Vec<Rc<TCPSegment>>,
    reorder: bool,
) {
    while !x.segments_out_mut().is_empty() {
        segments.push(x.segments_out_mut().pop_front().unwrap());
    }

    if reorder {
        for s in segments.iter_mut().rev() {
            y.segment_received(s);
        }
    } else {
        for s in segments {
            y.segment_received(s);
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

    if bytes_to_send.size() == 0 && !x_closed {
        x.end_input_stream();
        ret = true;
    }

    let mut segments1: Vec<Rc<TCPSegment>> = Vec::new();
    let mut segments2: Vec<Rc<TCPSegment>> = Vec::new();
    move_segments(x, y, &mut segments1, reorder);
    move_segments(y, x, &mut segments2, false);

    let available_output = y.inbound_stream().buffer_size();
    if available_output > 0 {
        string_received.push_str(y.inbound_stream_mut().read(available_output).as_str());
    }

    x.tick(1000);
    y.tick(1000);

    ret
}
fn main_loop(reorder: bool) {
    let config = TCPConfig {
        ..Default::default()
    };

    let mut x = TCPConnection::new(config.clone());
    let mut y = TCPConnection::new(config.clone());

    let string_to_send: String = (0..len)
        .map(|_| {
            let idx = rand::thread_rng().gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    let mut bytes_to_send = Buffer::new(string_to_send.as_bytes().to_vec());
    x.connect();
    y.end_input_stream();

    let mut x_closed = false;
    let mut string_received = String::with_capacity(len);

    let first_time = Instant::now();

    // let mut loop_ = || {
    //     while bytes_to_send.size() > 0 && x.remaining_outbound_capacity() > 0 {
    //         let want = min(x.remaining_outbound_capacity(), bytes_to_send.size());
    //         let written = x.write(&String::from_utf8(bytes_to_send.str()[0..want].to_owned()).unwrap());
    //         assert_eq!(want, written, "{}", format!("want = {}, written = {}", want, written));
    //         bytes_to_send.remove_prefix(written);
    //     }
    //
    //     if bytes_to_send.size() == 0 && !x_closed {
    //         x.end_input_stream();
    //         x_closed = true;
    //     }
    //
    //     let mut segments1: Vec<Rc<TCPSegment>> = Vec::new();
    //     let mut segments2: Vec<Rc<TCPSegment>> = Vec::new();
    //     move_segments(&mut x, &mut y, &mut segments1, reorder);
    //     move_segments(&mut y, &mut x, &mut segments2, false);
    //
    //     let available_output = y.inbound_stream().buffer_size();
    //     if available_output > 0 {
    //         string_received.push_str(y.inbound_stream_mut().read(available_output).as_str());
    //     }
    //
    //     x.tick(1000);
    //     y.tick(1000);
    // };

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
