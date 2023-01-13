use crate::tcp_fsm_test_harness::*;
use rand::{thread_rng, Rng};
use rust_sponge::tcp_helpers::tcp_config::TCPConfig;
use rust_sponge::wrapping_integers::WrappingInt32;
use rust_sponge::SizeT;
use std::cmp::{max, min};

mod tcp_fsm_test_harness;

#[test]
fn fsm_loopback() {
    const NREPS: u32 = 64;

    let mut cfg = TCPConfig {
        ..Default::default()
    };
    cfg.recv_capacity = 65000;

    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

    // non-overlapping out-of-order segments
    for rep_no in 0..NREPS {
        let rx_offset = WrappingInt32::new(rand::thread_rng().gen_range(0..=u32::MAX));
        let mut test_1 = TCPTestHarness::in_established(&cfg, rx_offset - 1, rx_offset - 1);
        test_1.send_ack(rx_offset, rx_offset, Option::Some(65000));

        let d: String = (0..cfg.recv_capacity)
            .map(|_| {
                let idx = rand::thread_rng().gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        let mut sendoff: SizeT = 0;
        while sendoff < d.len() {
            let len: SizeT = min(
                d.len() - sendoff,
                (thread_rng().gen_range(0..=u32::MAX) % 8192) as SizeT,
            );
            if len == 0 {
                continue;
            }
            test_1.execute(
                &mut Write::new(d[sendoff..(sendoff + len)].to_string()),
                "".to_string(),
            );
            test_1.execute(&mut Tick::new(1), "".to_string());
            test_1.execute(&mut ExpectBytesInFlight::new(len as u64), "".to_string());

            test_1.execute(
                &mut ExpectSegmentAvailable {},
                "test 1 failed: cannot read after write()".to_string(),
            );

            let n_segments: SizeT =
                (len as f64 / TCPConfig::MAX_PAYLOAD_SIZE as f64).ceil() as SizeT;
            let mut bytes_remaining = len;

            // Transfer the data segments
            for _i in 0..n_segments {
                let expect_size = min(bytes_remaining, TCPConfig::MAX_PAYLOAD_SIZE);
                let seg = test_1.expect_seg(
                    ExpectSegment::new().with_payload_size(expect_size),
                    "".to_string(),
                );
                bytes_remaining -= expect_size;
                test_1.execute(&mut SendSegment::new(&seg), "".to_string());
                test_1.execute(&mut Tick::new(1), "".to_string());
            }

            // Transfer the (bare) ack segments
            for _i in 0..n_segments {
                let seg = test_1.expect_seg(
                    ExpectSegment::new().with_ack(true).with_payload_size(0),
                    "".to_string(),
                );
                test_1.execute(&mut SendSegment::new(&seg), "".to_string());
                test_1.execute(&mut Tick::new(1), "".to_string());
            }

            test_1.execute(&mut ExpectNoSegment {}, "".to_string());
            test_1.execute(&mut ExpectBytesInFlight::new(0), "".to_string());

            sendoff += len;
        }

        test_1.execute(
            ExpectData::new().with_data(d),
            "test 1 failed: got back the wrong data".to_string(),
        );
    }
}
