use crate::sender_harness::{
    AckReceived, ExpectNoSegment, ExpectSegment, ExpectState, TCPSenderTestHarness, WriteBytes,
};
use rand::thread_rng;
use rust_sponge::tcp_helpers::tcp_config::TCPConfig;
use rust_sponge::tcp_helpers::tcp_state::TCPSenderStateSummary;
use rust_sponge::wrapping_integers::WrappingInt32;

mod sender_harness;

#[test]
fn t_send_ack() {
    use rand::Rng;

    let mut rd = thread_rng();

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);

        let mut test = TCPSenderTestHarness::new("Repeat ACK is ignored".to_string(), &cfg);
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(&ExpectNoSegment {});
        test.execute(&AckReceived::new(WrappingInt32::new(isn.raw_value() + 1)));
        test.execute(&WriteBytes::new("a".to_string()));
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_data("a".to_string()),
        );
        test.execute(&ExpectNoSegment {});
        test.execute(&AckReceived::new(WrappingInt32::new(isn.raw_value() + 1)));
        test.execute(&ExpectNoSegment {});
    }

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);

        let mut test = TCPSenderTestHarness::new("Old ACK is ignored".to_string(), &cfg);
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(&ExpectNoSegment {});
        test.execute(&AckReceived::new(WrappingInt32::new(isn.raw_value() + 1)));
        test.execute(&WriteBytes::new("a".to_string()));
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_data("a".to_string()),
        );
        test.execute(&ExpectNoSegment {});
        test.execute(&AckReceived::new(WrappingInt32::new(isn.raw_value() + 2)));
        test.execute(&ExpectNoSegment {});
        test.execute(&WriteBytes::new("b".to_string()));
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_data("b".to_string()),
        );
        test.execute(&ExpectNoSegment {});
        test.execute(&AckReceived::new(WrappingInt32::new(isn.raw_value() + 1)));
        test.execute(&ExpectNoSegment {});
    }

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);

        let mut test = TCPSenderTestHarness::new(
            "Impossible ackno (beyond next seqno) is ignored".to_string(),
            &cfg,
        );

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
        test.execute(AckReceived::new(WrappingInt32::new(isn.raw_value() + 2)).with_win(1000));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_SENT.to_string(),
        ));
    }
}
