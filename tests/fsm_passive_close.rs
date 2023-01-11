use crate::tcp_fsm_test_harness::*;
use rust_sponge::tcp_helpers::tcp_config::TCPConfig;
use rust_sponge::tcp_helpers::tcp_state::State::{CloseWait, LastAck, CLOSED, ESTABLISHED};
use rust_sponge::tcp_helpers::tcp_state::{State, TCPState};
use rust_sponge::wrapping_integers::WrappingInt32;
use rust_sponge::SizeT;

mod tcp_fsm_test_harness;

#[test]
fn fsm_passive_close() {
    let cfg = TCPConfig {
        ..Default::default()
    };

    // test #1: start in LAST_ACK, ack
    {
        let mut test_1 =
            TCPTestHarness::in_last_ack(&cfg, WrappingInt32::new(0), WrappingInt32::new(0));

        test_1.execute(
            &mut Tick::new((4 * cfg.rt_timeout) as SizeT),
            "".to_string(),
        );
        test_1.execute(
            &mut ExpectState::new(TCPState::from(State::LastAck)),
            "".to_string(),
        );
        test_1.send_ack(WrappingInt32::new(2), WrappingInt32::new(2), Option::None);
        test_1.execute(&mut Tick::new(1), "".to_string());
        test_1.execute(
            &mut ExpectState::new(TCPState::from(State::CLOSED)),
            "".to_string(),
        );
    }

    // test #2: start in CLOSE_WAIT, close(), throw away first FIN, ack re-tx FIN
    {
        let mut test_2 =
            TCPTestHarness::in_close_wait(&cfg, WrappingInt32::new(0), WrappingInt32::new(0));

        test_2.execute(
            &mut Tick::new((4 * cfg.rt_timeout) as SizeT),
            "".to_string(),
        );
        test_2.execute(
            &mut ExpectState::new(TCPState::from(CloseWait)),
            "".to_string(),
        );
        test_2.execute(&mut Close {}, "".to_string());
        test_2.execute(&mut Tick::new(1), "".to_string());

        test_2.execute(
            &mut ExpectState::new(TCPState::from(LastAck)),
            "".to_string(),
        );

        let mut one_seg1 = ExpectOneSegment::new();
        one_seg1.base_mut().with_fin(true);
        let seg1 = test_2.expect_one_seg(
            &mut one_seg1,
            "test 2 falied: bad seg or no FIN".to_string(),
        );

        test_2.execute(
            &mut Tick::new((cfg.rt_timeout - 2) as SizeT),
            "".to_string(),
        );

        test_2.execute(
            &mut ExpectNoSegment {},
            "test 2 failed: FIN re-tx was too fast".to_string(),
        );

        test_2.execute(&mut Tick::new(2), "".to_string());

        let mut one_seg2 = ExpectOneSegment::new();
        one_seg2
            .base_mut()
            .with_fin(true)
            .with_seqno(seg1.header().seqno);
        let seg2 = test_2.expect_one_seg(&mut one_seg2, "test 2 failed: bad re-tx FIN".to_string());

        let rx_seqno = WrappingInt32::new(2);
        let ack_expect = rx_seqno.clone();
        test_2.send_ack(ack_expect, seg2.header().seqno - 1, Option::None); // wrong ackno!
        test_2.execute(&mut Tick::new(1), "".to_string());

        test_2.execute(
            &mut ExpectState::new(TCPState::from(LastAck)),
            "".to_string(),
        );

        test_2.send_ack(ack_expect, seg2.header().seqno + 1, Option::None);
        test_2.execute(&mut Tick::new(1), "".to_string());

        test_2.execute(
            &mut ExpectState::new(TCPState::from(CLOSED)),
            "".to_string(),
        );
    }

    // test #3: start in ESTABLSHED, send FIN, recv ACK, check for CLOSE_WAIT
    {
        let mut test_3 =
            TCPTestHarness::in_established(&cfg, WrappingInt32::new(0), WrappingInt32::new(0));

        test_3.execute(
            &mut Tick::new((4 * cfg.rt_timeout) as SizeT),
            "".to_string(),
        );
        test_3.execute(
            &mut ExpectState::new(TCPState::from(ESTABLISHED)),
            "".to_string(),
        );

        let rx_seqno = WrappingInt32::new(1);
        let ack_expect = rx_seqno + 1;
        test_3.send_fin(rx_seqno, Option::Some(WrappingInt32::new(0)));
        test_3.execute(&mut Tick::new(1), "".to_string());

        let mut one_seg1 = ExpectOneSegment::new();
        one_seg1.base_mut().with_ack(true).with_ackno(ack_expect);
        test_3.execute(
            &mut one_seg1,
            "test 3 failed: bad seg, no ACK, or wrong ackno".to_string(),
        );

        test_3.execute(
            &mut ExpectState::new(TCPState::from(CloseWait)),
            "".to_string(),
        );

        test_3.send_fin(rx_seqno, Option::Some(WrappingInt32::new(0)));
        test_3.execute(&mut Tick::new(1), "".to_string());

        let mut one_seg2 = ExpectOneSegment::new();
        one_seg2.base_mut().with_ack(true).with_ackno(ack_expect);
        test_3.execute(
            &mut one_seg2,
            "test 3 falied: bad response to 2nd FIN".to_string(),
        );

        test_3.execute(
            &mut ExpectState::new(TCPState::from(CloseWait)),
            "".to_string(),
        );

        test_3.execute(&mut Tick::new(1), "".to_string());
        test_3.execute(&mut Close {}, "".to_string());
        test_3.execute(&mut Tick::new(1), "".to_string());

        test_3.execute(
            &mut ExpectState::new(TCPState::from(LastAck)),
            "".to_string(),
        );

        let mut one_seg3 = ExpectOneSegment::new();
        one_seg3.base_mut().with_fin(true);
        let seg3 = test_3.expect_one_seg(
            &mut one_seg3,
            "test 3 failed: bad seg or no FIN".to_string(),
        );

        test_3.send_ack(ack_expect, seg3.header().seqno + 1, Option::None);
        test_3.execute(&mut Tick::new(1), "".to_string());

        test_3.execute(
            &mut ExpectState::new(TCPState::from(CLOSED)),
            "".to_string(),
        );
    }
}
