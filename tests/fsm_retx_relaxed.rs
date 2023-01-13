use crate::tcp_fsm_test_harness::*;
use rand::Rng;
use rust_sponge::tcp_helpers::tcp_config::TCPConfig;
use rust_sponge::tcp_helpers::tcp_state::State::{CloseWait, LastAck, CLOSED, ESTABLISHED, RESET};
use rust_sponge::tcp_helpers::tcp_state::{State, TCPState};
use rust_sponge::wrapping_integers::WrappingInt32;
use rust_sponge::SizeT;

mod tcp_fsm_test_harness;

#[test]
fn fsm_retx_relaxed() {
    let mut cfg = TCPConfig {
        ..Default::default()
    };
    cfg.recv_capacity = 65000;

    // NOTE: the timeouts in this test are carefully adjusted to work whether the tcp_state_machine sends
    // immediately upon write() or only after the next tick(). If you edit this test, make sure that
    // it passes both ways (i.e., with and without calling _try_send() in TCPConnection::write())

    // NOTE 2: ACK -- I think I was successful, although given unrelated
    // refactor to template code it will be more challenging now
    // to wait until tick() to send an outgoing segment.

    // single segment re-transmit
    {
        let tx_ackno = WrappingInt32::new(rand::thread_rng().gen_range(1..=u32::MAX));
        let mut test_1 = TCPTestHarness::in_established(&cfg, tx_ackno - 1, tx_ackno - 1);

        let data = "asdf".to_string();
        test_1.execute(&mut Write::new(data.clone()), "".to_string());
        test_1.execute(&mut Tick::new(1), "".to_string());

        check_segment(&mut test_1, &data, false, line!());

        test_1.execute(
            &mut Tick::new((cfg.rt_timeout - 2) as SizeT),
            "".to_string(),
        );
        test_1.execute(
            &mut ExpectNoSegment {},
            "test 1 failed: re-tx too fast".to_string(),
        );

        test_1.execute(&mut Tick::new(2), "".to_string());
        check_segment(&mut test_1, &data, false, line!());

        test_1.execute(
            &mut Tick::new((10 * cfg.rt_timeout + 100) as SizeT),
            "".to_string(),
        );
        check_segment(&mut test_1, &data, false, line!());

        for i in 2..TCPConfig::MAX_RETX_ATTEMPTS {
            test_1.execute(
                &mut Tick::new((((cfg.rt_timeout as u32) << i) as u32 - i) as SizeT),
                "".to_string(),
            ); // exponentially increasing delay length
            test_1.execute(
                &mut ExpectNoSegment {},
                "test 1 failed: re-tx too fast after timeout".to_string(),
            );
            test_1.execute(&mut Tick::new(i as SizeT), "".to_string());
            check_segment(&mut test_1, &data, false, line!());
        }

        test_1.execute(
            &mut ExpectState::new(TCPState::from(ESTABLISHED)),
            "".to_string(),
        );

        test_1.execute(
            &mut Tick::new((1 + (cfg.rt_timeout << TCPConfig::MAX_RETX_ATTEMPTS)) as SizeT),
            "".to_string(),
        );
        test_1.execute(&mut ExpectState::new(TCPState::from(RESET)), "".to_string());
        let mut one_seg = ExpectOneSegment::new();
        one_seg.base_mut().with_rst(true);
        test_1.execute(
            &mut one_seg,
            "test 1 failed: RST on re-tx failure was malformed".to_string(),
        );
    }
}
