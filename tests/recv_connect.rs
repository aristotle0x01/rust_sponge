use crate::receiver_harness::{
    ExpectAckno, ExpectState, ExpectTotalAssembledBytes, ExpectUnassembledBytes, ExpectWindow,
    SegmentArrives, TCPReceiverTestHarness,
};
use rust_sponge::tcp_helpers::tcp_state::TCPReceiverStateSummary;
use rust_sponge::wrapping_integers::WrappingInt32;
use rust_sponge::SizeT;

mod receiver_harness;

#[test]
fn t_recv_connect() {
    {
        let mut test = TCPReceiverTestHarness::new(4000);
        test.execute(&ExpectWindow::new(4000));
        test.execute(&ExpectAckno::new(Option::None));
        test.execute(&ExpectUnassembledBytes::new(0));
        test.execute(&ExpectTotalAssembledBytes::new(0));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_syn()
                .with_seqno_u32(0)
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(1))));
        test.execute(&ExpectUnassembledBytes::new(0));
        test.execute(&ExpectTotalAssembledBytes::new(0));
    }

    {
        let mut test = TCPReceiverTestHarness::new(5435);
        test.execute(&ExpectAckno::new(Option::None));
        test.execute(&ExpectUnassembledBytes::new(0));
        test.execute(&ExpectTotalAssembledBytes::new(0));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_syn()
                .with_seqno_u32(89347598)
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(
            89347599,
        ))));
        test.execute(&ExpectUnassembledBytes::new(0));
        test.execute(&ExpectTotalAssembledBytes::new(0));
    }

    {
        let mut test = TCPReceiverTestHarness::new(5435);
        test.execute(&ExpectAckno::new(Option::None));
        test.execute(&ExpectUnassembledBytes::new(0));
        test.execute(&ExpectTotalAssembledBytes::new(0));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_seqno_u32(893475)
                .with_result(receiver_harness::Result::NotSyn),
        );
        test.execute(&ExpectAckno::new(Option::None));
        test.execute(&ExpectUnassembledBytes::new(0));
        test.execute(&ExpectTotalAssembledBytes::new(0));
    }

    {
        let mut test = TCPReceiverTestHarness::new(5435);
        test.execute(&ExpectAckno::new(Option::None));
        test.execute(&ExpectUnassembledBytes::new(0));
        test.execute(&ExpectTotalAssembledBytes::new(0));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_ack_u32(0)
                .with_fin()
                .with_seqno_u32(893475)
                .with_result(receiver_harness::Result::NotSyn),
        );
        test.execute(&ExpectAckno::new(Option::None));
        test.execute(&ExpectUnassembledBytes::new(0));
        test.execute(&ExpectTotalAssembledBytes::new(0));
    }

    {
        let mut test = TCPReceiverTestHarness::new(5435);
        test.execute(&ExpectAckno::new(Option::None));
        test.execute(&ExpectUnassembledBytes::new(0));
        test.execute(&ExpectTotalAssembledBytes::new(0));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_ack_u32(0)
                .with_fin()
                .with_seqno_u32(893475)
                .with_result(receiver_harness::Result::NotSyn),
        );
        test.execute(&ExpectAckno::new(Option::None));
        test.execute(&ExpectUnassembledBytes::new(0));
        test.execute(&ExpectTotalAssembledBytes::new(0));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_syn()
                .with_seqno_u32(89347598)
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(
            89347599,
        ))));
        test.execute(&ExpectUnassembledBytes::new(0));
        test.execute(&ExpectTotalAssembledBytes::new(0));
    }

    {
        let mut test = TCPReceiverTestHarness::new(4000);
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_syn()
                .with_seqno_u32(5)
                .with_fin()
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectState::new(
            TCPReceiverStateSummary::FIN_RECV.to_string(),
        ));
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(7))));
        test.execute(&ExpectUnassembledBytes::new(0));
        test.execute(&ExpectTotalAssembledBytes::new(0));
    }

    {
        // Window overflow
        let cap: SizeT = u16::MAX as SizeT + 5;
        let mut test = TCPReceiverTestHarness::new(cap);
        test.execute(&ExpectWindow::new(cap));
    }
}
