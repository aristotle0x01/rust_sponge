use crate::tcp_fsm_test_harness::*;
use rust_sponge::tcp_helpers::tcp_config::TCPConfig;
use rust_sponge::tcp_helpers::tcp_state::State::{TimeWait, CLOSED, CLOSING};
use rust_sponge::tcp_helpers::tcp_state::{State, TCPState};
use rust_sponge::wrapping_integers::WrappingInt32;
use rust_sponge::SizeT;

mod tcp_fsm_test_harness;

#[test]
fn fsm_active_close() {
    let cfg = TCPConfig {
        ..Default::default()
    };

    // test #1: start in TIME_WAIT, timeout
    {
        let mut test_1 =
            TCPTestHarness::in_time_wait(&cfg, WrappingInt32::new(0), WrappingInt32::new(0));

        test_1.execute(
            &mut Tick::new((10 * cfg.rt_timeout - 1) as SizeT),
            "".to_string(),
        );
        test_1.execute(
            &mut ExpectState::new(TCPState::from(State::TimeWait)),
            "".to_string(),
        );
        test_1.execute(&mut Tick::new(1), "".to_string());
        test_1.execute(
            &mut ExpectNotInState::new(TCPState::from(State::TimeWait)),
            "".to_string(),
        );
        test_1.execute(
            &mut Tick::new((10 * cfg.rt_timeout) as SizeT),
            "".to_string(),
        );
        test_1.execute(
            &mut ExpectState::new(TCPState::from(State::CLOSED)),
            "".to_string(),
        );
    }

    // test #2: start in CLOSING, send ack, time out
    {
        let mut test_2 =
            TCPTestHarness::in_closing(&cfg, WrappingInt32::new(0), WrappingInt32::new(0));

        test_2.execute(
            &mut Tick::new((4 * cfg.rt_timeout) as SizeT),
            "".to_string(),
        );
        let mut one_seg = ExpectOneSegment::new();
        one_seg.base_mut().with_fin(true);
        test_2.execute(&mut one_seg, "".to_string());
        test_2.execute(
            &mut ExpectState::new(TCPState::from(State::CLOSING)),
            "".to_string(),
        );
        test_2.send_ack(WrappingInt32::new(2), WrappingInt32::new(2), Option::None);
        test_2.execute(&mut ExpectNoSegment {}, "".to_string());
        test_2.execute(
            &mut ExpectState::new(TCPState::from(State::TimeWait)),
            "".to_string(),
        );
        test_2.execute(
            &mut Tick::new((10 * cfg.rt_timeout - 1) as SizeT),
            "".to_string(),
        );
        test_2.execute(
            &mut ExpectState::new(TCPState::from(State::TimeWait)),
            "".to_string(),
        );
        test_2.execute(&mut Tick::new(2), "".to_string());
        test_2.execute(
            &mut ExpectState::new(TCPState::from(State::CLOSED)),
            "".to_string(),
        );
    }

    // test #3: start in FIN_WAIT_2, send FIN, time out
    {
        let mut test_3 =
            TCPTestHarness::in_fin_wait_2(&cfg, WrappingInt32::new(0), WrappingInt32::new(0));

        test_3.execute(
            &mut Tick::new((4 * cfg.rt_timeout) as SizeT),
            "".to_string(),
        );
        test_3.execute(
            &mut ExpectState::new(TCPState::from(State::FinWait2)),
            "".to_string(),
        );
        let rx_seqno = WrappingInt32::new(1);
        test_3.send_fin(rx_seqno, Option::Some(WrappingInt32::new(2)));
        let ack_expect = rx_seqno + 1;
        test_3.execute(&mut Tick::new(1), "".to_string());
        let mut one_seg = ExpectOneSegment::new();
        one_seg.base_mut().with_ack(true).with_ackno(ack_expect);
        test_3.execute(&mut one_seg, "test 3 failed: wrong ACK for FIN".to_string());
        test_3.execute(
            &mut ExpectState::new(TCPState::from(State::TimeWait)),
            "".to_string(),
        );
        test_3.execute(
            &mut Tick::new((10 * cfg.rt_timeout) as SizeT),
            "".to_string(),
        );
        test_3.execute(
            &mut ExpectState::new(TCPState::from(State::CLOSED)),
            "".to_string(),
        );
    }

    // test #4: start in FIN_WAIT_1, ack, FIN, time out
    {
        let mut test_4 =
            TCPTestHarness::in_fin_wait_1(&cfg, WrappingInt32::new(0), WrappingInt32::new(0));

        // Expect retransmission of FIN
        test_4.execute(
            &mut Tick::new((4 * cfg.rt_timeout) as SizeT),
            "".to_string(),
        );
        let mut one_seg = ExpectOneSegment::new();
        one_seg.base_mut().with_fin(true);
        test_4.execute(&mut one_seg, "".to_string());

        // ACK the FIN
        let rx_seqno = WrappingInt32::new(1);
        test_4.send_ack(rx_seqno, WrappingInt32::new(2), Option::None);
        test_4.execute(&mut Tick::new(5), "".to_string());

        // Send our own FIN
        test_4.send_fin(rx_seqno, Option::Some(WrappingInt32::new(2)));
        let ack_expect = rx_seqno + 1;
        test_4.execute(&mut Tick::new(1), "".to_string());

        let mut one_seg1 = ExpectOneSegment::new();
        one_seg1
            .base_mut()
            .with_no_flags()
            .with_ack(true)
            .with_ackno(ack_expect);
        test_4.execute(&mut one_seg1, "".to_string());
        test_4.execute(
            &mut Tick::new((10 * cfg.rt_timeout) as SizeT),
            "".to_string(),
        );
        test_4.execute(
            &mut ExpectState::new(TCPState::from(State::CLOSED)),
            "".to_string(),
        );
    }

    // test 5: start in FIN_WAIT_1, ack, FIN, FIN again, time out
    {
        let mut test_5 =
            TCPTestHarness::in_fin_wait_1(&cfg, WrappingInt32::new(0), WrappingInt32::new(0));

        // ACK the FIN
        let rx_seqno = WrappingInt32::new(1);
        test_5.send_ack(rx_seqno, WrappingInt32::new(2), Option::None);
        test_5.execute(
            &mut ExpectState::new(TCPState::from(State::FinWait2)),
            "".to_string(),
        );
        test_5.execute(&mut Tick::new(5), "".to_string());

        test_5.send_fin(rx_seqno, Option::Some(WrappingInt32::new(2)));
        test_5.execute(
            &mut ExpectState::new(TCPState::from(State::TimeWait)),
            "".to_string(),
        );
        test_5.execute(&mut ExpectLingerTimer::new(0), "".to_string());
        let ack_expect = rx_seqno + 1;
        test_5.execute(&mut Tick::new(1), "".to_string());
        test_5.execute(&mut ExpectLingerTimer::new(1), "".to_string());
        let mut one_seg = ExpectOneSegment::new();
        one_seg
            .base_mut()
            .with_no_flags()
            .with_ack(true)
            .with_ackno(ack_expect);
        test_5.execute(&mut one_seg, "".to_string());

        test_5.execute(
            &mut Tick::new((10 * cfg.rt_timeout - 10) as SizeT),
            "".to_string(),
        );
        test_5.execute(
            &mut ExpectLingerTimer::new((10 * cfg.rt_timeout - 9) as u64),
            "".to_string(),
        );

        test_5.send_fin(rx_seqno, Option::Some(WrappingInt32::new(2)));
        test_5.execute(&mut ExpectLingerTimer::new(0), "".to_string());
        test_5.execute(&mut Tick::new(1), "".to_string());

        let mut one_seg1 = ExpectOneSegment::new();
        one_seg1.base_mut().with_ack(true).with_ackno(ack_expect);
        test_5.execute(
            &mut one_seg1,
            "test 5 failed: no ACK for 2nd FIN".to_string(),
        );

        test_5.execute(
            &mut ExpectState::new(TCPState::from(State::TimeWait)),
            "".to_string(),
        );

        // tick the timer and see what happens---a 2nd FIN in TIME_WAIT should reset the wait timer!
        // (this is an edge case of "throw it away and send another ack" for out-of-window segs)
        test_5.execute(
            &mut Tick::new((10 * cfg.rt_timeout - 10) as SizeT),
            "".to_string(),
        );
        test_5.execute(
            &mut ExpectLingerTimer::new((10 * cfg.rt_timeout - 9) as u64),
            "test 5 failed: time_since_last_segment_received() should reset after 2nd FIN"
                .to_string(),
        );

        test_5.execute(&mut ExpectNoSegment {}, "".to_string());
        test_5.execute(&mut Tick::new(10), "".to_string());
        test_5.execute(
            &mut ExpectState::new(TCPState::from(State::CLOSED)),
            "".to_string(),
        );
    }

    // test 6: start in ESTABLISHED, get FIN, get FIN re-tx, send FIN, get ACK, send ACK, time out
    {
        let mut test_6 =
            TCPTestHarness::in_established(&cfg, WrappingInt32::new(0), WrappingInt32::new(0));

        test_6.execute(&mut Close {}, "".to_string());
        test_6.execute(&mut Tick::new(1), "".to_string());

        let mut one_seg1 = ExpectOneSegment::new();
        one_seg1.base_mut().with_fin(true);
        let seg1 = test_6.expect_one_seg(
            &mut one_seg1,
            "test 6 failed: bad FIN after close()".to_string(),
        );

        let seg1_hdr = seg1.header();
        test_6.execute(
            &mut Tick::new((cfg.rt_timeout - 2) as SizeT),
            "".to_string(),
        );
        test_6.execute(
            &mut ExpectNoSegment {},
            "test 6 failed: FIN re-tx was too fast".to_string(),
        );
        test_6.execute(&mut Tick::new(2), "".to_string());
        let mut one_seg2 = ExpectOneSegment::new();
        one_seg2
            .base_mut()
            .with_fin(true)
            .with_seqno(seg1_hdr.seqno);
        let seg2 = test_6.expect_one_seg(&mut one_seg2, "test 6 failed: bad re-tx FIN".to_string());
        let seg2_hdr = seg2.header();
        let rx_seqno = WrappingInt32::new(1);
        test_6.send_fin(rx_seqno, Option::Some(WrappingInt32::new(0)));
        let ack_expect = rx_seqno + 1;
        test_6.execute(&mut Tick::new(1), "".to_string());

        test_6.execute(
            &mut ExpectState::new(TCPState::from(CLOSING)),
            "".to_string(),
        );
        let mut one_seg3 = ExpectOneSegment::new();
        one_seg3.base_mut().with_ack(true).with_ackno(ack_expect);
        test_6.execute(&mut one_seg3, "test 6 failed: bad ACK for FIN".to_string());

        test_6.send_ack(ack_expect, seg2_hdr.seqno + 1, Option::None);
        test_6.execute(&mut Tick::new(1), "".to_string());
        test_6.execute(
            &mut ExpectState::new(TCPState::from(TimeWait)),
            "".to_string(),
        );
        test_6.execute(
            &mut Tick::new((10 * cfg.rt_timeout) as SizeT),
            "".to_string(),
        );
        test_6.execute(
            &mut ExpectState::new(TCPState::from(CLOSED)),
            "".to_string(),
        );
    }
}
