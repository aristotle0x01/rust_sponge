use crate::receiver_harness::{
    ExpectAckno, ExpectBytes, ExpectTotalAssembledBytes, ExpectUnassembledBytes, SegmentArrives,
    TCPReceiverTestHarness,
};
use rand::thread_rng;
use rust_sponge::wrapping_integers::WrappingInt32;

mod receiver_harness;

#[test]
fn t_recv_reorder() {
    use rand::Rng;

    let mut rd = thread_rng();

    // An in-window, but later segment
    {
        let isn: u32 = rd.gen_range(0..u32::MAX);
        let mut test = TCPReceiverTestHarness::new(2358);
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_syn()
                .with_seqno_u32(isn)
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(isn + 1))));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_seqno_u32(isn + 10)
                .with_data("abcd".to_string())
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(isn + 1))));
        test.execute(&ExpectBytes::new("".to_string()));
        test.execute(&ExpectUnassembledBytes::new(4));
        test.execute(&ExpectTotalAssembledBytes::new(0));
    }

    // An in-window, but later segment, then the hole is filled
    {
        let isn: u32 = rd.gen_range(0..u32::MAX);
        let mut test = TCPReceiverTestHarness::new(2358);
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_syn()
                .with_seqno_u32(isn)
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(isn + 1))));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_seqno_u32(isn + 5)
                .with_data("efgh".to_string())
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(isn + 1))));
        test.execute(&ExpectBytes::new("".to_string()));
        test.execute(&ExpectUnassembledBytes::new(4));
        test.execute(&ExpectTotalAssembledBytes::new(0));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_seqno_u32(isn + 1)
                .with_data("abcd".to_string())
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(isn + 9))));
        test.execute(&ExpectBytes::new("abcdefgh".to_string()));
        test.execute(&ExpectUnassembledBytes::new(0));
        test.execute(&ExpectTotalAssembledBytes::new(8));
    }

    // An in-window, but later segment, then the hole is filled, bit by bit
    {
        let isn: u32 = rd.gen_range(0..u32::MAX);
        let mut test = TCPReceiverTestHarness::new(2358);
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_syn()
                .with_seqno_u32(isn)
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(isn + 1))));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_seqno_u32(isn + 5)
                .with_data("efgh".to_string())
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(isn + 1))));
        test.execute(&ExpectBytes::new("".to_string()));
        test.execute(&ExpectUnassembledBytes::new(4));
        test.execute(&ExpectTotalAssembledBytes::new(0));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_seqno_u32(isn + 1)
                .with_data("ab".to_string())
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(isn + 3))));
        test.execute(&ExpectBytes::new("ab".to_string()));
        test.execute(&ExpectUnassembledBytes::new(4));
        test.execute(&ExpectTotalAssembledBytes::new(2));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_seqno_u32(isn + 3)
                .with_data("cd".to_string())
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(isn + 9))));
        test.execute(&ExpectBytes::new("cdefgh".to_string()));
        test.execute(&ExpectUnassembledBytes::new(0));
        test.execute(&ExpectTotalAssembledBytes::new(8));
    }

    // Many gaps, then filled bit by bit.
    {
        let isn: u32 = rd.gen_range(0..u32::MAX);
        let mut test = TCPReceiverTestHarness::new(2358);
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_syn()
                .with_seqno_u32(isn)
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(isn + 1))));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_seqno_u32(isn + 5)
                .with_data("e".to_string())
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(isn + 1))));
        test.execute(&ExpectBytes::new("".to_string()));
        test.execute(&ExpectUnassembledBytes::new(1));
        test.execute(&ExpectTotalAssembledBytes::new(0));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_seqno_u32(isn + 7)
                .with_data("g".to_string())
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(isn + 1))));
        test.execute(&ExpectBytes::new("".to_string()));
        test.execute(&ExpectUnassembledBytes::new(2));
        test.execute(&ExpectTotalAssembledBytes::new(0));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_seqno_u32(isn + 3)
                .with_data("c".to_string())
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(isn + 1))));
        test.execute(&ExpectBytes::new("".to_string()));
        test.execute(&ExpectUnassembledBytes::new(3));
        test.execute(&ExpectTotalAssembledBytes::new(0));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_seqno_u32(isn + 1)
                .with_data("ab".to_string())
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(isn + 4))));
        test.execute(&ExpectBytes::new("abc".to_string()));
        test.execute(&ExpectUnassembledBytes::new(2));
        test.execute(&ExpectTotalAssembledBytes::new(3));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_seqno_u32(isn + 6)
                .with_data("f".to_string())
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectUnassembledBytes::new(3));
        test.execute(&ExpectTotalAssembledBytes::new(3));
        test.execute(&ExpectBytes::new("".to_string()));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_seqno_u32(isn + 4)
                .with_data("d".to_string())
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectUnassembledBytes::new(0));
        test.execute(&ExpectTotalAssembledBytes::new(7));
        test.execute(&ExpectBytes::new("defg".to_string()));
    }

    // Many gaps, then subsumed
    {
        let isn: u32 = rd.gen_range(0..u32::MAX);
        let mut test = TCPReceiverTestHarness::new(2358);
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_syn()
                .with_seqno_u32(isn)
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(isn + 1))));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_seqno_u32(isn + 5)
                .with_data("e".to_string())
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(isn + 1))));
        test.execute(&ExpectBytes::new("".to_string()));
        test.execute(&ExpectUnassembledBytes::new(1));
        test.execute(&ExpectTotalAssembledBytes::new(0));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_seqno_u32(isn + 7)
                .with_data("g".to_string())
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(isn + 1))));
        test.execute(&ExpectBytes::new("".to_string()));
        test.execute(&ExpectUnassembledBytes::new(2));
        test.execute(&ExpectTotalAssembledBytes::new(0));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_seqno_u32(isn + 3)
                .with_data("c".to_string())
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(isn + 1))));
        test.execute(&ExpectBytes::new("".to_string()));
        test.execute(&ExpectUnassembledBytes::new(3));
        test.execute(&ExpectTotalAssembledBytes::new(0));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_seqno_u32(isn + 1)
                .with_data("abcdefgh".to_string())
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(isn + 9))));
        test.execute(&ExpectBytes::new("abcdefgh".to_string()));
        test.execute(&ExpectUnassembledBytes::new(0));
        test.execute(&ExpectTotalAssembledBytes::new(8));
    }
}
