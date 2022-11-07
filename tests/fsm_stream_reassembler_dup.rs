use crate::fsm_stream_reassembler_harness::*;
use rand::Rng;

mod fsm_stream_reassembler_harness;

#[test]
fn fsm_stream_reassembler_dup() {
    {
        let mut test = ReassemblerTestHarness::new(65000);

        test.execute(&SubmitSegment::new(String::from("abcd"), 0, false));
        test.execute(&BytesAssembled::new(4));
        test.execute(&BytesAvailable::new("abcd".to_string()));
        test.execute(&NotAtEof {});

        test.execute(&SubmitSegment::new(String::from("abcd"), 0, false));
        test.execute(&BytesAssembled::new(4));
        test.execute(&BytesAvailable::new("".to_string()));
        test.execute(&NotAtEof {});
    }

    {
        let mut test = ReassemblerTestHarness::new(65000);

        test.execute(&SubmitSegment::new(String::from("abcd"), 0, false));
        test.execute(&BytesAssembled::new(4));
        test.execute(&BytesAvailable::new("abcd".to_string()));
        test.execute(&NotAtEof {});

        test.execute(&SubmitSegment::new(String::from("abcd"), 4, false));
        test.execute(&BytesAssembled::new(8));
        test.execute(&BytesAvailable::new("abcd".to_string()));
        test.execute(&NotAtEof {});

        test.execute(&SubmitSegment::new(String::from("abcd"), 0, false));
        test.execute(&BytesAssembled::new(8));
        test.execute(&BytesAvailable::new("".to_string()));
        test.execute(&NotAtEof {});

        test.execute(&SubmitSegment::new(String::from("abcd"), 4, false));
        test.execute(&BytesAssembled::new(8));
        test.execute(&BytesAvailable::new("".to_string()));
        test.execute(&NotAtEof {});
    }

    {
        let mut test = ReassemblerTestHarness::new(65000);
        test.execute(&SubmitSegment::new(String::from("abcdefgh"), 0, false));
        test.execute(&BytesAssembled::new(8));
        test.execute(&BytesAvailable::new("abcdefgh".to_string()));
        test.execute(&NotAtEof {});

        let data = "abcdefgh".to_string();
        for _i in 0..1000 {
            let start_i = rand::thread_rng().gen_range(1..8);
            let end_i = rand::thread_rng().gen_range(start_i..8);
            test.execute(&SubmitSegment::new(
                data[start_i..(end_i + 1)].to_string(),
                start_i,
                false,
            ));
            test.execute(&BytesAssembled::new(8));
            test.execute(&BytesAvailable::new("".to_string()));
            test.execute(&NotAtEof {});
        }
    }

    {
        let mut test = ReassemblerTestHarness::new(65000);

        test.execute(&SubmitSegment::new(String::from("abcd"), 0, false));
        test.execute(&BytesAssembled::new(4));
        test.execute(&BytesAvailable::new("abcd".to_string()));
        test.execute(&NotAtEof {});

        test.execute(&SubmitSegment::new(String::from("abcdef"), 0, false));
        test.execute(&BytesAssembled::new(6));
        test.execute(&BytesAvailable::new("ef".to_string()));
        test.execute(&NotAtEof {});
    }
}
