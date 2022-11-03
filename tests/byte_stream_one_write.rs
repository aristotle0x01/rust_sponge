use crate::util::*;
use rust_sponge::byte_stream::SizeT;

mod util;

#[test]
fn byte_stream_one_write() {
    {
        let mut test = ByteStreamTestHarness::new(String::from("write-end-pop"), 15);

        test.execute(&Write::new(String::from("cat"), None::<SizeT>));

        test.execute(&InputEnded::new(false));
        test.execute(&BufferEmpty::new(false));
        test.execute(&Eof::new(false));
        test.execute(&BytesRead::new(0));
        test.execute(&BytesWritten::new(3));
        test.execute(&RemainingCapacity::new(12));
        test.execute(&BufferSize::new(3));
        test.execute(&Peek::new(String::from("cat")));

        test.execute(&EndInput {});

        test.execute(&InputEnded::new(true));
        test.execute(&BufferEmpty::new(false));
        test.execute(&Eof::new(false));
        test.execute(&BytesRead::new(0));
        test.execute(&BytesWritten::new(3));
        test.execute(&RemainingCapacity::new(12));
        test.execute(&BufferSize::new(3));
        test.execute(&Peek::new(String::from("cat")));

        test.execute(&Pop::new(3));

        test.execute(&InputEnded::new(true));
        test.execute(&BufferEmpty::new(true));
        test.execute(&Eof::new(true));
        test.execute(&BytesRead::new(3));
        test.execute(&BytesWritten::new(3));
        test.execute(&RemainingCapacity::new(15));
        test.execute(&BufferSize::new(0));
    }

    {
        let mut test = ByteStreamTestHarness::new(String::from("write-pop-end"), 15);

        test.execute(&Write::new(String::from("cat"), None::<SizeT>));

        test.execute(&InputEnded::new(false));
        test.execute(&BufferEmpty::new(false));
        test.execute(&Eof::new(false));
        test.execute(&BytesRead::new(0));
        test.execute(&BytesWritten::new(3));
        test.execute(&RemainingCapacity::new(12));
        test.execute(&BufferSize::new(3));
        test.execute(&Peek::new(String::from("cat")));

        test.execute(&Pop::new(3));

        test.execute(&InputEnded::new(false));
        test.execute(&BufferEmpty::new(true));
        test.execute(&Eof::new(false));
        test.execute(&BytesRead::new(3));
        test.execute(&BytesWritten::new(3));
        test.execute(&RemainingCapacity::new(15));
        test.execute(&BufferSize::new(0));

        test.execute(&EndInput {});

        test.execute(&InputEnded::new(true));
        test.execute(&BufferEmpty::new(true));
        test.execute(&Eof::new(true));
        test.execute(&BytesRead::new(3));
        test.execute(&BytesWritten::new(3));
        test.execute(&RemainingCapacity::new(15));
        test.execute(&BufferSize::new(0));
    }

    {
        let mut test = ByteStreamTestHarness::new(String::from("write-pop2-end"), 15);

        test.execute(&Write::new(String::from("cat"), None::<SizeT>));

        test.execute(&InputEnded::new(false));
        test.execute(&BufferEmpty::new(false));
        test.execute(&Eof::new(false));
        test.execute(&BytesRead::new(0));
        test.execute(&BytesWritten::new(3));
        test.execute(&RemainingCapacity::new(12));
        test.execute(&BufferSize::new(3));
        test.execute(&Peek::new(String::from("cat")));

        test.execute(&Pop::new(1));

        test.execute(&InputEnded::new(false));
        test.execute(&BufferEmpty::new(false));
        test.execute(&Eof::new(false));
        test.execute(&BytesRead::new(1));
        test.execute(&BytesWritten::new(3));
        test.execute(&RemainingCapacity::new(13));
        test.execute(&BufferSize::new(2));
        test.execute(&Peek::new(String::from("at")));

        test.execute(&Pop::new(2));

        test.execute(&InputEnded::new(false));
        test.execute(&BufferEmpty::new(true));
        test.execute(&Eof::new(false));
        test.execute(&BytesRead::new(3));
        test.execute(&BytesWritten::new(3));
        test.execute(&RemainingCapacity::new(15));
        test.execute(&BufferSize::new(0));

        test.execute(&EndInput {});

        test.execute(&InputEnded::new(true));
        test.execute(&BufferEmpty::new(true));
        test.execute(&Eof::new(true));
        test.execute(&BytesRead::new(3));
        test.execute(&BytesWritten::new(3));
        test.execute(&RemainingCapacity::new(15));
        test.execute(&BufferSize::new(0));
    }
}
