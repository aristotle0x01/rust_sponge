use crate::util::*;

mod util;

#[test]
fn byte_stream_capacity() {
    {
        let mut test = ByteStreamTestHarness::new(String::from("overwrite"), 2);

        test.execute(Write::new(String::from("cat"), None).with_bytes_written(2));

        test.execute(&InputEnded { input_ended: false });
        test.execute(&BufferEmpty {
            buffer_empty: false,
        });
        test.execute(&Eof { eof: false });
        test.execute(&BytesRead { bytes_read: 0 });
        test.execute(&BytesWritten { bytes_written: 2 });
        test.execute(&RemainingCapacity {
            remaining_capacity: 0,
        });
        test.execute(&BufferSize { buffer_size: 2 });
        test.execute(&Peek::new(String::from("ca")));

        test.execute(Write::new(String::from("t"), None).with_bytes_written(0));

        test.execute(&InputEnded { input_ended: false });
        test.execute(&BufferEmpty {
            buffer_empty: false,
        });
        test.execute(&Eof { eof: false });
        test.execute(&BytesRead { bytes_read: 0 });
        test.execute(&BytesWritten { bytes_written: 2 });
        test.execute(&RemainingCapacity {
            remaining_capacity: 0,
        });
        test.execute(&BufferSize { buffer_size: 2 });
        test.execute(&Peek::new(String::from("ca")));
    }

    {
        let mut test = ByteStreamTestHarness::new(String::from("overwrite-clear-overwrite"), 2);

        test.execute(Write::new(String::from("cat"), None).with_bytes_written(2));
        test.execute(&Pop { len: 2 });
        test.execute(
            Write {
                data: "tac".to_string(),
                bytes_written: None,
            }
            .with_bytes_written(2),
        );

        test.execute(&InputEnded { input_ended: false });
        test.execute(&BufferEmpty {
            buffer_empty: false,
        });
        test.execute(&Eof { eof: false });
        test.execute(&BytesRead { bytes_read: 2 });
        test.execute(&BytesWritten { bytes_written: 4 });
        test.execute(&RemainingCapacity {
            remaining_capacity: 0,
        });
        test.execute(&BufferSize { buffer_size: 2 });
        test.execute(&Peek {
            output: "ta".to_string(),
        });
    }

    {
        let mut test = ByteStreamTestHarness::new(String::from("overwrite-pop-overwrite"), 2);

        test.execute(
            Write {
                data: "cat".to_string(),
                bytes_written: None,
            }
            .with_bytes_written(2),
        );
        test.execute(&Pop { len: 1 });
        test.execute(
            Write {
                data: "tac".to_string(),
                bytes_written: None,
            }
            .with_bytes_written(1),
        );

        test.execute(&InputEnded { input_ended: false });
        test.execute(&BufferEmpty {
            buffer_empty: false,
        });
        test.execute(&Eof { eof: false });
        test.execute(&BytesRead { bytes_read: 1 });
        test.execute(&BytesWritten { bytes_written: 3 });
        test.execute(&RemainingCapacity {
            remaining_capacity: 0,
        });
        test.execute(&BufferSize { buffer_size: 2 });
        test.execute(&Peek {
            output: "at".to_string(),
        });
    }

    {
        let mut test = ByteStreamTestHarness::new(String::from("long-stream"), 3);

        test.execute(
            Write {
                data: "abcdef".to_string(),
                bytes_written: None,
            }
            .with_bytes_written(3),
        );
        test.execute(&Peek {
            output: "abc".to_string(),
        });
        test.execute(&Pop { len: 1 });

        for _i in 0..99997 {
            test.execute(&RemainingCapacity {
                remaining_capacity: 1,
            });
            test.execute(&BufferSize { buffer_size: 2 });
            test.execute(
                Write {
                    data: "abc".to_string(),
                    bytes_written: None,
                }
                .with_bytes_written(1),
            );
            test.execute(&RemainingCapacity {
                remaining_capacity: 0,
            });
            test.execute(&Peek {
                output: "bca".to_string(),
            });
            test.execute(&Pop { len: 1 });

            test.execute(&RemainingCapacity {
                remaining_capacity: 1,
            });
            test.execute(&BufferSize { buffer_size: 2 });
            test.execute(
                Write {
                    data: "bca".to_string(),
                    bytes_written: None,
                }
                .with_bytes_written(1),
            );
            test.execute(&RemainingCapacity {
                remaining_capacity: 0,
            });
            test.execute(&Peek {
                output: "cab".to_string(),
            });
            test.execute(&Pop { len: 1 });

            test.execute(&RemainingCapacity {
                remaining_capacity: 1,
            });
            test.execute(&BufferSize { buffer_size: 2 });
            test.execute(
                Write {
                    data: "cab".to_string(),
                    bytes_written: None,
                }
                .with_bytes_written(1),
            );
            test.execute(&RemainingCapacity {
                remaining_capacity: 0,
            });
            test.execute(&Peek {
                output: "abc".to_string(),
            });
            test.execute(&Pop { len: 1 });
        }

        test.execute(&EndInput {});
        test.execute(&Peek {
            output: "bc".to_string(),
        });
        test.execute(&Pop { len: 2 });
        test.execute(&Eof { eof: true });
    }
}
