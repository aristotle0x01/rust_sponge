use rust_sponge::byte_stream::ByteStream;
use rust_sponge::SizeT;

pub trait ByteStreamTestStep {
    fn execute(&self, bs: &mut ByteStream);
}

pub trait ByteStreamExpectation: ByteStreamTestStep {
    fn description(&self) -> String;
    fn into(&self) -> String {
        String::from(format!("  Expectation: {}", self.description()))
    }
}

pub trait ByteStreamAction: ByteStreamTestStep {
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
    pub data: String,
    pub bytes_written: Option<SizeT>,
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
    pub fn new(data: String, bytes_written: Option<SizeT>) -> Write {
        Write {
            data,
            bytes_written,
        }
    }

    #[allow(dead_code)]
    pub fn with_bytes_written(&mut self, bytes_written: SizeT) -> &Write {
        let _a = self.bytes_written.insert(bytes_written);
        self
    }
}

pub struct Pop {
    pub len: SizeT,
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
    pub fn new(_len: SizeT) -> Pop {
        Pop { len: _len }
    }
}

pub struct InputEnded {
    pub input_ended: bool,
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
    pub fn new(_input_ended: bool) -> InputEnded {
        InputEnded {
            input_ended: _input_ended,
        }
    }
}

pub struct BufferEmpty {
    pub buffer_empty: bool,
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
    pub fn new(_buffer_empty: bool) -> BufferEmpty {
        BufferEmpty {
            buffer_empty: _buffer_empty,
        }
    }
}

pub struct Eof {
    pub eof: bool,
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
    pub fn new(_eof: bool) -> Eof {
        Eof { eof: _eof }
    }
}

pub struct BufferSize {
    pub buffer_size: SizeT,
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
    pub fn new(_buffer_size: SizeT) -> BufferSize {
        BufferSize {
            buffer_size: _buffer_size,
        }
    }
}

pub struct BytesWritten {
    pub bytes_written: SizeT,
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
    pub fn new(_bytes_written: SizeT) -> BytesWritten {
        BytesWritten {
            bytes_written: _bytes_written,
        }
    }
}

pub struct BytesRead {
    pub bytes_read: SizeT,
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
    pub fn new(_bytes_read: SizeT) -> BytesRead {
        BytesRead {
            bytes_read: _bytes_read,
        }
    }
}

pub struct RemainingCapacity {
    pub remaining_capacity: SizeT,
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
    pub fn new(_remaining_capacity: SizeT) -> RemainingCapacity {
        RemainingCapacity {
            remaining_capacity: _remaining_capacity,
        }
    }
}

pub struct Peek {
    pub output: String,
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
    pub fn new(_output: String) -> Peek {
        Peek { output: _output }
    }
}

pub struct ByteStreamTestHarness {
    test_name: String,
    byte_stream: ByteStream,
}
impl ByteStreamTestHarness {
    #[allow(dead_code)]
    pub fn new(_test_name: String, capacity: SizeT) -> ByteStreamTestHarness {
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
    pub fn execute(&mut self, step: &dyn ByteStreamTestStep) {
        step.execute(&mut self.byte_stream)
    }
}
