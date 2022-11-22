use crate::receiver_harness::{
    ExpectAckno, ExpectBytes, ExpectState, ExpectTotalAssembledBytes, ExpectUnassembledBytes,
    SegmentArrives, TCPReceiverTestHarness,
};
use rand::thread_rng;
use rust_sponge::tcp_helpers::tcp_state::TCPReceiverStateSummary;
use rust_sponge::wrapping_integers::WrappingInt32;

mod receiver_harness;

#[test]
fn t_recv_close() {
    use rand::Rng;

    let mut rd = thread_rng();

    {
        let isn: u32 = rd.gen_range(0..u32::MAX);

        let mut test = TCPReceiverTestHarness::new(4000);
        test.execute(&ExpectState::new(
            TCPReceiverStateSummary::LISTEN.to_string(),
        ));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_syn()
                .with_seqno_u32(isn + 0)
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectState::new(
            TCPReceiverStateSummary::SYN_RECV.to_string(),
        ));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_fin()
                .with_seqno_u32(isn + 1)
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(isn + 2))));
        test.execute(&ExpectUnassembledBytes::new(0));
        test.execute(&ExpectBytes::new(String::from("".to_string())));
        test.execute(&ExpectTotalAssembledBytes::new(0));
        test.execute(&ExpectState::new(
            TCPReceiverStateSummary::FIN_RECV.to_string(),
        ));
    }

    {
        let isn: u32 = rd.gen_range(0..u32::MAX);

        let mut test = TCPReceiverTestHarness::new(4000);
        test.execute(&ExpectState::new(
            TCPReceiverStateSummary::LISTEN.to_string(),
        ));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_syn()
                .with_seqno_u32(isn + 0)
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectState::new(
            TCPReceiverStateSummary::SYN_RECV.to_string(),
        ));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_fin()
                .with_seqno_u32(isn + 1)
                .with_data("a".to_string())
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectState::new(
            TCPReceiverStateSummary::FIN_RECV.to_string(),
        ));
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(isn + 3))));
        test.execute(&ExpectUnassembledBytes::new(0));
        test.execute(&ExpectBytes::new("a".to_string()));
        test.execute(&ExpectTotalAssembledBytes::new(1));
        test.execute(&ExpectState::new(
            TCPReceiverStateSummary::FIN_RECV.to_string(),
        ));
    }
}
