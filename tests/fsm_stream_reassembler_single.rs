use crate::fsm_stream_reassembler_harness::*;

mod fsm_stream_reassembler_harness;

#[test]
fn fsm_stream_reassembler_single() {
    {
        let mut test = ReassemblerTestHarness::new(65000);

        test.execute(&BytesAssembled::new(0));
        test.execute(&BytesAvailable::new("".to_string()));
        test.execute(&NotAtEof {});
    }

    {
        let mut test = ReassemblerTestHarness::new(65000);

        test.execute(&SubmitSegment::new(String::from("a"), 0, false));

        test.execute(&BytesAssembled::new(1));
        test.execute(&BytesAvailable::new("a".to_string()));
        test.execute(&NotAtEof {});
    }

    {
        let mut test = ReassemblerTestHarness::new(65000);

        test.execute(SubmitSegment::new(String::from("a"), 0, false).with_eof(true));

        test.execute(&BytesAssembled::new(1));
        test.execute(&BytesAvailable::new("a".to_string()));
        test.execute(&AtEof {});
    }

    {
        let mut test = ReassemblerTestHarness::new(65000);

        test.execute(SubmitSegment::new(String::from(""), 0, false).with_eof(true));

        test.execute(&BytesAssembled::new(0));
        test.execute(&BytesAvailable::new("".to_string()));
        test.execute(&AtEof {});
    }

    {
        let mut test = ReassemblerTestHarness::new(65000);

        test.execute(SubmitSegment::new(String::from("b"), 0, false).with_eof(true));

        test.execute(&BytesAssembled::new(1));
        test.execute(&BytesAvailable::new("b".to_string()));
        test.execute(&AtEof {});
    }

    {
        let mut test = ReassemblerTestHarness::new(65000);

        test.execute(&SubmitSegment::new(String::from(""), 0, false));

        test.execute(&BytesAssembled::new(0));
        test.execute(&BytesAvailable::new("".to_string()));
        test.execute(&NotAtEof {});
    }

    {
        let mut test = ReassemblerTestHarness::new(8);

        test.execute(&SubmitSegment::new(String::from("abcdefgh"), 0, false));

        test.execute(&BytesAssembled::new(8));
        test.execute(&BytesAvailable::new("abcdefgh".to_string()));
        test.execute(&NotAtEof {});
    }

    {
        let mut test = ReassemblerTestHarness::new(8);

        test.execute(SubmitSegment::new(String::from("abcdefgh"), 0, false).with_eof(true));

        test.execute(&BytesAssembled::new(8));
        test.execute(&BytesAvailable::new("abcdefgh".to_string()));
        test.execute(&AtEof {});
    }

    {
        let mut test = ReassemblerTestHarness::new(8);

        test.execute(&SubmitSegment::new(String::from("abc"), 0, false));
        test.execute(&BytesAssembled::new(3));

        test.execute(SubmitSegment::new(String::from("bcdefgh"), 1, false).with_eof(true));

        test.execute(&BytesAssembled::new(8));
        test.execute(&BytesAvailable::new("abcdefgh".to_string()));
        test.execute(&AtEof {});
    }

    {
        let mut test = ReassemblerTestHarness::new(8);

        test.execute(&SubmitSegment::new(String::from("abc"), 0, false));
        test.execute(&BytesAssembled::new(3));
        test.execute(&NotAtEof {});

        test.execute(SubmitSegment::new(String::from("ghX"), 6, false).with_eof(true));

        test.execute(&BytesAssembled::new(3));
        test.execute(&NotAtEof {});

        test.execute(&SubmitSegment::new(String::from("cdefg"), 2, false));

        test.execute(&BytesAssembled::new(8));
        test.execute(&BytesAvailable::new("abcdefgh".to_string()));
        test.execute(&NotAtEof {});
    }

    // credit for test: Bill Lin (2020)
    {
        let mut test = ReassemblerTestHarness::new(8);

        test.execute(&SubmitSegment::new(String::from("abc"), 0, false));
        test.execute(&BytesAssembled::new(3));
        test.execute(&NotAtEof {});

        // Stream re-assembler should ignore empty segments
        test.execute(&SubmitSegment::new(String::from(""), 6, false));
        test.execute(&BytesAssembled::new(3));
        test.execute(&NotAtEof {});

        test.execute(SubmitSegment::new(String::from("de"), 3, false).with_eof(true));
        test.execute(&BytesAssembled::new(5));
        test.execute(&BytesAvailable::new("abcde".to_string()));
        test.execute(&AtEof {});
    }
}
