use crate::tcp_fsm_test_harness::*;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use rust_sponge::tcp_helpers::tcp_config::TCPConfig;
use rust_sponge::tcp_helpers::tcp_segment::TCPSegment;
use rust_sponge::wrapping_integers::WrappingInt32;
use rust_sponge::SizeT;
use std::cmp::{max, min};

mod tcp_fsm_test_harness;

#[test]
fn fsm_loopback_win() {
    const NREPS: u32 = 32;

    let mut cfg = TCPConfig {
        ..Default::default()
    };
    cfg.recv_capacity = 65000;

    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

    // non-overlapping out-of-order segments
    for rep_no in 0..NREPS {
        let rx_offset = WrappingInt32::new(rand::thread_rng().gen_range(0..=u32::MAX));
        let mut test_2 = TCPTestHarness::in_established(&cfg, rx_offset - 1, rx_offset - 1);
        test_2.send_ack(rx_offset, rx_offset, Option::Some(65000));

        let d: String = (0..cfg.recv_capacity)
            .map(|_| {
                let idx = rand::thread_rng().gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        let mut segs: Vec<TCPSegment> = Vec::new();
        let mut sendoff: SizeT = 0;
        while sendoff < d.len() {
            let len: SizeT = min(
                d.len() - sendoff,
                (thread_rng().gen_range(0..=u32::MAX) % 8192) as SizeT,
            );
            if len == 0 {
                continue;
            }
            test_2.execute(
                &mut Write::new(d[sendoff..(sendoff + len)].to_string()),
                "".to_string(),
            );
            test_2.execute(&mut Tick::new(1), "".to_string());
            test_2.execute(
                &mut ExpectSegmentAvailable {},
                "test 2 failed: cannot read after write()".to_string(),
            );
            while test_2.can_read() {
                segs.push(test_2.expect_seg(&mut ExpectSegment::new(), "".to_string()));
            }
            sendoff += len;
        }

        // resend them in shuffled order
        let mut seg_idx: Vec<SizeT> = vec![0; segs.len()];
        let mut v: SizeT = 0;
        for _i in 0..seg_idx.len() {
            seg_idx[_i] = v;
            v += 1;
        }
        seg_idx.shuffle(&mut thread_rng());
        let mut acks: Vec<TCPSegment> = Vec::new();
        for _idx in seg_idx {
            test_2.execute(&mut SendSegment::new(&segs[_idx]), "".to_string());
            test_2.execute(&mut Tick::new(1), "".to_string());

            let s = test_2.expect_one_seg(
                ExpectOneSegment::new().with_ack(true),
                "test 2 failed: no ACK after rcvd".to_string(),
            );
            acks.push(s);
            test_2.execute(
                &mut ExpectNoSegment {},
                "test 2 failed: double ACK?".to_string(),
            );
        }

        // send just the final ack
        test_2.execute(&mut SendSegment::new(acks.last().unwrap()), "".to_string());
        test_2.execute(
            &mut ExpectNoSegment {},
            "test 2 failed: ACK for ACK?".to_string(),
        );

        test_2.execute(
            ExpectData::new().with_data(d),
            "test 2 failed: wrong data after loopback".to_string(),
        );
    }
}
