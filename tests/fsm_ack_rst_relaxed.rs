use crate::tcp_fsm_test_harness::*;
use rust_sponge::tcp_helpers::tcp_config::TCPConfig;
use rust_sponge::tcp_helpers::tcp_state::State::{LISTEN, RESET};
use rust_sponge::tcp_helpers::tcp_state::{State, TCPState};
use rust_sponge::wrapping_integers::WrappingInt32;
use rust_sponge::SizeT;

mod tcp_fsm_test_harness;

fn ack_listen_test(cfg: &TCPConfig, seqno: WrappingInt32, ackno: WrappingInt32, lineno: u32) {
    let mut test = TCPTestHarness::in_listen(&cfg);

    // any ACK should result in a RST
    test.send_ack(seqno, ackno, Option::None);

    test.execute(
        &mut ExpectState::new(TCPState::from(LISTEN)),
        "".to_string(),
    );
    test.execute(
        &mut ExpectNoSegment {},
        "test 3 failed: ACKs in LISTEN should be ignored".to_string(),
    );
}

fn ack_rst_syn_sent_test(
    cfg: &TCPConfig,
    base_seq: WrappingInt32,
    seqno: WrappingInt32,
    ackno: WrappingInt32,
    lineno: u32,
) {
    let mut test = TCPTestHarness::in_syn_sent(&cfg, base_seq);

    // any ACK should result in a RST
    test.send_ack(seqno, ackno, Option::None);

    test.execute(
        &mut ExpectState::new(TCPState::from(State::SynSent)),
        "".to_string(),
    );
    test.execute(
        &mut ExpectNoSegment {},
        "test 3 failed: bad ACKs in SYN_SENT should be ignored".to_string(),
    );
}

#[test]
fn fsm_ack_rst_relaxed() {
    let cfg = TCPConfig {
        ..Default::default()
    };
    let base_seq = WrappingInt32::new(1 << 31);

    // test #1: in ESTABLISHED, send unacceptable segments and ACKs
    {
        println!("Test 1");
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

        test_1.execute(
            ExpectOneSegment::new().with_ack(true).with_ackno(base_seq),
            "test 1 failed: bad ACK".to_string(),
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
        test_1.execute(
            ExpectOneSegment::new().with_ack(true).with_ackno(base_seq),
            "test 1 failed: bad ACK on late seqno".to_string(),
        );

        // packet next byte in the window - ack should advance and data should be readable
        test_1.send_byte(base_seq, Option::Some(base_seq), 1);

        test_1.execute(
            &mut ExpectData::new(),
            "test 1 failed: pkt not processed on next seqno".to_string(),
        );

        test_1.execute(
            ExpectOneSegment::new()
                .with_ack(true)
                .with_ackno(base_seq + 1),
            "test 1 failed: bad ACK".to_string(),
        );

        test_1.send_rst(base_seq + 1, Option::None);
        test_1.execute(&mut ExpectState::new(TCPState::from(RESET)), "".to_string());
    }

    // test #2: in LISTEN, send RSTs
    {
        println!("Test 2");
        let mut test_2 = TCPTestHarness::in_listen(&cfg);

        // all RSTs should be ignored in LISTEN
        test_2.send_rst(base_seq, Option::None);
        test_2.send_rst(base_seq - 1, Option::None);
        test_2.send_rst(base_seq + cfg.recv_capacity as u32, Option::None);

        test_2.execute(
            &mut ExpectNoSegment {},
            "test 2 failed: RST was not ignored in LISTEN".to_string(),
        );
    }

    // test 3: ACKs in LISTEN
    println!("Test 3");
    ack_listen_test(&cfg, base_seq, base_seq, line!());
    ack_listen_test(&cfg, base_seq - 1, base_seq, line!());
    ack_listen_test(&cfg, base_seq, base_seq - 1, line!());
    ack_listen_test(&cfg, base_seq - 1, base_seq, line!());
    ack_listen_test(&cfg, base_seq - 1, base_seq - 1, line!());
    ack_listen_test(&cfg, base_seq + cfg.recv_capacity as u32, base_seq, line!());
    ack_listen_test(&cfg, base_seq, base_seq + cfg.recv_capacity as u32, line!());
    ack_listen_test(&cfg, base_seq + cfg.recv_capacity as u32, base_seq, line!());
    ack_listen_test(
        &cfg,
        base_seq + cfg.recv_capacity as u32,
        base_seq + cfg.recv_capacity as u32,
        line!(),
    );

    // test 4: ACK and RST in SYN_SENT
    {
        println!("Test 4");
        let mut test_4 = TCPTestHarness::in_syn_sent(&cfg, base_seq);

        // good ACK with RST should result in a RESET but no RST segment sent
        test_4.send_rst(base_seq, Option::Some(base_seq + 1));

        test_4.execute(&mut ExpectState::new(TCPState::from(RESET)), "".to_string());
        test_4.execute(
            &mut ExpectNoSegment {},
            "test 4 failed: RST with good ackno should RESET the connection".to_string(),
        );
    }

    // test 5: ack/rst in SYN_SENT
    println!("Test 5");
    ack_rst_syn_sent_test(&cfg, base_seq, base_seq, base_seq, line!());
    ack_rst_syn_sent_test(&cfg, base_seq, base_seq, base_seq + 2, line!());
}
