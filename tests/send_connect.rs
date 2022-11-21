use crate::sender_harness::{
    AckReceived, ExpectBytesInFlight, ExpectNoSegment, ExpectSegment, ExpectSeqno, ExpectState,
    TCPSenderTestHarness, Tick, WriteBytes,
};
use rand::thread_rng;
use rust_sponge::tcp_helpers::tcp_config::TCPConfig;
use rust_sponge::tcp_helpers::tcp_state::TCPSenderStateSummary;
use rust_sponge::wrapping_integers::WrappingInt32;

mod sender_harness;

#[test]
fn t_send_connect() {
    use rand::Rng;

    let mut rd = thread_rng();

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);

        let mut test = TCPSenderTestHarness::new("SYN sent test".to_string(), &cfg);
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_SENT.to_string(),
        ));
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(&ExpectBytesInFlight::new(1));
    }

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);

        let mut test = TCPSenderTestHarness::new("SYN acked test".to_string(), &cfg);
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_SENT.to_string(),
        ));
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(&ExpectBytesInFlight::new(1));
        test.execute(&AckReceived::new(WrappingInt32::new(isn.raw_value() + 1)));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_ACKED.to_string(),
        ));
        test.execute(&ExpectNoSegment {});
        test.execute(&ExpectBytesInFlight::new(0));
    }

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);

        let mut test = TCPSenderTestHarness::new("SYN -> wrong ack test".to_string(), &cfg);
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_SENT.to_string(),
        ));
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(&ExpectBytesInFlight::new(1));
        test.execute(&AckReceived::new(isn));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_SENT.to_string(),
        ));
        test.execute(&ExpectNoSegment {});
        test.execute(&ExpectBytesInFlight::new(1));
    }

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);

        let mut test = TCPSenderTestHarness::new("SYN acked, data".to_string(), &cfg);
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_SENT.to_string(),
        ));
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(&ExpectBytesInFlight::new(1));
        test.execute(&AckReceived::new(WrappingInt32::new(isn.raw_value() + 1)));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_ACKED.to_string(),
        ));
        test.execute(&ExpectNoSegment {});
        test.execute(&ExpectBytesInFlight::new(0));
        test.execute(&WriteBytes::new("abcdefgh".to_string()));
        test.execute(&Tick::new(1));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_ACKED.to_string(),
        ));
        test.execute(
            ExpectSegment::new()
                .with_seqno(WrappingInt32::new(isn.raw_value() + 1))
                .with_data("abcdefgh".to_string()),
        );
        test.execute(&ExpectBytesInFlight::new(8));
        test.execute(&AckReceived::new(WrappingInt32::new(isn.raw_value() + 9)));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_ACKED.to_string(),
        ));
        test.execute(&ExpectNoSegment {});
        test.execute(&ExpectBytesInFlight::new(0));
        test.execute(&ExpectSeqno::new(WrappingInt32::new(isn.raw_value() + 9)));
    }
}
