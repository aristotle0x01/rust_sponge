use crate::tcp_fsm_test_harness::*;
use rust_sponge::tcp_helpers::tcp_config::TCPConfig;
use rust_sponge::tcp_helpers::tcp_state::{State, TCPState};
use rust_sponge::wrapping_integers::WrappingInt32;

mod tcp_fsm_test_harness;

#[test]
fn fsm_listen_relaxed() {
    let cfg = TCPConfig {
        ..Default::default()
    };
    let mut test_1 = TCPTestHarness::new(&cfg);

    // test #1: START -> LISTEN -> SYN -> SYN/ACK -> ACK
    {
        // tell the FSM to connect, make sure we get a SYN
        test_1.execute(&mut Listen {}, "".to_string());
        test_1.execute(
            &mut ExpectState::new(TCPState::from(State::LISTEN)),
            "".to_string(),
        );
        test_1.execute(&mut Tick::new(1), "".to_string());
        test_1.execute(
            &mut ExpectState::new(TCPState::from(State::LISTEN)),
            "".to_string(),
        );

        test_1.send_syn(WrappingInt32::new(0), Option::None);
        test_1.execute(&mut Tick::new(1), "".to_string());

        let mut one_seg = ExpectOneSegment::new();
        one_seg
            .base_mut()
            .with_syn(true)
            .with_ack(true)
            .with_ackno_32(1);
        let seg = test_1.expect_one_seg(
            &mut one_seg,
            "test 1 failed: no SYN/ACK in response to SYN".to_string(),
        );
        test_1.execute(
            &mut ExpectState::new(TCPState::from(State::SynRcvd)),
            "".to_string(),
        );
        test_1.send_ack(WrappingInt32::new(1), seg.header().seqno + 1, Option::None);
        test_1.execute(&mut Tick::new(1), "".to_string());
        test_1.execute(
            &mut ExpectNoSegment {},
            "test 1 failed: no need to ACK an ACK".to_string(),
        );
        test_1.execute(
            &mut ExpectState::new(TCPState::from(State::ESTABLISHED)),
            "".to_string(),
        );
    }
}
