use crate::receiver_harness::{
    ExpectAckno, ExpectBytes, ExpectTotalAssembledBytes, ExpectUnassembledBytes, SegmentArrives,
    TCPReceiverTestHarness,
};
use rand::{thread_rng, Rng};
use rust_sponge::wrapping_integers::WrappingInt32;
use rust_sponge::SizeT;

mod receiver_harness;

#[test]
fn t_recv_transmit() {
    let mut rd = thread_rng();

    {
        let mut test = TCPReceiverTestHarness::new(4000);
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_syn()
                .with_seqno_u32(0)
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_seqno_u32(1)
                .with_data("abcd".to_string())
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(5))));
        test.execute(&ExpectBytes::new("abcd".to_string()));
        test.execute(&ExpectUnassembledBytes::new(0));
        test.execute(&ExpectTotalAssembledBytes::new(4));
    }

    {
        let isn: u32 = 384678;
        let mut test = TCPReceiverTestHarness::new(4000);
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_syn()
                .with_seqno_u32(isn)
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_seqno_u32(isn + 1)
                .with_data("abcd".to_string())
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(isn + 5))));
        test.execute(&ExpectUnassembledBytes::new(0));
        test.execute(&ExpectTotalAssembledBytes::new(4));
        test.execute(&ExpectBytes::new("abcd".to_string()));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_seqno_u32(isn + 5)
                .with_data("efgh".to_string())
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(isn + 9))));
        test.execute(&ExpectUnassembledBytes::new(0));
        test.execute(&ExpectTotalAssembledBytes::new(8));
        test.execute(&ExpectBytes::new("efgh".to_string()));
    }

    {
        let isn: u32 = 5;
        let mut test = TCPReceiverTestHarness::new(4000);
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_syn()
                .with_seqno_u32(isn)
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_seqno_u32(isn + 1)
                .with_data("abcd".to_string())
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(isn + 5))));
        test.execute(&ExpectUnassembledBytes::new(0));
        test.execute(&ExpectTotalAssembledBytes::new(4));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_seqno_u32(isn + 5)
                .with_data("efgh".to_string())
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(isn + 9))));
        test.execute(&ExpectUnassembledBytes::new(0));
        test.execute(&ExpectTotalAssembledBytes::new(8));
        test.execute(&ExpectBytes::new("abcdefgh".to_string()));
    }

    // Many (arrive/read)s
    {
        let max_block_size: u32 = 10;
        let n_rounds: u32 = 10000;
        let mut bytes_sent: SizeT = 0;
        let isn: u32 = 893472;
        let mut test = TCPReceiverTestHarness::new(4000);
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_syn()
                .with_seqno_u32(isn)
                .with_result(receiver_harness::Result::OK),
        );
        for _i in 0..n_rounds {
            let mut data = String::new();
            let block_size = rd.gen_range(1..max_block_size);
            for _j in 0..block_size {
                let c: u8 = (97 + ((_i + _j) % 26)) as u8;
                data.push(char::from(c));
            }
            test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(
                isn + (bytes_sent as u32) + 1,
            ))));
            test.execute(&ExpectTotalAssembledBytes::new(bytes_sent));
            test.execute(
                SegmentArrives::new(String::from("".to_string()))
                    .with_seqno_u32(isn + (bytes_sent as u32) + 1)
                    .with_data(data.to_string())
                    .with_result(receiver_harness::Result::OK),
            );
            bytes_sent += block_size as SizeT;
            test.execute(&ExpectBytes::new(data.to_string()));
        }
    }

    // Many arrives, one read
    {
        let max_block_size: u32 = 10;
        let n_rounds: u32 = 100;
        let mut bytes_sent: SizeT = 0;
        let isn: u32 = 238;
        let mut test = TCPReceiverTestHarness::new(((max_block_size * n_rounds) as u16) as SizeT);
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_syn()
                .with_seqno_u32(isn)
                .with_result(receiver_harness::Result::OK),
        );

        let mut all_data = String::new();
        for _i in 0..n_rounds {
            let mut data = String::new();
            let block_size = rd.gen_range(1..max_block_size);
            for _j in 0..block_size {
                let c: u8 = (97 + ((_i + _j) % 26)) as u8;
                data.push(char::from(c));
                all_data.push(char::from(c));
            }
            test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(
                isn + (bytes_sent as u32) + 1,
            ))));
            test.execute(&ExpectTotalAssembledBytes::new(bytes_sent));
            test.execute(
                SegmentArrives::new(String::from("".to_string()))
                    .with_seqno_u32(isn + (bytes_sent as u32) + 1)
                    .with_data(data)
                    .with_result(receiver_harness::Result::OK),
            );
            bytes_sent += block_size as SizeT;
        }
        test.execute(&ExpectBytes::new(all_data));
    }
}
