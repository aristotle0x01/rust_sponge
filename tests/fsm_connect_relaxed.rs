use crate::tcp_fsm_test_harness::*;
use rand::Rng;
use rust_sponge::tcp_helpers::tcp_config::TCPConfig;
use rust_sponge::tcp_helpers::tcp_state::{State, TCPState};
use rust_sponge::wrapping_integers::WrappingInt32;

mod tcp_fsm_test_harness;

#[test]
fn fsm_connect_relaxed() {
    let cfg = TCPConfig {
        ..Default::default()
    };

    // test #1: START -> SYN_SENT -> SYN/ACK -> ACK
    {
        let mut test_1 = TCPTestHarness::new(&cfg);

        // tell the FSM to connect, make sure we get a SYN
        test_1.execute(&mut Connect {}, "".to_string());
        test_1.execute(&mut Tick::new(1), "".to_string());
        let seg1 = test_1.expect_one_seg(
            ExpectOneSegment::new().with_syn(true).with_ack(false),
            "test 1 failed: could not parse SYN segment or invalid flags".to_string(),
        );
        test_1.execute(
            &mut ExpectState::new(TCPState::from(State::SynSent)),
            "".to_string(),
        );

        // now send SYN/ACK
        let isn = rand::thread_rng().gen_range(0..=u32::MAX);
        test_1.send_syn(
            WrappingInt32::new(isn),
            Option::Some(seg1.header().seqno + 1),
        );
        test_1.execute(&mut Tick::new(1), "".to_string());
        test_1.execute(
            &mut ExpectState::new(TCPState::from(State::ESTABLISHED)),
            "".to_string(),
        );

        test_1.execute(
            ExpectOneSegment::new()
                .with_syn(false)
                .with_ack(true)
                .with_ackno_32(isn + 1),
            "".to_string(),
        );

        test_1.execute(&mut ExpectBytesInFlight::new(0), "".to_string());
    }

    // test #2: START -> SYN_SENT -> SYN -> ACK -> ESTABLISHED
    {
        let mut test_2 = TCPTestHarness::new(&cfg);

        test_2.execute(&mut Connect {}, "".to_string());
        test_2.execute(&mut Tick::new(1), "".to_string());
        let seg = test_2.expect_one_seg(
            ExpectOneSegment::new().with_syn(true).with_ack(false),
            "test 2 failed: could not parse SYN segment or invalid flags".to_string(),
        );
        let seg_hdr = seg.header();

        test_2.execute(
            &mut ExpectState::new(TCPState::from(State::SynSent)),
            "".to_string(),
        );

        // send SYN (no ACK yet)
        let isn = rand::thread_rng().gen_range(0..=u32::MAX);
        test_2.send_syn(WrappingInt32::new(isn), Option::None);
        test_2.execute(&mut Tick::new(1), "".to_string());
        test_2.expect_one_seg(
            ExpectOneSegment::new()
                .with_syn(false)
                .with_ack(true)
                .with_ackno_32(isn + 1),
            "test 2 failed: bad ACK for SYN".to_string(),
        );

        test_2.execute(
            &mut ExpectState::new(TCPState::from(State::SynRcvd)),
            "".to_string(),
        );

        // now send ACK
        test_2.send_ack(WrappingInt32::new(isn + 1), seg_hdr.seqno + 1, Option::None);
        test_2.execute(&mut Tick::new(1), "".to_string());
        test_2.execute(
            &mut ExpectNoSegment {},
            "test 2 failed: got spurious ACK after ACKing SYN".to_string(),
        );
        test_2.execute(
            &mut ExpectState::new(TCPState::from(State::ESTABLISHED)),
            "".to_string(),
        );
    }

    // test #3: START -> SYN_SENT -> SYN/ACK -> ESTABLISHED
    {
        let mut test_3 = TCPTestHarness::new(&cfg);

        test_3.execute(&mut Connect {}, "".to_string());
        test_3.execute(&mut Tick::new(1), "".to_string());
        let seg = test_3.expect_one_seg(
            ExpectOneSegment::new().with_syn(true).with_ack(false),
            "test 3 failed: could not parse SYN segment or invalid flags".to_string(),
        );
        let seg_hdr = seg.header();
        test_3.execute(
            &mut ExpectState::new(TCPState::from(State::SynSent)),
            "".to_string(),
        );

        // send SYN/ACK
        let isn = rand::thread_rng().gen_range(0..=u32::MAX);
        test_3.send_syn(WrappingInt32::new(isn), Option::Some(seg_hdr.seqno + 1));
        test_3.execute(&mut Tick::new(1), "".to_string());
        test_3.execute(
            ExpectOneSegment::new()
                .with_syn(false)
                .with_ack(true)
                .with_ackno_32(isn + 1),
            "test 3 failed: bad ACK for SYN".to_string(),
        );

        test_3.execute(
            &mut ExpectState::new(TCPState::from(State::ESTABLISHED)),
            "".to_string(),
        );
    }
}
