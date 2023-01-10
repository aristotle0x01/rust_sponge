use crate::fsm_stream_reassembler_harness::*;
use rust_sponge::SizeT;

mod fsm_stream_reassembler_harness;

#[test]
fn fsm_stream_reassembler_cap() {
    {
        let mut test = ReassemblerTestHarness::new(2);

        test.execute(&SubmitSegment::new(String::from("ab"), 0, false));
        test.execute(&BytesAssembled::new(2));
        test.execute(&BytesAvailable::new("ab".to_string()));

        test.execute(&SubmitSegment::new("cd".to_string(), 2, false));
        test.execute(&BytesAssembled::new(4));
        test.execute(&BytesAvailable::new("cd".to_string()));

        test.execute(&SubmitSegment::new("ef".to_string(), 4, false));
        test.execute(&BytesAssembled::new(6));
        test.execute(&BytesAvailable::new("ef".to_string()));
    }

    {
        let mut test = ReassemblerTestHarness::new(2);

        test.execute(&SubmitSegment::new(String::from("ab"), 0, false));
        test.execute(&BytesAssembled::new(2));

        test.execute(&SubmitSegment::new(String::from("cd"), 2, false));
        test.execute(&BytesAssembled::new(2));

        test.execute(&BytesAvailable::new("ab".to_string()));
        test.execute(&BytesAssembled::new(2));

        test.execute(&SubmitSegment::new(String::from("cd"), 2, false));
        test.execute(&BytesAssembled::new(4));

        test.execute(&BytesAvailable::new("cd".to_string()));
    }

    {
        let mut test = ReassemblerTestHarness::new(2);

        test.execute(&SubmitSegment::new(String::from("bX"), 1, false));
        test.execute(&BytesAssembled::new(0));

        test.execute(&SubmitSegment::new(String::from("a"), 0, false));
        test.execute(&BytesAssembled::new(2));

        test.execute(&BytesAvailable::new("ab".to_string()));
    }

    {
        let mut test = ReassemblerTestHarness::new(1);

        test.execute(&SubmitSegment::new(String::from("ab"), 0, false));
        test.execute(&BytesAssembled::new(1));

        test.execute(&SubmitSegment::new(String::from("ab"), 0, false));
        test.execute(&BytesAssembled::new(1));

        test.execute(&BytesAvailable::new("a".to_string()));
        test.execute(&BytesAssembled::new(1));

        test.execute(&SubmitSegment::new(String::from("abc"), 0, false));
        test.execute(&BytesAssembled::new(2));

        test.execute(&BytesAvailable::new("b".to_string()));
        test.execute(&BytesAssembled::new(2));
    }

    {
        let mut test = ReassemblerTestHarness::new(8);

        test.execute(&SubmitSegment::new(String::from("a"), 0, false));
        test.execute(&BytesAssembled::new(1));
        test.execute(&BytesAvailable::new("a".to_string()));
        test.execute(&NotAtEof {});

        test.execute(&SubmitSegment::new(String::from("bc"), 1, false));
        test.execute(&BytesAssembled::new(3));
        test.execute(&NotAtEof {});

        test.execute(SubmitSegment::new(String::from("ghi"), 6, false).with_eof(true));
        test.execute(&BytesAssembled::new(3));
        test.execute(&NotAtEof {});

        test.execute(&SubmitSegment::new(String::from("cdefg"), 2, false));
        test.execute(&BytesAssembled::new(9));
        test.execute(&BytesAvailable::new("bcdefghi".to_string()));
        test.execute(&AtEof {});
    }

    {
        let mut test = ReassemblerTestHarness::new(3);
        for _i in (0..99997).step_by(3) {
            println!("loop {}", _i);
            let segment = format!(
                "{}{}{}{}{}{}",
                std::char::from_u32(_i % 128).unwrap(),
                std::char::from_u32((_i + 1) % 128).unwrap(),
                std::char::from_u32((_i + 2) % 128).unwrap(),
                std::char::from_u32((_i + 13) % 128).unwrap(),
                std::char::from_u32((_i + 47) % 128).unwrap(),
                std::char::from_u32((_i + 9) % 128).unwrap()
            );
            let sub = segment[0..3].to_string();
            test.execute(&SubmitSegment::new(segment, _i as SizeT, false));
            test.execute(&BytesAssembled::new((_i + 3) as SizeT));
            test.execute(&BytesAvailable::new(sub));
        }
    }
}
