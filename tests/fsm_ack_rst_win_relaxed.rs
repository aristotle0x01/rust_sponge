use crate::tcp_fsm_test_harness::*;
use rand::Rng;
use rust_sponge::tcp_helpers::tcp_config::TCPConfig;
use rust_sponge::tcp_helpers::tcp_state::State::{
    CloseWait, LastAck, CLOSED, ESTABLISHED, LISTEN, RESET,
};
use rust_sponge::tcp_helpers::tcp_state::{State, TCPState};
use rust_sponge::wrapping_integers::WrappingInt32;
use rust_sponge::SizeT;

mod tcp_fsm_test_harness;

#[test]
fn fsm_ack_rst_win_relaxed() {
    let cfg = TCPConfig {
        ..Default::default()
    };
    let base_seq = WrappingInt32::new(1 << 31);

    // test #1: in ESTABLISHED, send unacceptable segments and ACKs
    {
        let mut test_1 = TCPTestHarness::in_established(&cfg, base_seq - 1, base_seq - 1);

        // acceptable ack---no response
        test_1.send_ack(base_seq, base_seq, Option::None);

        test_1.execute(
            &mut ExpectNoSegment {},
            "test 1 failed: ACK after acceptable ACK".to_string(),
        );

        // ack in the past---no response
        test_1.send_ack(base_seq, base_seq - 1, Option::None);

        test_1.execute(
            &mut ExpectNoSegment {},
            "test 1 failed: ACK after past ACK".to_string(),
        );

        // segment out of the window---should get an ACK
        test_1.send_byte(base_seq - 1, Option::Some(base_seq), 1);

        test_1.execute(
            &mut ExpectUnassembledBytes::new(0),
            "test 1 failed: seg queued on early seqno".to_string(),
        );
        let mut one_seg = ExpectOneSegment::new();
        one_seg.base_mut().with_ackno(base_seq);
        test_1.execute(
            &mut one_seg,
            "test 1 failed: no ack on early seqno".to_string(),
        );

        // segment out of the window---should get an ACK
        test_1.send_byte(
            base_seq + cfg.recv_capacity as u32,
            Option::Some(base_seq),
            1,
        );

        test_1.execute(
            &mut ExpectUnassembledBytes::new(0),
            "test 1 failed: seg queued on late seqno".to_string(),
        );
        let mut one_seg1 = ExpectOneSegment::new();
        one_seg1.base_mut().with_ackno(base_seq);
        test_1.execute(
            &mut one_seg1,
            "test 1 failed: no ack on late seqno".to_string(),
        );

        // segment in the window but late---should get an ACK and seg should be queued
        test_1.send_byte(
            base_seq + cfg.recv_capacity as u32 - 1,
            Option::Some(base_seq),
            1,
        );

        test_1.execute(
            &mut ExpectUnassembledBytes::new(1),
            "seg not queued on end-of-window seqno".to_string(),
        );

        let mut one_seg2 = ExpectOneSegment::new();
        one_seg2.base_mut().with_ackno(base_seq);
        test_1.execute(
            &mut one_seg2,
            "test 1 failed: no ack on end-of-window seqno".to_string(),
        );
        test_1.execute(
            &mut ExpectNoData {},
            "test 1 failed: no ack on end-of-window seqno".to_string(),
        );

        // segment next byte in the window - ack should advance and data should be readable
        test_1.send_byte(base_seq, Option::Some(base_seq), 1);

        test_1.execute(
            &mut ExpectUnassembledBytes::new(1),
            "seg not processed on next seqno".to_string(),
        );
        let mut one_seg3 = ExpectOneSegment::new();
        one_seg3.base_mut().with_ackno(base_seq + 1);
        test_1.execute(
            &mut one_seg3,
            "test 1 failed: no ack on next seqno".to_string(),
        );
        test_1.execute(
            &mut ExpectData::new(),
            "test 1 failed: no ack on next seqno".to_string(),
        );

        test_1.send_rst(base_seq + 1, Option::None);
        test_1.execute(&mut ExpectState::new(TCPState::from(RESET)), "".to_string());
    }
}
