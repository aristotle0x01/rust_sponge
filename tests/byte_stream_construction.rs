use crate::util::*;

mod util;

#[test]
fn byte_stream_construction() {
    {
        let mut test = ByteStreamTestHarness::new(String::from("construction"), 15);

        test.execute(&InputEnded { input_ended: false });
        test.execute(&BufferEmpty { buffer_empty: true });
        test.execute(&Eof { eof: false });
        test.execute(&BytesRead { bytes_read: 0 });
        test.execute(&BytesWritten { bytes_written: 0 });
        test.execute(&RemainingCapacity {
            remaining_capacity: 15,
        });
        test.execute(&BufferSize { buffer_size: 0 });
    }

    {
        let mut test = ByteStreamTestHarness::new(String::from("construction-end"), 15);

        test.execute(&EndInput {});
        test.execute(&InputEnded { input_ended: true });
        test.execute(&BufferEmpty { buffer_empty: true });
        test.execute(&Eof { eof: true });
        test.execute(&BytesRead { bytes_read: 0 });
        test.execute(&BytesWritten { bytes_written: 0 });
        test.execute(&RemainingCapacity {
            remaining_capacity: 15,
        });
        test.execute(&BufferSize { buffer_size: 0 });
    }
}
