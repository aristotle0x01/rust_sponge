use crate::tcp_fsm_test_harness::*;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use rust_sponge::tcp_helpers::tcp_config::TCPConfig;
use rust_sponge::wrapping_integers::WrappingInt32;
use rust_sponge::SizeT;
use std::cmp::{max, min};

mod tcp_fsm_test_harness;

#[test]
fn fsm_reorder() {
    const NREPS: u32 = 32;

    let mut cfg = TCPConfig {
        ..Default::default()
    };
    cfg.recv_capacity = 65000;

    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

    // non-overlapping out-of-order segments
    for rep_no in 0..NREPS {
        let rx_isn = WrappingInt32::new(rand::thread_rng().gen_range(0..=u32::MAX));
        let tx_isn = WrappingInt32::new(rand::thread_rng().gen_range(0..=u32::MAX));
        let mut test_1 = TCPTestHarness::in_established(&cfg, tx_isn, rx_isn);
        let mut seq_size: Vec<(SizeT, SizeT)> = Vec::new();
        let mut datalen = 0;

        while datalen < cfg.recv_capacity {
            let rd = rand::thread_rng().gen_range(0..=u32::MAX);
            let size = min(
                cfg.recv_capacity - datalen,
                (1 + rd % (TCPConfig::MAX_PAYLOAD_SIZE - 1) as u32) as SizeT,
            );
            seq_size.push((datalen, size));
            datalen += size;
        }
        seq_size.shuffle(&mut thread_rng());

        let d: String = (0..datalen)
            .map(|_| {
                let idx = rand::thread_rng().gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        let mut min_expect_ackno = rx_isn + 1;
        let mut max_expect_ackno = rx_isn + 1;
        for (off, sz) in seq_size {
            test_1.send_data(rx_isn + (1 + off) as u32, tx_isn + 1, &d[off..(off + sz)]);
            if off == min_expect_ackno.raw_value() as usize {
                min_expect_ackno = min_expect_ackno + sz as u32;
            }
            max_expect_ackno = max_expect_ackno + sz as u32;

            let seg = test_1.expect_seg(
                &mut ExpectSegment::new().with_ack(true),
                "test 1 failed: no ACK for datagram".to_string(),
            );
            let seg_hdr = seg.header();
            let b = (seg_hdr.ackno.raw_value() < min_expect_ackno.raw_value())
                || (seg_hdr.ackno.raw_value() > max_expect_ackno.raw_value());
            assert!(!b, "test 1 failed: no ack or out of expected range");
        }

        test_1.execute(&mut Tick::new(1), "".to_string());
        test_1.execute(
            ExpectData::new().with_data(d),
            "test 1 failed: got back the wrong data".to_string(),
        );
    }

    // overlapping out-of-order segments
    for rep_no in 0..NREPS {
        let rx_isn = WrappingInt32::new(rand::thread_rng().gen_range(0..=u32::MAX));
        let tx_isn = WrappingInt32::new(rand::thread_rng().gen_range(0..=u32::MAX));
        let mut test_2 = TCPTestHarness::in_established(&cfg, tx_isn, rx_isn);
        let mut seq_size: Vec<(SizeT, SizeT)> = Vec::new();
        let mut datalen = 0;

        while datalen < cfg.recv_capacity {
            let rd = rand::thread_rng().gen_range(0..=u32::MAX);
            let size = min(
                cfg.recv_capacity - datalen,
                (1 + rd % (TCPConfig::MAX_PAYLOAD_SIZE - 1) as u32) as SizeT,
            );
            let rem = TCPConfig::MAX_PAYLOAD_SIZE - size;
            let offs: SizeT;
            if rem == 0 {
                offs = 0;
            } else if rem == 1 {
                offs = min(1, datalen);
            } else {
                offs = min(
                    min(datalen, rem),
                    (1 + thread_rng().gen_range(0..=u32::MAX) as SizeT % (rem - 1)) as SizeT,
                );
            }
            assert!(
                (size + offs) <= TCPConfig::MAX_PAYLOAD_SIZE,
                "test 2 internal error: bad payload size"
            );
            seq_size.push((datalen - offs, size + offs));
            datalen += size;
        }
        assert!(
            datalen <= cfg.recv_capacity,
            "test 2 internal error: bad offset sequence"
        );
        seq_size.shuffle(&mut thread_rng());

        let d: String = (0..datalen)
            .map(|_| {
                let idx = rand::thread_rng().gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        let mut min_expect_ackno = rx_isn + 1;
        let mut max_expect_ackno = rx_isn + 1;
        for (off, sz) in seq_size {
            test_2.send_data(rx_isn + (1 + off) as u32, tx_isn + 1, &d[off..(off + sz)]);
            if off <= min_expect_ackno.raw_value() as usize
                && (off + sz) > min_expect_ackno.raw_value() as usize
            {
                min_expect_ackno = WrappingInt32::new((sz + off) as u32);
            }
            max_expect_ackno = max_expect_ackno + sz as u32;

            let seg = test_2.expect_seg(
                &mut ExpectSegment::new().with_ack(true),
                "test 2 failed: no ACK for datagram".to_string(),
            );
            let seg_hdr = seg.header();
            let b = (seg_hdr.ackno.raw_value() < min_expect_ackno.raw_value())
                || (seg_hdr.ackno.raw_value() > max_expect_ackno.raw_value());
            assert!(!b, "test 2 failed: no ack or out of expected range");
        }

        test_2.execute(&mut Tick::new(1), "".to_string());
        test_2.execute(
            ExpectData::new().with_data(d),
            "test 2 failed: got back the wrong data".to_string(),
        );
    }
}
