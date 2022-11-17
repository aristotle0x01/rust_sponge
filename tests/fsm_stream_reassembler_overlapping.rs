use crate::fsm_stream_reassembler_harness::*;

mod fsm_stream_reassembler_harness;

#[test]
fn fsm_stream_reassembler_overlapping() {
    {
        // Overlapping assembled (unread) section
        let mut test = ReassemblerTestHarness::new(1000);

        test.execute(&SubmitSegment::new(String::from("a"), 0, false));
        test.execute(&SubmitSegment::new(String::from("ab"), 0, false));

        test.execute(&BytesAssembled::new(2));
        test.execute(&BytesAvailable::new("ab".to_string()));
    }

    {
        // Overlapping assembled (read) section
        let mut test = ReassemblerTestHarness::new(1000);

        test.execute(&SubmitSegment::new(String::from("a"), 0, false));
        test.execute(&BytesAvailable::new("a".to_string()));

        test.execute(&SubmitSegment::new(String::from("ab"), 0, false));
        test.execute(&BytesAvailable::new("b".to_string()));
        test.execute(&BytesAssembled::new(2));
    }

    {
        // Overlapping unassembled section, resulting in assembly
        let mut test = ReassemblerTestHarness::new(1000);

        test.execute(&SubmitSegment::new(String::from("b"), 1, false));
        test.execute(&BytesAvailable::new("".to_string()));

        test.execute(&SubmitSegment::new(String::from("ab"), 0, false));
        test.execute(&BytesAvailable::new("ab".to_string()));
        test.execute(&UnassembledBytes::new(0));
        test.execute(&BytesAssembled::new(2));
    }
    {
        // Overlapping unassembled section, not resulting in assembly
        let mut test = ReassemblerTestHarness::new(1000);

        test.execute(&SubmitSegment::new(String::from("b"), 1, false));
        test.execute(&BytesAvailable::new("".to_string()));

        test.execute(&SubmitSegment::new(String::from("bc"), 1, false));
        test.execute(&BytesAvailable::new("".to_string()));
        test.execute(&UnassembledBytes::new(2));
        test.execute(&BytesAssembled::new(0));
    }
    {
        // Overlapping unassembled section, not resulting in assembly
        let mut test = ReassemblerTestHarness::new(1000);

        test.execute(&SubmitSegment::new(String::from("c"), 2, false));
        test.execute(&BytesAvailable::new("".to_string()));

        test.execute(&SubmitSegment::new(String::from("bcd"), 1, false));
        test.execute(&BytesAvailable::new("".to_string()));
        test.execute(&UnassembledBytes::new(3));
        test.execute(&BytesAssembled::new(0));
    }

    {
        // Overlapping multiple unassembled sections
        let mut test = ReassemblerTestHarness::new(1000);

        test.execute(&SubmitSegment::new(String::from("b"), 1, false));
        test.execute(&SubmitSegment::new(String::from("d"), 3, false));
        test.execute(&BytesAvailable::new("".to_string()));

        test.execute(&SubmitSegment::new(String::from("bcde"), 1, false));
        test.execute(&BytesAvailable::new("".to_string()));
        test.execute(&BytesAssembled::new(0));
        test.execute(&UnassembledBytes::new(4));
    }

    {
        // Submission over existing
        let mut test = ReassemblerTestHarness::new(1000);

        test.execute(&SubmitSegment::new(String::from("c"), 2, false));
        test.execute(&SubmitSegment::new(String::from("bcd"), 1, false));

        test.execute(&BytesAvailable::new("".to_string()));
        test.execute(&BytesAssembled::new(0));
        test.execute(&UnassembledBytes::new(3));

        test.execute(&SubmitSegment::new(String::from("a"), 0, false));
        test.execute(&BytesAvailable::new("abcd".to_string()));
        test.execute(&BytesAssembled::new(4));
        test.execute(&UnassembledBytes::new(0));
    }

    {
        // Submission within existing
        let mut test = ReassemblerTestHarness::new(1000);

        test.execute(&SubmitSegment::new(String::from("bcd"), 1, false));
        test.execute(&SubmitSegment::new(String::from("c"), 2, false));

        test.execute(&BytesAvailable::new("".to_string()));
        test.execute(&BytesAssembled::new(0));
        test.execute(&UnassembledBytes::new(3));

        test.execute(&SubmitSegment::new(String::from("a"), 0, false));
        test.execute(&BytesAvailable::new("abcd".to_string()));
        test.execute(&BytesAssembled::new(4));
        test.execute(&UnassembledBytes::new(0));
    }
}
