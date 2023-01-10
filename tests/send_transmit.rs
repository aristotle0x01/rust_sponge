use crate::sender_harness::{
    AckReceived, ExpectBytesInFlight, ExpectNoSegment, ExpectSegment, ExpectSeqno, ExpectState,
    TCPSenderTestHarness, WriteBytes,
};
use rand::thread_rng;
use rust_sponge::tcp_helpers::tcp_config::TCPConfig;
use rust_sponge::tcp_helpers::tcp_state::TCPSenderStateSummary;
use rust_sponge::wrapping_integers::WrappingInt32;
use rust_sponge::SizeT;

mod sender_harness;

#[test]
fn t_send_transmit() {
    use rand::Rng;

    let mut rd = thread_rng();

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);

        let mut test = TCPSenderTestHarness::new("Three short writes".to_string(), &cfg);
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(&AckReceived::new(isn + 1));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_ACKED.to_string(),
        ));
        test.execute(&WriteBytes::new("ab".to_string()));
        test.execute(
            ExpectSegment::new()
                .with_data("ab".to_string())
                .with_seqno(isn + 1),
        );
        test.execute(&WriteBytes::new("cd".to_string()));
        test.execute(
            ExpectSegment::new()
                .with_data("cd".to_string())
                .with_seqno(isn + 3),
        );
        test.execute(&WriteBytes::new("abcd".to_string()));
        test.execute(
            ExpectSegment::new()
                .with_data("abcd".to_string())
                .with_seqno(isn + 5),
        );
        test.execute(&ExpectSeqno::new(isn + 9));
        test.execute(&ExpectBytesInFlight::new(8));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_ACKED.to_string(),
        ));
    }

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);

        let mut test =
            TCPSenderTestHarness::new("Many short writes, continuous acks".to_string(), &cfg);
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(&AckReceived::new(isn + 1));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_ACKED.to_string(),
        ));
        let max_block_size: u32 = 10;
        let n_rounds: u32 = 10000;
        let mut bytes_sent = 0;
        for _i in 0..n_rounds {
            let mut data = String::new();
            let block_size = rd.gen_range(1..max_block_size);
            for _j in 0..block_size {
                // 'a' == 97
                let c = 97 + ((_i + _j) % 26);
                data.push(char::from_u32(c).unwrap())
            }
            test.execute(&ExpectSeqno::new(isn + bytes_sent + 1));
            test.execute(&WriteBytes::new(data.to_string()));
            bytes_sent += block_size;
            test.execute(&ExpectBytesInFlight::new(block_size as SizeT));
            test.execute(
                ExpectSegment::new()
                    .with_seqno(isn + 1 + bytes_sent - block_size)
                    .with_data(data.to_string()),
            );
            test.execute(&ExpectNoSegment {});
            test.execute(&AckReceived::new(isn + 1 + bytes_sent));
        }
    }

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);

        let mut test = TCPSenderTestHarness::new("Many short writes, ack at end".to_string(), &cfg);
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(AckReceived::new(isn + 1).with_win(65000));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_ACKED.to_string(),
        ));
        let max_block_size: u32 = 10;
        let n_rounds: u32 = 1000;
        let mut bytes_sent = 0;
        for _i in 0..n_rounds {
            let mut data = String::new();
            let block_size = rd.gen_range(1..max_block_size);
            for _j in 0..block_size {
                // 'a' == 97
                let c = 97 + ((_i + _j) % 26);
                data.push(char::from_u32(c).unwrap())
            }
            test.execute(&ExpectSeqno::new(isn + bytes_sent + 1));
            test.execute(&WriteBytes::new(data.to_string()));
            bytes_sent += block_size;
            test.execute(&ExpectBytesInFlight::new(bytes_sent as SizeT));
            test.execute(
                ExpectSegment::new()
                    .with_seqno(isn + 1 + bytes_sent - block_size)
                    .with_data(data.to_string()),
            );
            test.execute(&ExpectNoSegment {});
        }
        test.execute(&ExpectBytesInFlight::new(bytes_sent as SizeT));
        test.execute(&AckReceived::new(isn + 1 + bytes_sent));
        test.execute(&ExpectBytesInFlight::new(0));
    }

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);

        let mut test = TCPSenderTestHarness::new("Window filling".to_string(), &cfg);
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(AckReceived::new(isn + 1).with_win(3));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_ACKED.to_string(),
        ));
        test.execute(&WriteBytes::new("01234567".to_string()));
        test.execute(&ExpectBytesInFlight::new(3));
        test.execute(ExpectSegment::new().with_data("012".to_string()));
        test.execute(&ExpectNoSegment {});
        test.execute(&ExpectSeqno::new(isn + 1 + 3));
        test.execute(AckReceived::new(isn + 1 + 3).with_win(3));
        test.execute(&ExpectBytesInFlight::new(3));
        test.execute(ExpectSegment::new().with_data("345".to_string()));
        test.execute(&ExpectNoSegment {});
        test.execute(&ExpectSeqno::new(isn + 1 + 6));
        test.execute(AckReceived::new(isn + 1 + 6).with_win(3));
        test.execute(&ExpectBytesInFlight::new(2));
        test.execute(ExpectSegment::new().with_data("67".to_string()));
        test.execute(&ExpectNoSegment {});
        test.execute(&ExpectSeqno::new(isn + 1 + 8));
        test.execute(AckReceived::new(isn + 1 + 8).with_win(3));
        test.execute(&ExpectBytesInFlight::new(0));
        test.execute(&ExpectNoSegment {});
    }

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);

        let mut test =
            TCPSenderTestHarness::new("Immediate writes respect the window".to_string(), &cfg);
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(AckReceived::new(isn + 1).with_win(3));
        test.execute(&ExpectState::new(
            TCPSenderStateSummary::SYN_ACKED.to_string(),
        ));
        test.execute(&WriteBytes::new("01".to_string()));
        test.execute(&ExpectBytesInFlight::new(2));
        test.execute(ExpectSegment::new().with_data("01".to_string()));
        test.execute(&ExpectNoSegment {});
        test.execute(&ExpectSeqno::new(isn + 1 + 2));
        test.execute(&WriteBytes::new("23".to_string()));
        test.execute(&ExpectBytesInFlight::new(3));
        test.execute(ExpectSegment::new().with_data("2".to_string()));
        test.execute(&ExpectNoSegment {});
        test.execute(&ExpectSeqno::new(isn + 1 + 3));
    }
}
