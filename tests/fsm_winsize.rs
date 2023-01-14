use crate::tcp_fsm_test_harness::*;
use rand::{thread_rng, Rng};
use rust_sponge::tcp_helpers::tcp_config::TCPConfig;
use rust_sponge::tcp_helpers::tcp_state::State::{ESTABLISHED, LISTEN};
use rust_sponge::tcp_helpers::tcp_state::{State, TCPState};
use rust_sponge::wrapping_integers::WrappingInt32;
use rust_sponge::SizeT;
use std::cmp::{max, min};

mod tcp_fsm_test_harness;

#[test]
fn fsm_winsize() {
    const NREPS: u32 = 32;
    const MIN_SWIN: u32 = 2048;
    const MAX_SWIN: u32 = 34816;
    const MIN_SWIN_MUL: u32 = 2;
    const MAX_SWIN_MUL: u32 = 6;

    let mut cfg = TCPConfig {
        ..Default::default()
    };
    cfg.send_capacity = (MAX_SWIN * MAX_SWIN_MUL) as SizeT;

    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

    // test 1: listen -> established -> check advertised winsize -> check sent bytes before ACK
    for rep_no in 0..NREPS {
        cfg.recv_capacity = (2048 + (thread_rng().gen_range(0..=u32::MAX) % 32768)) as SizeT;
        let seq_base = WrappingInt32::new(rand::thread_rng().gen_range(0..=u32::MAX));
        let mut test_1 = TCPTestHarness::new(&cfg);

        // connect
        test_1.execute(&mut Listen {}, "".to_string());
        test_1.send_syn(seq_base, Option::None);

        let mut one_seg = ExpectOneSegment::new();
        one_seg
            .base_mut()
            .with_ack(true)
            .with_ackno(seq_base + 1)
            .with_win(cfg.recv_capacity as u16);
        let seg = test_1.expect_one_seg(&mut one_seg, "test 1 failed: SYN/ACK invalid".to_string());
        let seg_hdr = seg.header();

        let ack_base = seg_hdr.seqno;

        // ack
        let swin: u16 =
            (MIN_SWIN + (thread_rng().gen_range(0..=u32::MAX) % (MAX_SWIN - MIN_SWIN))) as u16;
        test_1.send_ack(seq_base + 1, ack_base + 1, Option::Some(swin));

        test_1.execute(
            &mut ExpectNoSegment {},
            "test 1 failed: ACK after acceptable ACK".to_string(),
        );
        test_1.execute(
            &mut ExpectState::new(TCPState::from(ESTABLISHED)),
            "".to_string(),
        );

        // write swin_mul * swin, make sure swin gets sent
        let swin_mul: u32 =
            MIN_SWIN_MUL + (thread_rng().gen_range(0..=u32::MAX) % (MAX_SWIN_MUL - MIN_SWIN_MUL));
        let d: String = (0..(swin as u32 * swin_mul))
            .map(|_| {
                let idx = rand::thread_rng().gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();
        test_1.execute(
            Write::new(d.clone()).with_bytes_written((swin_mul * swin as u32) as SizeT),
            "".to_string(),
        );
        test_1.execute(&mut Tick::new(1), "".to_string());

        let mut d_out: String = String::with_capacity((swin as u32 * swin_mul) as usize);
        let mut bytes_total: SizeT = 0;
        while bytes_total < (swin_mul * swin as u32) as usize {
            test_1.execute(
                &mut ExpectSegmentAvailable {},
                "test 1 failed: nothing sent after write()".to_string(),
            );
            let mut bytes_read: SizeT = 0;
            while test_1.can_read() {
                let seg2 = test_1.expect_seg(
                    ExpectSegment::new()
                        .with_ack(true)
                        .with_ackno(seq_base + 1)
                        .with_win(cfg.recv_capacity as u16),
                    "test 1 failed: invalid datagrams carrying write() data".to_string(),
                );
                let seg2_hdr = seg2.header();
                bytes_read += seg2.payload().size();
                let seg2_first = (seg2_hdr.seqno - ack_base - 1).raw_value();
                d_out.insert_str(
                    seg2_first as usize,
                    String::from_utf8(seg2.payload().str().to_vec())
                        .unwrap()
                        .as_str(),
                );
            }
            assert!(
                (bytes_read + TCPConfig::MAX_PAYLOAD_SIZE) >= swin as usize,
                "test 1 failed: sender did not fill window"
            );
            test_1.execute(
                &mut ExpectBytesInFlight::new(bytes_read as u64),
                "test 1 failed: sender wrong bytes_in_flight".to_string(),
            );

            bytes_total += bytes_read;
            // NOTE that we don't override send window here because cfg should have been updated
            test_1.send_ack(
                seq_base + 1,
                ack_base + 1 + bytes_total as u32,
                Option::Some(swin),
            );
            test_1.execute(&mut Tick::new(1), "".to_string());
        }

        test_1.execute(
            &mut ExpectBytesInFlight::new(0),
            "test 1 failed: after acking, bytes still in flight?".to_string(),
        );
        assert_eq!(d, d_out, "test 1 failed: data mismatch");
    }
}
