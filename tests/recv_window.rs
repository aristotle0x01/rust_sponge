use crate::receiver_harness::{
    ExpectAckno, ExpectBytes, ExpectTotalAssembledBytes, ExpectWindow, SegmentArrives,
    TCPReceiverTestHarness,
};
use rust_sponge::wrapping_integers::WrappingInt32;
use rust_sponge::SizeT;

mod receiver_harness;

#[test]
fn t_recv_window() {
    {
        // Window size decreases appropriately
        let cap: SizeT = 4000;
        let isn: u32 = 23452;
        let mut test = TCPReceiverTestHarness::new(cap);

        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_syn()
                .with_seqno_u32(isn)
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(isn + 1))));
        test.execute(&ExpectWindow::new(cap));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_seqno_u32(isn + 1)
                .with_data("abcd".to_string())
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(isn + 5))));
        test.execute(&ExpectWindow::new(cap - 4));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_seqno_u32(isn + 9)
                .with_data("ijkl".to_string())
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(isn + 5))));
        test.execute(&ExpectWindow::new(cap - 4));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_seqno_u32(isn + 5)
                .with_data("efgh".to_string())
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(
            isn + 13,
        ))));
        test.execute(&ExpectWindow::new(cap - 12));
    }

    {
        // Window size expands upon read
        let cap: SizeT = 4000;
        let isn: u32 = 23452;
        let mut test = TCPReceiverTestHarness::new(cap);

        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_syn()
                .with_seqno_u32(isn)
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(isn + 1))));
        test.execute(&ExpectWindow::new(cap));
        test.execute(
            SegmentArrives::new(String::from("".to_string()))
                .with_seqno_u32(isn + 1)
                .with_data("abcd".to_string())
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(isn + 5))));
        test.execute(&ExpectWindow::new(cap - 4));
        test.execute(&ExpectBytes::new("abcd".to_string()));
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(isn + 5))));
        test.execute(&ExpectWindow::new(cap));
    }

    {
        // almost-high-seqno segment is accepted, but only some bytes are kept
        let cap: SizeT = 2;
        let isn: u32 = 23452;
        let mut test = TCPReceiverTestHarness::new(cap);

        test.execute(
            SegmentArrives::new("".to_string())
                .with_syn()
                .with_seqno_u32(isn)
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(
            SegmentArrives::new("".to_string())
                .with_seqno_u32(isn + 2)
                .with_data("bc".to_string())
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectTotalAssembledBytes::new(0));
        test.execute(
            SegmentArrives::new("".to_string())
                .with_seqno_u32(isn + 1)
                .with_data("a".to_string())
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectAckno::new(Option::Some(WrappingInt32::new(isn + 3))));
        test.execute(&ExpectWindow::new(0));
        test.execute(&ExpectTotalAssembledBytes::new(2));
        test.execute(&ExpectBytes::new("ab".to_string()));
        test.execute(&ExpectWindow::new(2));
    }

    {
        // almost-low-seqno segment is accepted
        let cap: SizeT = 4;
        let isn: u32 = 294058;
        let mut test = TCPReceiverTestHarness::new(cap);

        test.execute(
            SegmentArrives::new("".to_string())
                .with_syn()
                .with_seqno_u32(isn)
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(
            SegmentArrives::new("".to_string())
                .with_data("ab".to_string())
                .with_seqno_u32(isn + 1)
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectTotalAssembledBytes::new(2));
        test.execute(&ExpectWindow::new(cap - 2));
        test.execute(
            SegmentArrives::new("".to_string())
                .with_data("abc".to_string())
                .with_seqno_u32(isn + 1)
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectTotalAssembledBytes::new(3));
        test.execute(&ExpectWindow::new(cap - 3));
    }

    {
        // Segment overflowing the window on left side is acceptable.
        let cap: SizeT = 4;
        let isn: u32 = 23452;
        let mut test = TCPReceiverTestHarness::new(cap);

        test.execute(
            SegmentArrives::new("".to_string())
                .with_syn()
                .with_seqno_u32(isn)
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(
            SegmentArrives::new("".to_string())
                .with_seqno_u32(isn + 1)
                .with_data("ab".to_string())
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(
            SegmentArrives::new("".to_string())
                .with_seqno_u32(isn + 3)
                .with_data("cdef".to_string())
                .with_result(receiver_harness::Result::OK),
        );
    }

    {
        // Segment matching the window is acceptable.
        let cap: SizeT = 4;
        let isn: u32 = 23452;
        let mut test = TCPReceiverTestHarness::new(cap);

        test.execute(
            SegmentArrives::new("".to_string())
                .with_syn()
                .with_seqno_u32(isn)
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(
            SegmentArrives::new("".to_string())
                .with_seqno_u32(isn + 1)
                .with_data("ab".to_string())
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(
            SegmentArrives::new("".to_string())
                .with_seqno_u32(isn + 3)
                .with_data("cd".to_string())
                .with_result(receiver_harness::Result::OK),
        );
    }

    // credit for test: Jared Wasserman
    {
        // A byte with invalid stream index should be ignored
        let cap: SizeT = 4;
        let isn: u32 = 23452;
        let mut test = TCPReceiverTestHarness::new(cap);

        test.execute(
            SegmentArrives::new("".to_string())
                .with_syn()
                .with_seqno_u32(isn)
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(
            SegmentArrives::new("".to_string())
                .with_seqno_u32(isn)
                .with_data("a".to_string())
                .with_result(receiver_harness::Result::OK),
        );
        test.execute(&ExpectTotalAssembledBytes::new(0));
    }
}
