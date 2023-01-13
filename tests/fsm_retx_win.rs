use crate::tcp_fsm_test_harness::*;
use rand::Rng;
use rust_sponge::tcp_helpers::tcp_config::TCPConfig;
use rust_sponge::tcp_helpers::tcp_state::State::{CloseWait, LastAck, CLOSED, ESTABLISHED, RESET};
use rust_sponge::tcp_helpers::tcp_state::{State, TCPState};
use rust_sponge::wrapping_integers::WrappingInt32;
use rust_sponge::SizeT;

mod tcp_fsm_test_harness;

fn backoff_test(num_backoffs: u32, cfg: &TCPConfig) {
    let tx_ackno = WrappingInt32::new(rand::thread_rng().gen_range(0..=u32::MAX));
    let mut test_1 = TCPTestHarness::in_established(&cfg, tx_ackno - 1, tx_ackno - 1);

    let d1 = "asdf".to_string();
    let d2 = "qwer".to_string();
    test_1.execute(&mut Write::new(d1.clone()), "".to_string());
    test_1.execute(&mut Tick::new(1), "".to_string());
    test_1.execute(&mut Tick::new(20), "".to_string());
    test_1.execute(&mut Write::new(d2.clone()), "".to_string());
    test_1.execute(&mut Tick::new(1), "".to_string());

    test_1.execute(
        &mut ExpectSegmentAvailable {},
        "test 4 failed: cannot read after write()s".to_string(),
    );
    check_segment(&mut test_1, &d1, true, line!());
    check_segment(&mut test_1, &d2, false, line!());

    test_1.execute(
        &mut Tick::new((cfg.rt_timeout - 23) as SizeT),
        "".to_string(),
    );
    test_1.execute(
        &mut ExpectNoSegment {},
        "test 4 failed: re-tx too fast".to_string(),
    );

    test_1.execute(&mut Tick::new(4), "".to_string());
    check_segment(&mut test_1, &d1, false, line!());

    for _i in 1..num_backoffs {
        test_1.execute(
            &mut Tick::new((((cfg.rt_timeout as u32) << _i) - _i) as SizeT),
            "".to_string(),
        );
        test_1.execute(
            &mut ExpectNoSegment {},
            "test 4 failed: re-tx too fast after timeout".to_string(),
        );
        test_1.execute(&mut Tick::new(_i as SizeT), "".to_string());
        check_segment(&mut test_1, &d1, false, line!());
    }

    test_1.send_ack(tx_ackno, tx_ackno + 4, Option::None);
    test_1.execute(
        &mut Tick::new((cfg.rt_timeout - 2) as SizeT),
        "".to_string(),
    );
    test_1.execute(
        &mut ExpectNoSegment {},
        format!(
            "test 4 failed: re-tx of 2nd seg after ack for 1st seg too fast after {} backoffs",
            num_backoffs
        ),
    );

    test_1.execute(&mut Tick::new(3), "".to_string());
    check_segment(&mut test_1, &d2, false, line!());
}

#[test]
fn fsm_retx_win() {
    let mut cfg = TCPConfig {
        ..Default::default()
    };
    cfg.recv_capacity = 65000;

    // multiple segments with intervening ack
    {
        let tx_ackno = WrappingInt32::new(rand::thread_rng().gen_range(0..=u32::MAX));
        let mut test_1 = TCPTestHarness::in_established(&cfg, tx_ackno - 1, tx_ackno - 1);

        let d1 = "asdf".to_string();
        let d2 = "qwer".to_string();
        test_1.execute(&mut Write::new(d1.clone()), "".to_string());
        test_1.execute(&mut Tick::new(1), "".to_string());
        test_1.execute(&mut Tick::new(20), "".to_string());
        test_1.execute(&mut Write::new(d2.clone()), "".to_string());
        test_1.execute(&mut Tick::new(1), "".to_string());

        test_1.execute(
            &mut ExpectSegmentAvailable {},
            "test 2 failed: cannot read after write()s".to_string(),
        );

        check_segment(&mut test_1, &d1, true, line!());
        check_segment(&mut test_1, &d2, false, line!());

        test_1.execute(
            &mut Tick::new((cfg.rt_timeout - 23) as SizeT),
            "".to_string(),
        );
        test_1.execute(
            &mut ExpectNoSegment {},
            "test 2 failed: re-tx too fast".to_string(),
        );

        test_1.execute(&mut Tick::new(4), "".to_string());
        check_segment(&mut test_1, &d1, false, line!());

        test_1.execute(
            &mut Tick::new((2 * cfg.rt_timeout - 2) as SizeT),
            "".to_string(),
        );
        test_1.execute(
            &mut ExpectNoSegment {},
            "test 2 failed: re-tx too fast".to_string(),
        );

        test_1.send_ack(tx_ackno, tx_ackno + 4, Option::None);
        test_1.execute(
            &mut Tick::new((cfg.rt_timeout - 2) as SizeT),
            "".to_string(),
        );
        test_1.execute(
            &mut ExpectNoSegment {},
            "test 2 failed: re-tx of 2nd seg after ack for 1st seg too fast".to_string(),
        );
        test_1.execute(&mut Tick::new(3), "".to_string());
        check_segment(&mut test_1, &d2, false, line!());
    }

    // multiple segments without intervening ack
    {
        let tx_ackno = WrappingInt32::new(rand::thread_rng().gen_range(0..=u32::MAX));
        let mut test_1 = TCPTestHarness::in_established(&cfg, tx_ackno - 1, tx_ackno - 1);

        let d1 = "asdf".to_string();
        let d2 = "qwer".to_string();
        test_1.execute(&mut Write::new(d1.clone()), "".to_string());
        test_1.execute(&mut Tick::new(1), "".to_string());
        test_1.execute(&mut Tick::new(20), "".to_string());
        test_1.execute(&mut Write::new(d2.clone()), "".to_string());
        test_1.execute(&mut Tick::new(1), "".to_string());

        test_1.execute(
            &mut ExpectSegmentAvailable {},
            "test 3 failed: cannot read after write()s".to_string(),
        );
        check_segment(&mut test_1, &d1, true, line!());
        check_segment(&mut test_1, &d2, false, line!());

        test_1.execute(
            &mut Tick::new((cfg.rt_timeout - 23) as SizeT),
            "".to_string(),
        );
        test_1.execute(
            &mut ExpectNoSegment {},
            "test 3 failed: re-tx too fast".to_string(),
        );

        test_1.execute(&mut Tick::new(4), "".to_string());
        check_segment(&mut test_1, &d1, false, line!());

        test_1.execute(
            &mut Tick::new((2 * cfg.rt_timeout - 2) as SizeT),
            "".to_string(),
        );
        test_1.execute(
            &mut ExpectNoSegment {},
            "test 3 failed: re-tx of 2nd seg too fast".to_string(),
        );

        test_1.execute(&mut Tick::new(3), "".to_string());
        check_segment(&mut test_1, &d1, false, line!());
    }

    for _i in 0..TCPConfig::MAX_RETX_ATTEMPTS {
        backoff_test(_i, &cfg);
    }
}
