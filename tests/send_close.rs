use crate::sender_harness::{
    AckReceived, Close, ExpectBytesInFlight, ExpectNoSegment, ExpectSegment, ExpectState,
    TCPSenderTestHarness, Tick,
};
use rand::thread_rng;
use rust_sponge::tcp_helpers::tcp_config::TCPConfig;
use rust_sponge::tcp_helpers::tcp_state::TCPSenderStateSummary;
use rust_sponge::wrapping_integers::WrappingInt32;
use rust_sponge::SizeT;

mod sender_harness;

#[test]
fn t_send_close() {
    use rand::Rng;

    let mut rd = thread_rng();

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);

        let mut test = TCPSenderTestHarness::new("FIN sent test".to_string(), &cfg);
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(&AckReceived::new(WrappingInt32::new(isn.raw_value() + 1)));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_ACKED.to_string(),
        ));
        test.execute(&Close {});
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::FIN_SENT.to_string(),
        ));
        test.execute(&ExpectBytesInFlight::new(1));
        test.execute(
            ExpectSegment::new()
                .with_fin(true)
                .with_seqno(WrappingInt32::new(isn.raw_value() + 1)),
        );
        test.execute(&ExpectNoSegment {});
    }

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);

        let mut test = TCPSenderTestHarness::new("FIN sent test".to_string(), &cfg);
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(&AckReceived::new(WrappingInt32::new(isn.raw_value() + 1)));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_ACKED.to_string(),
        ));
        test.execute(&Close {});
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::FIN_SENT.to_string(),
        ));
        test.execute(
            ExpectSegment::new()
                .with_fin(true)
                .with_seqno(WrappingInt32::new(isn.raw_value() + 1)),
        );
        test.execute(&AckReceived::new(WrappingInt32::new(isn.raw_value() + 2)));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::FIN_ACKED.to_string(),
        ));
        test.execute(&ExpectBytesInFlight::new(0));
        test.execute(&ExpectNoSegment {});
    }

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);

        let mut test = TCPSenderTestHarness::new("FIN not acked test".to_string(), &cfg);
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(&AckReceived::new(WrappingInt32::new(isn.raw_value() + 1)));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_ACKED.to_string(),
        ));
        test.execute(&Close {});
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::FIN_SENT.to_string(),
        ));
        test.execute(
            ExpectSegment::new()
                .with_fin(true)
                .with_seqno(WrappingInt32::new(isn.raw_value() + 1)),
        );
        test.execute(&AckReceived::new(WrappingInt32::new(isn.raw_value() + 1)));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::FIN_SENT.to_string(),
        ));
        test.execute(&ExpectBytesInFlight::new(1));
        test.execute(&ExpectNoSegment {});
    }

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);

        let mut test = TCPSenderTestHarness::new("FIN retx test".to_string(), &cfg);

        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(&AckReceived::new(WrappingInt32::new(isn.raw_value() + 1)));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_ACKED.to_string(),
        ));
        test.execute(&Close {});
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::FIN_SENT.to_string(),
        ));
        test.execute(
            ExpectSegment::new()
                .with_fin(true)
                .with_seqno(WrappingInt32::new(isn.raw_value() + 1)),
        );
        test.execute(&AckReceived::new(WrappingInt32::new(isn.raw_value() + 1)));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::FIN_SENT.to_string(),
        ));
        test.execute(&ExpectBytesInFlight::new(1));
        test.execute(&ExpectNoSegment {});
        test.execute(&Tick::new((TCPConfig::TIMEOUT_DFLT - 1) as SizeT));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::FIN_SENT.to_string(),
        ));
        test.execute(&ExpectBytesInFlight::new(1));
        test.execute(&ExpectNoSegment {});
        test.execute(&Tick::new(1));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::FIN_SENT.to_string(),
        ));
        test.execute(&ExpectBytesInFlight::new(1));
        test.execute(
            ExpectSegment::new()
                .with_fin(true)
                .with_seqno(WrappingInt32::new(isn.raw_value() + 1)),
        );
        test.execute(&ExpectNoSegment {});
        test.execute(&Tick::new(1));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::FIN_SENT.to_string(),
        ));
        test.execute(&ExpectBytesInFlight::new(1));
        test.execute(&ExpectNoSegment {});
        test.execute(&AckReceived::new(WrappingInt32::new(isn.raw_value() + 2)));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::FIN_ACKED.to_string(),
        ));
        test.execute(&ExpectBytesInFlight::new(0));
        test.execute(&ExpectNoSegment {});
    }
}
