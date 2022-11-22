use crate::sender_harness::{
    AckReceived, ExpectBytesInFlight, ExpectNoSegment, ExpectSegment, ExpectState,
    TCPSenderTestHarness, Tick, WriteBytes,
};
use rand::thread_rng;
use rust_sponge::tcp_helpers::tcp_config::TCPConfig;
use rust_sponge::tcp_helpers::tcp_state::TCPSenderStateSummary;
use rust_sponge::wrapping_integers::WrappingInt32;
use rust_sponge::SizeT;

mod sender_harness;

#[test]
fn t_send_retx() {
    use rand::Rng;

    let mut rd = thread_rng();

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);
        let retx_timeout: u16 = rd.gen_range(10..10000);
        cfg.fixed_isn = Option::from(isn);
        cfg.rt_timeout = retx_timeout;

        let mut test = TCPSenderTestHarness::new(
            "Retx SYN twice at the right times, then ack".to_string(),
            &cfg,
        );
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(&ExpectNoSegment {});
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_SENT.to_string(),
        ));
        test.execute(&Tick::new((retx_timeout - 1) as SizeT));
        test.execute(&ExpectNoSegment {});
        test.execute(&Tick::new(1));
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_SENT.to_string(),
        ));
        test.execute(&ExpectBytesInFlight::new(1));
        // Wait twice as long b/c exponential back-off
        test.execute(&Tick::new((2 * retx_timeout - 1) as SizeT));
        test.execute(&ExpectNoSegment {});
        test.execute(&Tick::new(1));
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_SENT.to_string(),
        ));
        test.execute(&ExpectBytesInFlight::new(1));
        test.execute(&AckReceived::new(isn + 1));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_ACKED.to_string(),
        ));
        test.execute(&ExpectBytesInFlight::new(0));
    }

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        let retx_timeout: u16 = rd.gen_range(10..10000);
        cfg.fixed_isn = Option::from(isn);
        cfg.rt_timeout = retx_timeout;

        let mut test =
            TCPSenderTestHarness::new("Retx SYN until too many retransmissions".to_string(), &cfg);
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(&ExpectNoSegment {});
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_SENT.to_string(),
        ));
        for _attempt_no in 0..TCPConfig::MAX_RETX_ATTEMPTS {
            test.execute(
                Tick::new((((retx_timeout as SizeT) << _attempt_no) - 1) as SizeT)
                    .with_max_retx_exceeded(false),
            );
            test.execute(&ExpectNoSegment {});
            test.execute(Tick::new(1).with_max_retx_exceeded(false));
            test.execute(
                ExpectSegment::new()
                    .with_no_flags()
                    .with_syn(true)
                    .with_payload_size(0)
                    .with_seqno(isn),
            );
            test.execute(&ExpectState::new(
                TCPSenderStateSummary::SYN_SENT.to_string(),
            ));
            test.execute(&ExpectBytesInFlight::new(1));
        }
        test.execute(
            Tick::new((((retx_timeout as SizeT) << TCPConfig::MAX_RETX_ATTEMPTS) - 1) as SizeT)
                .with_max_retx_exceeded(false),
        );
        test.execute(Tick::new(1).with_max_retx_exceeded(true));
    }

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);
        let retx_timeout: u16 = rd.gen_range(10..10000);
        cfg.fixed_isn = Option::from(isn);
        cfg.rt_timeout = retx_timeout;

        let mut test = TCPSenderTestHarness::new(
            "Send some data, the retx and succeed, then retx till limit".to_string(),
            &cfg,
        );

        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(&ExpectNoSegment {});
        test.execute(&AckReceived::new(isn + 1));
        test.execute(&WriteBytes::new("abcd".to_string()));
        test.execute(ExpectSegment::new().with_payload_size(4));
        test.execute(&ExpectNoSegment {});
        test.execute(&AckReceived::new(isn + 5));
        test.execute(&ExpectBytesInFlight::new(0));
        test.execute(&WriteBytes::new("efgh".to_string()));
        test.execute(ExpectSegment::new().with_payload_size(4));
        test.execute(&ExpectNoSegment {});
        test.execute(Tick::new(retx_timeout as SizeT).with_max_retx_exceeded(false));
        test.execute(ExpectSegment::new().with_payload_size(4));
        test.execute(&ExpectNoSegment {});
        test.execute(&AckReceived::new(isn + 9));
        test.execute(&ExpectBytesInFlight::new(0));
        test.execute(&WriteBytes::new("ijkl".to_string()));
        test.execute(
            ExpectSegment::new()
                .with_payload_size(4)
                .with_seqno(isn + 9),
        );
        for _attempt_no in 0..TCPConfig::MAX_RETX_ATTEMPTS {
            test.execute(
                Tick::new((((retx_timeout as SizeT) << _attempt_no) - 1) as SizeT)
                    .with_max_retx_exceeded(false),
            );
            test.execute(&ExpectNoSegment {});
            test.execute(Tick::new(1).with_max_retx_exceeded(false));
            test.execute(
                ExpectSegment::new()
                    .with_payload_size(4)
                    .with_seqno(isn + 9),
            );
            test.execute(&ExpectState::new(
                TCPSenderStateSummary::SYN_ACKED.to_string(),
            ));
            test.execute(&ExpectBytesInFlight::new(4));
        }
        test.execute(
            Tick::new((((retx_timeout as SizeT) << TCPConfig::MAX_RETX_ATTEMPTS) - 1) as SizeT)
                .with_max_retx_exceeded(false),
        );
        test.execute(Tick::new(1).with_max_retx_exceeded(true));
    }
}
