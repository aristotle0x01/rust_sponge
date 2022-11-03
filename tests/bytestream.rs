#[cfg(test)]
mod tests {
    use rust_sponge::byte_stream::ByteStream;
    use rust_sponge::byte_stream::SizeT;

    trait ByteStreamTestStep {
        fn execute(&self, bs: &mut ByteStream);
    }

    trait ByteStreamExpectation: ByteStreamTestStep {
        fn description(&self) -> String;
        fn into(&self) -> String {
            String::from(format!("  Expectation: {}", self.description()))
        }
    }

    trait ByteStreamAction: ByteStreamTestStep {
        fn description(&self) -> String;
        fn into(&self) -> String {
            String::from(format!("  Action: {}", self.description()))
        }
    }

    pub struct EndInput;
    impl ByteStreamTestStep for EndInput {
        fn execute(&self, bs: &mut ByteStream) {
            println!("  step: {}", ByteStreamAction::into(self));
            bs.end_input();
        }
    }
    impl ByteStreamAction for EndInput {
        fn description(&self) -> String {
            String::from("end input")
        }
    }

    pub struct Write {
        data: String,
        bytes_written: Option<SizeT>,
    }
    impl ByteStreamTestStep for Write {
        fn execute(&self, bs: &mut ByteStream) {
            println!("  step: {}", ByteStreamAction::into(self));

            let written = bs.write(&self.data);
            match self.bytes_written {
                Some(v) => {
                    assert_eq!(v, written)
                }
                _ => {}
            }
        }
    }
    impl ByteStreamAction for Write {
        fn description(&self) -> String {
            format!("write \"{}\" to the stream", self.data)
        }
    }
    impl Write {
        #[allow(dead_code)]
        fn new(data: String, bytes_written: Option<SizeT>) -> Write {
            Write {
                data,
                bytes_written,
            }
        }

        #[allow(dead_code)]
        fn with_bytes_written(&mut self, bytes_written: SizeT) -> &Write {
            let _a = self.bytes_written.insert(bytes_written);
            self
        }
    }

    pub struct Pop {
        len: SizeT,
    }
    impl ByteStreamTestStep for Pop {
        fn execute(&self, bs: &mut ByteStream) {
            println!("  step: {}", ByteStreamAction::into(self));
            bs.pop_output(self.len);
        }
    }
    impl ByteStreamAction for Pop {
        fn description(&self) -> String {
            format!("pop {}", self.len)
        }
    }
    impl Pop {
        #[allow(dead_code)]
        fn new(_len: SizeT) -> Pop {
            Pop { len: _len }
        }
    }

    pub struct InputEnded {
        input_ended: bool,
    }
    impl ByteStreamTestStep for InputEnded {
        fn execute(&self, bs: &mut ByteStream) {
            println!("  step: {}", ByteStreamExpectation::into(self));
            let b = bs.input_ended();
            assert_eq!(b, self.input_ended);
        }
    }
    impl ByteStreamExpectation for InputEnded {
        fn description(&self) -> String {
            format!("input_ended: {}", self.input_ended)
        }
    }
    impl InputEnded {
        #[allow(dead_code)]
        fn new(_input_ended: bool) -> InputEnded {
            InputEnded {
                input_ended: _input_ended,
            }
        }
    }

    pub struct BufferEmpty {
        buffer_empty: bool,
    }
    impl ByteStreamTestStep for BufferEmpty {
        fn execute(&self, bs: &mut ByteStream) {
            println!("  step: {}", ByteStreamExpectation::into(self));
            let b = bs.buffer_empty();
            assert_eq!(b, self.buffer_empty);
        }
    }
    impl ByteStreamExpectation for BufferEmpty {
        fn description(&self) -> String {
            format!("buffer_empty: {}", self.buffer_empty)
        }
    }
    impl BufferEmpty {
        #[allow(dead_code)]
        fn new(_buffer_empty: bool) -> BufferEmpty {
            BufferEmpty {
                buffer_empty: _buffer_empty,
            }
        }
    }

    pub struct Eof {
        eof: bool,
    }
    impl ByteStreamTestStep for Eof {
        fn execute(&self, bs: &mut ByteStream) {
            println!("  step: {}", ByteStreamExpectation::into(self));
            let b = bs.eof();
            assert_eq!(b, self.eof);
        }
    }
    impl ByteStreamExpectation for Eof {
        fn description(&self) -> String {
            format!("eof: {}", self.eof)
        }
    }
    impl Eof {
        #[allow(dead_code)]
        fn new(_eof: bool) -> Eof {
            Eof { eof: _eof }
        }
    }

    pub struct BufferSize {
        buffer_size: SizeT,
    }
    impl ByteStreamTestStep for BufferSize {
        fn execute(&self, bs: &mut ByteStream) {
            println!("  step: {}", ByteStreamExpectation::into(self));
            let b = bs.buffer_size();
            assert_eq!(b, self.buffer_size);
        }
    }
    impl ByteStreamExpectation for BufferSize {
        fn description(&self) -> String {
            format!("buffer_size: {}", self.buffer_size)
        }
    }
    impl BufferSize {
        #[allow(dead_code)]
        fn new(_buffer_size: SizeT) -> BufferSize {
            BufferSize {
                buffer_size: _buffer_size,
            }
        }
    }

    pub struct BytesWritten {
        bytes_written: SizeT,
    }
    impl ByteStreamTestStep for BytesWritten {
        fn execute(&self, bs: &mut ByteStream) {
            println!("  step: {}", ByteStreamExpectation::into(self));
            let b = bs.bytes_written();
            assert_eq!(b, self.bytes_written);
        }
    }
    impl ByteStreamExpectation for BytesWritten {
        fn description(&self) -> String {
            format!("bytes_written: {}", self.bytes_written)
        }
    }
    impl BytesWritten {
        #[allow(dead_code)]
        fn new(_bytes_written: SizeT) -> BytesWritten {
            BytesWritten {
                bytes_written: _bytes_written,
            }
        }
    }

    pub struct BytesRead {
        bytes_read: SizeT,
    }
    impl ByteStreamTestStep for BytesRead {
        fn execute(&self, bs: &mut ByteStream) {
            println!("  step: {}", ByteStreamExpectation::into(self));
            let b = bs.bytes_read();
            assert_eq!(b, self.bytes_read);
        }
    }
    impl ByteStreamExpectation for BytesRead {
        fn description(&self) -> String {
            format!("bytes_read: {}", self.bytes_read)
        }
    }
    impl BytesRead {
        #[allow(dead_code)]
        fn new(_bytes_read: SizeT) -> BytesRead {
            BytesRead {
                bytes_read: _bytes_read,
            }
        }
    }

    pub struct RemainingCapacity {
        remaining_capacity: SizeT,
    }
    impl ByteStreamTestStep for RemainingCapacity {
        fn execute(&self, bs: &mut ByteStream) {
            println!("  step: {}", ByteStreamExpectation::into(self));
            let b = bs.remaining_capacity();
            assert_eq!(b, self.remaining_capacity);
        }
    }
    impl ByteStreamExpectation for RemainingCapacity {
        fn description(&self) -> String {
            format!("remaining_capacity: {}", self.remaining_capacity)
        }
    }
    impl RemainingCapacity {
        #[allow(dead_code)]
        fn new(_remaining_capacity: SizeT) -> RemainingCapacity {
            RemainingCapacity {
                remaining_capacity: _remaining_capacity,
            }
        }
    }

    pub struct Peek {
        output: String,
    }
    impl ByteStreamTestStep for Peek {
        fn execute(&self, bs: &mut ByteStream) {
            println!("  step: {}", ByteStreamExpectation::into(self));
            let b = bs.peek_output(self.output.len());
            assert_eq!(b, self.output);
        }
    }
    impl ByteStreamExpectation for Peek {
        fn description(&self) -> String {
            format!("\"{}\" at the front of the stream", self.output)
        }
    }
    impl Peek {
        #[allow(dead_code)]
        fn new(_output: String) -> Peek {
            Peek { output: _output }
        }
    }

    pub struct ByteStreamTestHarness {
        test_name: String,
        byte_stream: ByteStream,
    }
    impl ByteStreamTestHarness {
        #[allow(dead_code)]
        fn new(_test_name: String, capacity: SizeT) -> ByteStreamTestHarness {
            let harness = ByteStreamTestHarness {
                test_name: _test_name,
                byte_stream: ByteStream::new(capacity),
            };
            println!(
                "test:=> {} start with capacity {}",
                harness.test_name, capacity
            );

            harness
        }

        #[allow(dead_code)]
        fn execute(&mut self, step: &dyn ByteStreamTestStep) {
            step.execute(&mut self.byte_stream)
        }
    }

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

    #[test]
    fn byte_stream_many_writes() {
        use rand::Rng;

        const NREPS: SizeT = 1000;
        const MIN_WRITE: SizeT = 10;
        const MAX_WRITE: SizeT = 200;
        const CAPACITY: SizeT = MAX_WRITE * NREPS;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

        {
            let mut test = ByteStreamTestHarness::new(String::from("many writes"), CAPACITY);

            let mut acc: SizeT = 0;
            for _i in 0..NREPS {
                let rd1 = rand::thread_rng().gen_range(1..=1000000);
                let size = MIN_WRITE + (rd1 % (MAX_WRITE - MIN_WRITE));
                // https://github.com/HKarimiA/rust-generate-random-string
                let d: String = (0..size)
                    .map(|_| {
                        let idx = rand::thread_rng().gen_range(0..CHARSET.len());
                        CHARSET[idx] as char
                    })
                    .collect();

                test.execute(
                    Write {
                        data: d,
                        bytes_written: None,
                    }
                        .with_bytes_written(size),
                );
                acc += size;

                test.execute(&InputEnded { input_ended: false });
                test.execute(&BufferEmpty {
                    buffer_empty: false,
                });
                test.execute(&Eof { eof: false });
                test.execute(&BytesRead { bytes_read: 0 });
                test.execute(&BytesWritten { bytes_written: acc });
                test.execute(&RemainingCapacity {
                    remaining_capacity: CAPACITY - acc,
                });
                test.execute(&BufferSize { buffer_size: acc });
            }
        }
    }

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
                bytes_written: None,
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
                bytes_written: None,
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
                bytes_written: None,
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
}
