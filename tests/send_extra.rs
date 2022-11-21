use crate::sender_harness::{
    AckReceived, Close, ExpectNoSegment, ExpectSegment, ExpectState, TCPSenderTestHarness, Tick,
    WriteBytes,
};
use rand::thread_rng;
use rust_sponge::tcp_helpers::tcp_config::TCPConfig;
use rust_sponge::tcp_helpers::tcp_state::TCPSenderStateSummary;
use rust_sponge::wrapping_integers::WrappingInt32;
use rust_sponge::SizeT;
use std::cmp::min;

mod sender_harness;

#[test]
fn t_send_extra() {
    use rand::Rng;

    let mut rd = thread_rng();

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);
        let rto: SizeT = rd.gen_range(30..10000);
        cfg.rt_timeout = rto as u16;

        let mut test = TCPSenderTestHarness::new(
            "If already running, timer stays running when new segment sent".to_string(),
            &cfg,
        );
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(AckReceived::new(isn + 1).with_win(1000));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_ACKED.to_string(),
        ));
        test.execute(&WriteBytes::new("abc".to_string()));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(3)
                .with_data("abc".to_string())
                .with_seqno(isn + 1),
        );
        test.execute(&Tick::new(rto - 5));
        test.execute(&ExpectNoSegment {});
        test.execute(&WriteBytes::new("def".to_string()));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(3)
                .with_data("def".to_string()),
        );
        test.execute(&Tick::new(6));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(3)
                .with_data("abc".to_string())
                .with_seqno(isn + 1),
        );
        test.execute(&ExpectNoSegment {});
    }

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);
        let rto: SizeT = rd.gen_range(30..10000);
        cfg.rt_timeout = rto as u16;

        let mut test = TCPSenderTestHarness::new(
            "Retransmission still happens when expiration time not hit exactly".to_string(),
            &cfg,
        );
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(AckReceived::new(isn + 1).with_win(1000));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_ACKED.to_string(),
        ));
        test.execute(&WriteBytes::new("abc".to_string()));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(3)
                .with_data("abc".to_string())
                .with_seqno(isn + 1),
        );
        test.execute(&Tick::new(rto - 5));
        test.execute(&ExpectNoSegment {});
        test.execute(&WriteBytes::new("def".to_string()));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(3)
                .with_data("def".to_string()),
        );
        test.execute(&Tick::new(200));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(3)
                .with_data("abc".to_string())
                .with_seqno(isn + 1),
        );
        test.execute(&ExpectNoSegment {});
    }

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);
        let rto: SizeT = rd.gen_range(30..10000);
        cfg.rt_timeout = rto as u16;

        let mut test =
            TCPSenderTestHarness::new("Timer restarts on ACK of new data".to_string(), &cfg);
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(AckReceived::new(isn + 1).with_win(1000));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_ACKED.to_string(),
        ));
        test.execute(&WriteBytes::new("abc".to_string()));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(3)
                .with_data("abc".to_string())
                .with_seqno(isn + 1),
        );
        test.execute(&Tick::new(rto - 5));
        test.execute(&WriteBytes::new("def".to_string()));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(3)
                .with_data("def".to_string())
                .with_seqno(isn + 4),
        );
        test.execute(AckReceived::new(isn + 4).with_win(1000));
        test.execute(&Tick::new(rto - 1));
        test.execute(&ExpectNoSegment {});
        test.execute(&Tick::new(2));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(3)
                .with_data("def".to_string())
                .with_seqno(isn + 4),
        );
    }

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);
        let rto: SizeT = rd.gen_range(30..10000);
        cfg.rt_timeout = rto as u16;

        let mut test = TCPSenderTestHarness::new(
            "Timer doesn't restart without ACK of new data".to_string(),
            &cfg,
        );
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(AckReceived::new(isn + 1).with_win(1000));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_ACKED.to_string(),
        ));
        test.execute(&WriteBytes::new("abc".to_string()));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(3)
                .with_data("abc".to_string())
                .with_seqno(isn + 1),
        );
        test.execute(&Tick::new(rto - 5));
        test.execute(&WriteBytes::new("def".to_string()));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(3)
                .with_data("def".to_string())
                .with_seqno(isn + 4),
        );
        test.execute(AckReceived::new(isn + 1).with_win(1000));
        test.execute(&Tick::new(6));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(3)
                .with_data("abc".to_string())
                .with_seqno(isn + 1),
        );
        test.execute(&ExpectNoSegment {});
        test.execute(&Tick::new(rto * 2 - 5));
        test.execute(&ExpectNoSegment {});
        test.execute(&Tick::new(8));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(3)
                .with_data("abc".to_string())
                .with_seqno(isn + 1),
        );
        test.execute(&ExpectNoSegment {});
    }

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);
        let rto: SizeT = rd.gen_range(30..10000);
        cfg.rt_timeout = rto as u16;

        let mut test = TCPSenderTestHarness::new("RTO resets on ACK of new data".to_string(), &cfg);
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(AckReceived::new(isn + 1).with_win(1000));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_ACKED.to_string(),
        ));
        test.execute(&WriteBytes::new("abc".to_string()));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(3)
                .with_data("abc".to_string())
                .with_seqno(isn + 1),
        );
        test.execute(&Tick::new(rto - 5));
        test.execute(&WriteBytes::new("def".to_string()));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(3)
                .with_data("def".to_string())
                .with_seqno(isn + 4),
        );
        test.execute(&WriteBytes::new("ghi".to_string()));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(3)
                .with_data("ghi".to_string())
                .with_seqno(isn + 7),
        );
        test.execute(AckReceived::new(isn + 1).with_win(1000));
        test.execute(&Tick::new(6));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(3)
                .with_data("abc".to_string())
                .with_seqno(isn + 1),
        );
        test.execute(&ExpectNoSegment {});
        test.execute(&Tick::new(rto * 2 - 5));
        test.execute(&ExpectNoSegment {});
        test.execute(&Tick::new(5));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(3)
                .with_data("abc".to_string())
                .with_seqno(isn + 1),
        );
        test.execute(&ExpectNoSegment {});
        test.execute(&Tick::new(rto * 4 - 5));
        test.execute(&ExpectNoSegment {});
        test.execute(AckReceived::new(isn + 4).with_win(1000));
        test.execute(&Tick::new(rto - 1));
        test.execute(&ExpectNoSegment {});
        test.execute(&Tick::new(2));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(3)
                .with_data("def".to_string())
                .with_seqno(isn + 4),
        );
        test.execute(&ExpectNoSegment {});
    }

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);

        let nicechars = "abcdefghijklmnopqrstuvwxyz".to_string();
        let mut bigstring = String::new();
        for _i in 0..TCPConfig::DEFAULT_CAPACITY {
            let b: u8 = nicechars.as_bytes()[_i % nicechars.len()];
            let c: char = b as char;
            bigstring.push(c);
        }

        let window_size: SizeT = rd.gen_range(50000..63000);

        let mut test = TCPSenderTestHarness::new(
            "fill_window() correctly fills a big window".to_string(),
            &cfg,
        );

        test.execute(&WriteBytes::new(bigstring.to_string()));
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(AckReceived::new(isn + 1).with_win(window_size as u16));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_ACKED.to_string(),
        ));

        let _min = min(bigstring.len(), window_size);
        let mut _i = 0;
        while (_i + TCPConfig::MAX_PAYLOAD_SIZE) < _min {
            let expected_size: SizeT = min(TCPConfig::MAX_PAYLOAD_SIZE, _min - _i);
            test.execute(
                ExpectSegment::new()
                    .with_no_flags()
                    .with_payload_size(expected_size)
                    .with_data(bigstring[_i..(expected_size + _i)].to_string())
                    .with_seqno(isn + 1 + _i as u32),
            );

            _i += TCPConfig::MAX_PAYLOAD_SIZE;
        }
    }

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);
        let rto: SizeT = rd.gen_range(30..10000);
        cfg.rt_timeout = rto as u16;

        let mut test = TCPSenderTestHarness::new(
            "Retransmit a FIN-containing segment same as any other".to_string(),
            &cfg,
        );
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(AckReceived::new(isn + 1).with_win(1000));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_ACKED.to_string(),
        ));
        test.execute(WriteBytes::new("abc".to_string()).with_end_input(true));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(3)
                .with_data("abc".to_string())
                .with_seqno(isn + 1)
                .with_fin(true),
        );
        test.execute(&Tick::new(rto - 1));
        test.execute(&ExpectNoSegment {});
        test.execute(&Tick::new(2));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(3)
                .with_data("abc".to_string())
                .with_seqno(isn + 1)
                .with_fin(true),
        );
    }

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);
        let rto: SizeT = rd.gen_range(30..10000);
        cfg.rt_timeout = rto as u16;

        let mut test = TCPSenderTestHarness::new(
            "Retransmit a FIN-only segment same as any other".to_string(),
            &cfg,
        );
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(AckReceived::new(isn + 1).with_win(1000));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_ACKED.to_string(),
        ));
        test.execute(&WriteBytes::new("abc".to_string()));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(3)
                .with_data("abc".to_string())
                .with_seqno(isn + 1)
                .with_no_flags(),
        );
        test.execute(&Close {});
        test.execute(
            ExpectSegment::new()
                .with_payload_size(0)
                .with_seqno(isn + 4)
                .with_fin(true),
        );
        test.execute(&Tick::new(rto - 1));
        test.execute(&ExpectNoSegment {});
        test.execute(AckReceived::new(isn + 4).with_win(1000));
        test.execute(&Tick::new(rto - 1));
        test.execute(&ExpectNoSegment {});
        test.execute(&Tick::new(2));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(0)
                .with_seqno(isn + 4)
                .with_fin(true),
        );
        test.execute(&Tick::new(2 * rto - 5));
        test.execute(&ExpectNoSegment {});
        test.execute(&Tick::new(10));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(0)
                .with_seqno(isn + 4)
                .with_fin(true),
        );
    }

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);
        let rto: SizeT = rd.gen_range(30..10000);
        cfg.rt_timeout = rto as u16;

        let mut test = TCPSenderTestHarness::new(
            "Don't add FIN if this would make the segment exceed the receiver's window".to_string(),
            &cfg,
        );
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(WriteBytes::new("abc".to_string()).with_end_input(true));
        test.execute(AckReceived::new(isn + 1).with_win(3));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_ACKED.to_string(),
        ));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(3)
                .with_data("abc".to_string())
                .with_seqno(isn + 1)
                .with_no_flags(),
        );
        test.execute(AckReceived::new(isn + 2).with_win(2));
        test.execute(&ExpectNoSegment {});
        test.execute(AckReceived::new(isn + 3).with_win(1));
        test.execute(&ExpectNoSegment {});
        test.execute(AckReceived::new(isn + 4).with_win(1));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(0)
                .with_seqno(isn + 4)
                .with_fin(true),
        );
    }

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);
        let rto: SizeT = rd.gen_range(30..10000);
        cfg.rt_timeout = rto as u16;

        let mut test = TCPSenderTestHarness::new(
            "Don't send FIN by itself if the window is full".to_string(),
            &cfg,
        );
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(&WriteBytes::new("abc".to_string()));
        test.execute(&ExpectNoSegment {});
        test.execute(AckReceived::new(isn + 1).with_win(3));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_ACKED.to_string(),
        ));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(3)
                .with_data("abc".to_string())
                .with_seqno(isn + 1)
                .with_no_flags(),
        );
        test.execute(&Close {});
        test.execute(&ExpectNoSegment {});
        test.execute(AckReceived::new(isn + 2).with_win(2));
        test.execute(&ExpectNoSegment {});
        test.execute(AckReceived::new(isn + 3).with_win(1));
        test.execute(&ExpectNoSegment {});
        test.execute(AckReceived::new(isn + 4).with_win(1));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(0)
                .with_seqno(isn + 4)
                .with_fin(true),
        );
    }

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);
        let rto: SizeT = rd.gen_range(30..10000);
        cfg.rt_timeout = rto as u16;

        let mut test =
            TCPSenderTestHarness::new("MAX_PAYLOAD_SIZE limits payload only".to_string(), &cfg);
        let nicechars = "abcdefghijklmnopqrstuvwxyz".to_string();
        let mut bigstring = String::new();
        for _i in 0..TCPConfig::MAX_PAYLOAD_SIZE {
            let b: u8 = nicechars.as_bytes()[_i % nicechars.len()];
            let c: char = b as char;
            bigstring.push(c);
        }

        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(WriteBytes::new(bigstring.to_string()).with_end_input(true));
        test.execute(AckReceived::new(isn + 1).with_win(40000));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(TCPConfig::MAX_PAYLOAD_SIZE)
                .with_data(bigstring.to_string())
                .with_seqno(isn + 1)
                .with_fin(true),
        );
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::FIN_SENT.to_string(),
        ));
        test.execute(&AckReceived::new(
            isn + 2 + TCPConfig::MAX_PAYLOAD_SIZE as u32,
        ));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::FIN_ACKED.to_string(),
        ));
    }

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);
        let rto: SizeT = rd.gen_range(30..10000);
        cfg.rt_timeout = rto as u16;

        let mut test = TCPSenderTestHarness::new(
            "When filling window, treat a '0' window size as equal to '1' but don't back off RTO"
                .to_string(),
            &cfg,
        );
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(&WriteBytes::new("abc".to_string()));
        test.execute(&ExpectNoSegment {});
        test.execute(AckReceived::new(isn + 1).with_win(0));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_ACKED.to_string(),
        ));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(1)
                .with_data("a".to_string())
                .with_seqno(isn + 1)
                .with_no_flags(),
        );
        test.execute(&Close {});
        test.execute(&ExpectNoSegment {});

        for _i in 0..5 {
            test.execute(&Tick::new(rto - 1));
            test.execute(&ExpectNoSegment {});
            test.execute(&Tick::new(1));
            test.execute(
                ExpectSegment::new()
                    .with_payload_size(1)
                    .with_data("a".to_string())
                    .with_seqno(isn + 1)
                    .with_no_flags(),
            );
        }

        test.execute(AckReceived::new(isn + 2).with_win(0));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(1)
                .with_data("b".to_string())
                .with_seqno(isn + 2)
                .with_no_flags(),
        );

        for _i in 0..5 {
            test.execute(&Tick::new(rto - 1));
            test.execute(&ExpectNoSegment {});
            test.execute(&Tick::new(1));
            test.execute(
                ExpectSegment::new()
                    .with_payload_size(1)
                    .with_data("b".to_string())
                    .with_seqno(isn + 2)
                    .with_no_flags(),
            );
        }

        test.execute(AckReceived::new(isn + 3).with_win(0));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(1)
                .with_data("c".to_string())
                .with_seqno(isn + 3)
                .with_no_flags(),
        );

        for _i in 0..5 {
            test.execute(&Tick::new(rto - 1));
            test.execute(&ExpectNoSegment {});
            test.execute(&Tick::new(1));
            test.execute(
                ExpectSegment::new()
                    .with_payload_size(1)
                    .with_data("c".to_string())
                    .with_seqno(isn + 3)
                    .with_no_flags(),
            );
        }

        test.execute(AckReceived::new(isn + 4).with_win(0));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(0)
                .with_data("".to_string())
                .with_seqno(isn + 4)
                .with_fin(true),
        );

        for _i in 0..5 {
            test.execute(&Tick::new(rto - 1));
            test.execute(&ExpectNoSegment {});
            test.execute(&Tick::new(1));
            test.execute(
                ExpectSegment::new()
                    .with_payload_size(0)
                    .with_data("".to_string())
                    .with_seqno(isn + 4)
                    .with_fin(true),
            );
        }
    }

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);
        let rto: SizeT = rd.gen_range(30..10000);
        cfg.rt_timeout = rto as u16;

        let mut test = TCPSenderTestHarness::new(
            "Unlike a zero-size window, a full window of nonzero size should be respected"
                .to_string(),
            &cfg,
        );
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(&WriteBytes::new("abc".to_string()));
        test.execute(&ExpectNoSegment {});
        test.execute(AckReceived::new(isn + 1).with_win(1));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_ACKED.to_string(),
        ));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(1)
                .with_data("a".to_string())
                .with_seqno(isn + 1)
                .with_no_flags(),
        );
        test.execute(&Tick::new(rto - 1));
        test.execute(&ExpectNoSegment {});
        test.execute(&Tick::new(1));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(1)
                .with_data("a".to_string())
                .with_seqno(isn + 1)
                .with_no_flags(),
        );

        test.execute(&Close {});

        test.execute(&Tick::new(2 * rto - 1));
        test.execute(&ExpectNoSegment {});
        test.execute(&Tick::new(1));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(1)
                .with_data("a".to_string())
                .with_seqno(isn + 1)
                .with_no_flags(),
        );

        test.execute(&Tick::new(4 * rto - 1));
        test.execute(&ExpectNoSegment {});
        test.execute(&Tick::new(1));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(1)
                .with_data("a".to_string())
                .with_seqno(isn + 1)
                .with_no_flags(),
        );

        test.execute(AckReceived::new(isn + 2).with_win(3));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(2)
                .with_data("bc".to_string())
                .with_seqno(isn + 2)
                .with_fin(true),
        );
    }

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);
        let rto: SizeT = rd.gen_range(30..10000);
        cfg.rt_timeout = rto as u16;

        let mut test = TCPSenderTestHarness::new(
            "Repeated ACKs and outdated ACKs are harmless".to_string(),
            &cfg,
        );
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(AckReceived::new(isn + 1).with_win(1000));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_ACKED.to_string(),
        ));
        test.execute(&WriteBytes::new("abcdefg".to_string()));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(7)
                .with_data("abcdefg".to_string())
                .with_seqno(isn + 1),
        );
        test.execute(AckReceived::new(isn + 8).with_win(1000));
        test.execute(&ExpectNoSegment {});
        test.execute(AckReceived::new(isn + 8).with_win(1000));
        test.execute(AckReceived::new(isn + 8).with_win(1000));
        test.execute(AckReceived::new(isn + 8).with_win(1000));
        test.execute(&ExpectNoSegment {});
        test.execute(AckReceived::new(isn + 1).with_win(1000));
        test.execute(AckReceived::new(isn + 1).with_win(1000));
        test.execute(AckReceived::new(isn + 1).with_win(1000));
        test.execute(&ExpectNoSegment {});
        test.execute(WriteBytes::new("ijkl".to_string()).with_end_input(true));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(4)
                .with_data("ijkl".to_string())
                .with_seqno(isn + 8)
                .with_fin(true),
        );
        test.execute(AckReceived::new(isn + 1).with_win(1000));
        test.execute(AckReceived::new(isn + 1).with_win(1000));
        test.execute(AckReceived::new(isn + 1).with_win(1000));
        test.execute(AckReceived::new(isn + 8).with_win(1000));
        test.execute(AckReceived::new(isn + 8).with_win(1000));
        test.execute(AckReceived::new(isn + 8).with_win(1000));
        test.execute(AckReceived::new(isn + 12).with_win(1000));
        test.execute(AckReceived::new(isn + 12).with_win(1000));
        test.execute(AckReceived::new(isn + 12).with_win(1000));
        test.execute(AckReceived::new(isn + 1).with_win(1000));
        test.execute(&Tick::new(5 * rto));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(4)
                .with_data("ijkl".to_string())
                .with_seqno(isn + 8)
                .with_fin(true),
        );
        test.execute(&ExpectNoSegment {});
        test.execute(AckReceived::new(isn + 13).with_win(1000));
        test.execute(AckReceived::new(isn + 1).with_win(1000));
        test.execute(&Tick::new(5 * rto));
        test.execute(&ExpectNoSegment {});
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::FIN_ACKED.to_string(),
        ));
    }
}
