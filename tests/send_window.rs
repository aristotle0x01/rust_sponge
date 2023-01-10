use crate::sender_harness::{
    AckReceived, Close, ExpectNoSegment, ExpectSegment, TCPSenderTestHarness, WriteBytes,
};
use rand::thread_rng;
use rust_sponge::tcp_helpers::tcp_config::TCPConfig;
use rust_sponge::wrapping_integers::WrappingInt32;
use rust_sponge::SizeT;

mod sender_harness;

#[test]
fn t_send_window() {
    use rand::Rng;

    let mut rd = thread_rng();

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);

        let mut test = TCPSenderTestHarness::new(
            "Initial receiver advertised window is respected".to_string(),
            &cfg,
        );
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(AckReceived::new(isn + 1).with_win(4));
        test.execute(&ExpectNoSegment {});
        test.execute(&WriteBytes::new("abcdefg".to_string()));
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_data("abcd".to_string()),
        );
        test.execute(&ExpectNoSegment {});
    }

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);

        let mut test = TCPSenderTestHarness::new("Immediate window is respected".to_string(), &cfg);
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(AckReceived::new(isn + 1).with_win(6));
        test.execute(&ExpectNoSegment {});
        test.execute(&WriteBytes::new("abcdefg".to_string()));
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_data("abcdef".to_string()),
        );
        test.execute(&ExpectNoSegment {});
    }

    {
        let min_win: SizeT = 5;
        let max_win: SizeT = 100;
        let n_reps: SizeT = 1000;

        for _i in 0..n_reps {
            let len = min_win + (rd.gen_range(0..u32::MAX) % (max_win - min_win) as u32) as SizeT;
            let mut cfg = TCPConfig::default();
            let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
            cfg.fixed_isn = Option::from(isn);

            let mut test = TCPSenderTestHarness::new(format!("Window {}", _i).to_string(), &cfg);
            test.execute(
                ExpectSegment::new()
                    .with_no_flags()
                    .with_syn(true)
                    .with_payload_size(0)
                    .with_seqno(isn),
            );
            test.execute(AckReceived::new(isn + 1).with_win(len as u16));
            test.execute(&ExpectNoSegment {});
            test.execute(&WriteBytes::new(
                String::from_utf8(vec![b'a'; 2 * n_reps]).unwrap(),
            ));
            test.execute(ExpectSegment::new().with_no_flags().with_payload_size(len));
            test.execute(&ExpectNoSegment {});
        }
    }

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);

        let mut test = TCPSenderTestHarness::new("Window growth is exploited".to_string(), &cfg);
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(AckReceived::new(isn + 1).with_win(4));
        test.execute(&ExpectNoSegment {});
        test.execute(&WriteBytes::new("0123456789".to_string()));
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_data("0123".to_string()),
        );
        test.execute(AckReceived::new(isn + 5).with_win(5));
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_data("45678".to_string()),
        );
        test.execute(&ExpectNoSegment {});
    }

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);

        let mut test =
            TCPSenderTestHarness::new("FIN flag occupies space in window".to_string(), &cfg);
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(AckReceived::new(isn + 1).with_win(7));
        test.execute(&ExpectNoSegment {});
        test.execute(&WriteBytes::new("1234567".to_string()));
        test.execute(&Close {});
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_data("1234567".to_string()),
        );
        test.execute(&ExpectNoSegment {}); // window is full
        test.execute(AckReceived::new(isn + 8).with_win(1));
        test.execute(
            ExpectSegment::new()
                .with_fin(true)
                .with_data("".to_string()),
        );
        test.execute(&ExpectNoSegment {});
    }

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);

        let mut test = TCPSenderTestHarness::new(
            "FIN flag occupies space in window (part II)".to_string(),
            &cfg,
        );
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(AckReceived::new(isn + 1).with_win(7));
        test.execute(&ExpectNoSegment {});
        test.execute(&WriteBytes::new("1234567".to_string()));
        test.execute(&Close {});
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_data("1234567".to_string()),
        );
        test.execute(&ExpectNoSegment {}); // window is full
        test.execute(AckReceived::new(isn + 1).with_win(8));
        test.execute(
            ExpectSegment::new()
                .with_fin(true)
                .with_data("".to_string()),
        );
        test.execute(&ExpectNoSegment {});
    }

    {
        let mut cfg = TCPConfig::default();
        let isn = WrappingInt32::new(rd.gen_range(0..u32::MAX));
        cfg.fixed_isn = Option::from(isn);

        let mut test = TCPSenderTestHarness::new(
            "Piggyback FIN in segment when space is available".to_string(),
            &cfg,
        );
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_syn(true)
                .with_payload_size(0)
                .with_seqno(isn),
        );
        test.execute(AckReceived::new(isn + 1).with_win(3));
        test.execute(&ExpectNoSegment {});
        test.execute(&WriteBytes::new("1234567".to_string()));
        test.execute(&Close {});
        test.execute(
            ExpectSegment::new()
                .with_no_flags()
                .with_data("123".to_string()),
        );
        test.execute(&ExpectNoSegment {}); // window is full
        test.execute(AckReceived::new(isn + 1).with_win(8));
        test.execute(
            ExpectSegment::new()
                .with_fin(true)
                .with_data("4567".to_string()),
        );
        test.execute(&ExpectNoSegment {});
    }
}
