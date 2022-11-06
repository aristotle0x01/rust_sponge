use crate::fsm_stream_reassembler_harness::*;

mod fsm_stream_reassembler_harness;

#[test]
fn fsm_stream_reassembler_holes() {
    {
        let mut test = ReassemblerTestHarness::new(65000);

        test.execute(&SubmitSegment::new(String::from("b"), 1, false));

        test.execute(&BytesAssembled::new(0));
        test.execute(&BytesAvailable::new("".to_string()));
        test.execute(&NotAtEof {});
    }

    {
        let mut test = ReassemblerTestHarness::new(65000);

        test.execute(&SubmitSegment::new(String::from("b"), 1, false));
        test.execute(&SubmitSegment::new(String::from("a"), 0, false));

        test.execute(&BytesAssembled::new(2));
        test.execute(&BytesAvailable::new("ab".to_string()));
        test.execute(&NotAtEof {});
    }

    {
        let mut test = ReassemblerTestHarness::new(65000);

        test.execute(SubmitSegment::new(String::from("b"), 1, false).with_eof(true));

        test.execute(&BytesAssembled::new(0));
        test.execute(&BytesAvailable::new("".to_string()));
        test.execute(&NotAtEof {});

        test.execute(&SubmitSegment::new(String::from("a"), 0, false));

        test.execute(&BytesAssembled::new(2));
        test.execute(&BytesAvailable::new("ab".to_string()));
        test.execute(&AtEof {});
    }

    {
        let mut test = ReassemblerTestHarness::new(65000);

        test.execute(&SubmitSegment::new(String::from("b"), 1, false));
        test.execute(&SubmitSegment::new(String::from("ab"), 0, false));

        test.execute(&BytesAssembled::new(2));
        test.execute(&BytesAvailable::new("ab".to_string()));
        test.execute(&NotAtEof {});
    }

    {
        let mut test = ReassemblerTestHarness::new(65000);

        test.execute(&SubmitSegment::new(String::from("b"), 1, false));
        test.execute(&BytesAssembled::new(0));
        test.execute(&BytesAvailable::new("".to_string()));
        test.execute(&NotAtEof {});

        test.execute(&SubmitSegment::new(String::from("d"), 3, false));
        test.execute(&BytesAssembled::new(0));
        test.execute(&BytesAvailable::new("".to_string()));
        test.execute(&NotAtEof {});

        test.execute(&SubmitSegment::new(String::from("c"), 2, false));
        test.execute(&BytesAssembled::new(0));
        test.execute(&BytesAvailable::new("".to_string()));
        test.execute(&NotAtEof {});

        test.execute(&SubmitSegment::new(String::from("a"), 0, false));

        test.execute(&BytesAssembled::new(4));
        test.execute(&BytesAvailable::new("abcd".to_string()));
        test.execute(&NotAtEof {});
    }

    {
        let mut test = ReassemblerTestHarness::new(65000);

        test.execute(&SubmitSegment::new(String::from("b"), 1, false));
        test.execute(&BytesAssembled::new(0));
        test.execute(&BytesAvailable::new("".to_string()));
        test.execute(&NotAtEof {});

        test.execute(&SubmitSegment::new(String::from("d"), 3, false));
        test.execute(&BytesAssembled::new(0));
        test.execute(&BytesAvailable::new("".to_string()));
        test.execute(&NotAtEof {});

        test.execute(&SubmitSegment::new(String::from("abc"), 0, false));

        test.execute(&BytesAssembled::new(4));
        test.execute(&BytesAvailable::new("abcd".to_string()));
        test.execute(&NotAtEof {});
    }

    {
        let mut test = ReassemblerTestHarness::new(65000);

        test.execute(&SubmitSegment::new(String::from("b"), 1, false));
        test.execute(&BytesAssembled::new(0));
        test.execute(&BytesAvailable::new("".to_string()));
        test.execute(&NotAtEof {});

        test.execute(&SubmitSegment::new(String::from("d"), 3, false));
        test.execute(&BytesAssembled::new(0));
        test.execute(&BytesAvailable::new("".to_string()));
        test.execute(&NotAtEof {});

        test.execute(&SubmitSegment::new(String::from("a"), 0, false));
        test.execute(&BytesAssembled::new(2));
        test.execute(&BytesAvailable::new("ab".to_string()));
        test.execute(&NotAtEof {});

        test.execute(&SubmitSegment::new(String::from("c"), 2, false));
        test.execute(&BytesAssembled::new(4));
        test.execute(&BytesAvailable::new("cd".to_string()));
        test.execute(&NotAtEof {});

        test.execute(SubmitSegment::new(String::from(""), 4, false).with_eof(true));
        test.execute(&BytesAssembled::new(4));
        test.execute(&BytesAvailable::new("".to_string()));
        test.execute(&AtEof {});
    }
}
