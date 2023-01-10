use crate::receiver_harness::{
    ExpectAckno, ExpectBytes, ExpectEof, ExpectInputNotEnded, ExpectState,
    ExpectTotalAssembledBytes, ExpectUnassembledBytes, SegmentArrives, TCPReceiverTestHarness,
};
use rand::thread_rng;
use rust_sponge::tcp_helpers::tcp_state::TCPReceiverStateSummary;
use rust_sponge::wrapping_integers::WrappingInt32;

mod receiver_harness;

#[test]
fn t_recv_special() {
    use rand::Rng;

    let mut rd = thread_rng();

    /* segment before SYN */
    {
        let isn: u32 = rd.gen_range(0..u32::MAX);
        let mut test = TCPReceiverTestHarness::new(4000);
        test.execute(&ExpectState::new(
            TCPReceiverStateSummary::LISTEN.to_string(),
        ));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_seqno_u32(isn + 1)
                .with_data("hello".to_string())
                .with_result(receiver_harness::Result::NotSyn),
        );
        test.execute(&ExpectState::new(
            TCPReceiverStateSummary::LISTEN.to_string(),
        ));
        test.execute(&ExpectUnassembledBytes::new(0));
        test.execute(&ExpectBytes::new("".to_string()));
        test.execute(&ExpectTotalAssembledBytes::new(0));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_syn()
                .with_seqno_u32(isn)
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectState::new(
            TCPReceiverStateSummary::SYN_RECV.to_string(),
        ));
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(isn + 1))));
    }

    /* segment with SYN + data */
    {
        let isn: u32 = rd.gen_range(0..u32::MAX);
        let mut test = TCPReceiverTestHarness::new(4000);
        test.execute(&ExpectState::new(
            TCPReceiverStateSummary::LISTEN.to_string(),
        ));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_syn()
                .with_seqno_u32(isn)
                .with_data("Hello, CS144!".to_string())
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectState::new(
            TCPReceiverStateSummary::SYN_RECV.to_string(),
        ));
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(
            isn + 14,
        ))));
        test.execute(&ExpectUnassembledBytes::new(0));
        test.execute(&ExpectBytes::new("Hello, CS144!".to_string()));
    }

    /* empty segment */
    {
        let isn: u32 = rd.gen_range(0..u32::MAX);
        let mut test = TCPReceiverTestHarness::new(4000);
        test.execute(&ExpectState::new(
            TCPReceiverStateSummary::LISTEN.to_string(),
        ));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_syn()
                .with_seqno_u32(isn)
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectState::new(
            TCPReceiverStateSummary::SYN_RECV.to_string(),
        ));
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(isn + 1))));
        test.execute(&ExpectUnassembledBytes::new(0));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_syn()
                .with_seqno_u32(isn + 1)
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectUnassembledBytes::new(0));
        test.execute(&ExpectTotalAssembledBytes::new(0));
        test.execute(&ExpectInputNotEnded::new());
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_syn()
                .with_seqno_u32(isn + 5)
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectUnassembledBytes::new(0));
        test.execute(&ExpectTotalAssembledBytes::new(0));
        test.execute(&ExpectInputNotEnded::new());
    }

    /* segment with null byte */
    {
        let isn: u32 = rd.gen_range(0..u32::MAX);
        let mut test = TCPReceiverTestHarness::new(4000);
        let text = format!("Here's a null byte:{}and it's gone.", '\0');
        test.execute(&ExpectState::new(
            TCPReceiverStateSummary::LISTEN.to_string(),
        ));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_syn()
                .with_seqno_u32(isn)
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectState::new(
            TCPReceiverStateSummary::SYN_RECV.to_string(),
        ));
        test.execute(&ExpectUnassembledBytes::new(0));
        test.execute(&ExpectTotalAssembledBytes::new(0));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_seqno_u32(isn + 1)
                .with_data(text.to_string())
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectBytes::new(text.to_string()));
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(
            isn + 35,
        ))));
        test.execute(&ExpectInputNotEnded::new());
    }

    /* segment with data + FIN */
    {
        let isn: u32 = rd.gen_range(0..u32::MAX);
        let mut test = TCPReceiverTestHarness::new(4000);
        test.execute(&ExpectState::new(
            TCPReceiverStateSummary::LISTEN.to_string(),
        ));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_syn()
                .with_seqno_u32(isn)
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectState::new(
            TCPReceiverStateSummary::SYN_RECV.to_string(),
        ));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_fin()
                .with_data("Goodbye, CS144!".to_string())
                .with_seqno_u32(isn + 1)
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectState::new(
            TCPReceiverStateSummary::FIN_RECV.to_string(),
        ));
        test.execute(&ExpectBytes::new("Goodbye, CS144!".to_string()));
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(
            isn + 17,
        ))));
        test.execute(&ExpectEof::new());
    }

    /* segment with FIN (but can't be assembled yet) */
    {
        let isn: u32 = rd.gen_range(0..u32::MAX);
        let mut test = TCPReceiverTestHarness::new(4000);
        test.execute(&ExpectState::new(
            TCPReceiverStateSummary::LISTEN.to_string(),
        ));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_syn()
                .with_seqno_u32(isn)
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectState::new(
            TCPReceiverStateSummary::SYN_RECV.to_string(),
        ));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_fin()
                .with_data("oodbye, CS144!".to_string())
                .with_seqno_u32(isn + 2)
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectState::new(
            TCPReceiverStateSummary::SYN_RECV.to_string(),
        ));
        test.execute(&ExpectBytes::new("".to_string()));
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(isn + 1))));
        test.execute(&ExpectInputNotEnded {});
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_data("G".to_string())
                .with_seqno_u32(isn + 1)
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectState::new(
            TCPReceiverStateSummary::FIN_RECV.to_string(),
        ));
        test.execute(&ExpectBytes::new("Goodbye, CS144!".to_string()));
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(
            isn + 17,
        ))));
        test.execute(&ExpectEof::new());
    }

    /* segment with SYN + data + FIN */
    {
        let isn: u32 = rd.gen_range(0..u32::MAX);
        let mut test = TCPReceiverTestHarness::new(4000);

        test.execute(&ExpectState::new(
            TCPReceiverStateSummary::LISTEN.to_string(),
        ));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_syn()
                .with_seqno_u32(isn)
                .with_data("Hello and goodbye, CS144!".to_string())
                .with_fin()
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectState::new(
            TCPReceiverStateSummary::FIN_RECV.to_string(),
        ));
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(
            isn + 27,
        ))));
        test.execute(&ExpectUnassembledBytes::new(0));
        test.execute(&ExpectBytes::new("Hello and goodbye, CS144!".to_string()));
        test.execute(&ExpectEof::new());
    }
}
