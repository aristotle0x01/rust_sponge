use crate::fsm_stream_reassembler_harness::*;

mod fsm_stream_reassembler_harness;

#[test]
fn fsm_stream_reassembler_seq() {
    {
        let mut test = ReassemblerTestHarness::new(65000);

        test.execute(&SubmitSegment::new(String::from("abcd"), 0, false));
        test.execute(&BytesAssembled::new(4));
        test.execute(&BytesAvailable::new("abcd".to_string()));
        test.execute(&NotAtEof {});

        test.execute(&SubmitSegment::new(String::from("efgh"), 4, false));
        test.execute(&BytesAssembled::new(8));
        test.execute(&BytesAvailable::new("efgh".to_string()));
        test.execute(&NotAtEof {});
    }

    {
        let mut test = ReassemblerTestHarness::new(65000);

        test.execute(&SubmitSegment::new(String::from("abcd"), 0, false));
        test.execute(&BytesAssembled::new(4));
        test.execute(&NotAtEof {});
        test.execute(&SubmitSegment::new(String::from("efgh"), 4, false));
        test.execute(&BytesAssembled::new(8));

        test.execute(&BytesAvailable::new("abcdefgh".to_string()));
        test.execute(&NotAtEof {});
    }

    {
        let mut test = ReassemblerTestHarness::new(65000);

        let mut s = String::new();
        for _i in 0..100 {
            test.execute(&BytesAssembled::new(4 * _i));
            test.execute(&SubmitSegment::new(String::from("abcd"), 4 * _i, false));
            test.execute(&NotAtEof {});
            s.push_str("abcd");
        }
        test.execute(&BytesAvailable::new(s));
        test.execute(&NotAtEof {});
    }

    {
        let mut test = ReassemblerTestHarness::new(65000);

        for _i in 0..100 {
            test.execute(&BytesAssembled::new(4 * _i));
            test.execute(&SubmitSegment::new(String::from("abcd"), 4 * _i, false));
            test.execute(&NotAtEof {});

            test.execute(&BytesAvailable::new("abcd".to_string()));
        }
    }
}
