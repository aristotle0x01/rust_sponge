use crate::byte_stream_harness::*;
use rust_sponge::SizeT;

mod byte_stream_harness;

#[test]
fn byte_stream_two_writes() {
    {
        let mut test = ByteStreamTestHarness::new(String::from("write-write-end-pop-pop"), 15);

        test.execute(&Write {
            data: "cat".to_string(),
            bytes_written: None::<SizeT>,
        });

        test.execute(&InputEnded { input_ended: false });
        test.execute(&BufferEmpty {
            buffer_empty: false,
        });
        test.execute(&Eof { eof: false });
        test.execute(&BytesRead { bytes_read: 0 });
        test.execute(&BytesWritten { bytes_written: 3 });
        test.execute(&RemainingCapacity {
            remaining_capacity: 12,
        });
        test.execute(&BufferSize { buffer_size: 3 });
        test.execute(&Peek {
            output: "cat".to_string(),
        });

        test.execute(&Write {
            data: "tac".to_string(),
            bytes_written: None::<SizeT>,
        });

        test.execute(&InputEnded { input_ended: false });
        test.execute(&BufferEmpty {
            buffer_empty: false,
        });
        test.execute(&Eof { eof: false });
        test.execute(&BytesRead { bytes_read: 0 });
        test.execute(&BytesWritten { bytes_written: 6 });
        test.execute(&RemainingCapacity {
            remaining_capacity: 9,
        });
        test.execute(&BufferSize { buffer_size: 6 });
        test.execute(&Peek {
            output: "cattac".to_string(),
        });

        test.execute(&EndInput {});

        test.execute(&InputEnded { input_ended: true });
        test.execute(&BufferEmpty {
            buffer_empty: false,
        });
        test.execute(&Eof { eof: false });
        test.execute(&BytesRead { bytes_read: 0 });
        test.execute(&BytesWritten { bytes_written: 6 });
        test.execute(&RemainingCapacity {
            remaining_capacity: 9,
        });
        test.execute(&BufferSize { buffer_size: 6 });
        test.execute(&Peek {
            output: "cattac".to_string(),
        });

        test.execute(&Pop { len: 2 });

        test.execute(&InputEnded { input_ended: true });
        test.execute(&BufferEmpty {
            buffer_empty: false,
        });
        test.execute(&Eof { eof: false });
        test.execute(&BytesRead { bytes_read: 2 });
        test.execute(&BytesWritten { bytes_written: 6 });
        test.execute(&RemainingCapacity {
            remaining_capacity: 11,
        });
        test.execute(&BufferSize { buffer_size: 4 });
        test.execute(&Peek {
            output: "ttac".to_string(),
        });

        test.execute(&Pop { len: 4 });

        test.execute(&InputEnded { input_ended: true });
        test.execute(&BufferEmpty { buffer_empty: true });
        test.execute(&Eof { eof: true });
        test.execute(&BytesRead { bytes_read: 6 });
        test.execute(&BytesWritten { bytes_written: 6 });
        test.execute(&RemainingCapacity {
            remaining_capacity: 15,
        });
        test.execute(&BufferSize { buffer_size: 0 });
    }

    {
        let mut test = ByteStreamTestHarness::new(String::from("write-pop-write-end-pop"), 15);

        test.execute(&Write {
            data: "cat".to_string(),
            bytes_written: None::<SizeT>,
        });

        test.execute(&InputEnded { input_ended: false });
        test.execute(&BufferEmpty {
            buffer_empty: false,
        });
        test.execute(&Eof { eof: false });
        test.execute(&BytesRead { bytes_read: 0 });
        test.execute(&BytesWritten { bytes_written: 3 });
        test.execute(&RemainingCapacity {
            remaining_capacity: 12,
        });
        test.execute(&BufferSize { buffer_size: 3 });
        test.execute(&Peek {
            output: "cat".to_string(),
        });

        test.execute(&Pop { len: 2 });

        test.execute(&InputEnded { input_ended: false });
        test.execute(&BufferEmpty {
            buffer_empty: false,
        });
        test.execute(&Eof { eof: false });
        test.execute(&BytesRead { bytes_read: 2 });
        test.execute(&BytesWritten { bytes_written: 3 });
        test.execute(&RemainingCapacity {
            remaining_capacity: 14,
        });
        test.execute(&BufferSize { buffer_size: 1 });
        test.execute(&Peek {
            output: "t".to_string(),
        });

        test.execute(&Write {
            data: "tac".to_string(),
            bytes_written: None::<SizeT>,
        });

        test.execute(&InputEnded { input_ended: false });
        test.execute(&BufferEmpty {
            buffer_empty: false,
        });
        test.execute(&Eof { eof: false });
        test.execute(&BytesRead { bytes_read: 2 });
        test.execute(&BytesWritten { bytes_written: 6 });
        test.execute(&RemainingCapacity {
            remaining_capacity: 11,
        });
        test.execute(&BufferSize { buffer_size: 4 });
        test.execute(&Peek {
            output: "ttac".to_string(),
        });

        test.execute(&EndInput {});

        test.execute(&InputEnded { input_ended: true });
        test.execute(&BufferEmpty {
            buffer_empty: false,
        });
        test.execute(&Eof { eof: false });
        test.execute(&BytesRead { bytes_read: 2 });
        test.execute(&BytesWritten { bytes_written: 6 });
        test.execute(&RemainingCapacity {
            remaining_capacity: 11,
        });
        test.execute(&BufferSize { buffer_size: 4 });
        test.execute(&Peek {
            output: "ttac".to_string(),
        });

        test.execute(&Pop { len: 4 });

        test.execute(&InputEnded { input_ended: true });
        test.execute(&BufferEmpty { buffer_empty: true });
        test.execute(&Eof { eof: true });
        test.execute(&BytesRead { bytes_read: 6 });
        test.execute(&BytesWritten { bytes_written: 6 });
        test.execute(&RemainingCapacity {
            remaining_capacity: 15,
        });
        test.execute(&BufferSize { buffer_size: 0 });
    }
}
