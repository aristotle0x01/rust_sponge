use std::cmp;

type SizeT = usize;

pub struct ByteStream {
    capacity: SizeT,
    read_pos: SizeT,
    write_pos: SizeT,
    total_read_count: SizeT,
    total_write_count: SizeT,
    avail: SizeT,
    input_ended: bool,
    error: bool,
    buffer: Vec<u8>
}
impl ByteStream {
  #[allow(dead_code)]
  fn new(capacity: SizeT) -> ByteStream {
    ByteStream { capacity, read_pos: 0, write_pos: 0,
      total_read_count: 0, total_write_count: 0, avail: capacity,
      input_ended: false, error: false, buffer: vec![0; capacity]
    }
  }

  #[allow(dead_code)]
  fn write(&mut self, data: &String) -> SizeT {
    let capacity = self.capacity;
    let bytes_to_write = cmp::min(self.remaining_capacity(), data.as_bytes().len());
    if bytes_to_write == 0 {
      let w: SizeT = 0;
      return w;
    }

    if bytes_to_write <= (capacity - self.write_pos){
      let writable = &data.as_bytes()[..bytes_to_write];
      self.buffer[self.write_pos..(self.write_pos+bytes_to_write)].copy_from_slice(writable);
      self.write_pos = (self.write_pos + bytes_to_write) % capacity;
      self.total_write_count = self.total_write_count + bytes_to_write;
    } else {
      let size_1 = capacity - self.write_pos;
      let writable1 = &data.as_bytes()[0..size_1];
      self.buffer[self.write_pos..(self.write_pos+size_1)].copy_from_slice(writable1);
      self.write_pos = (self.write_pos + size_1) % capacity;

      let size_2 = bytes_to_write - size_1;
      let writable2 = &data.as_bytes()[size_1..bytes_to_write];
      self.buffer[self.write_pos..(self.write_pos+size_2)].copy_from_slice(writable2);
      self.write_pos = (self.write_pos + size_2) % capacity;

      self.total_write_count = self.total_write_count + bytes_to_write;
    }
    self.avail = self.avail - bytes_to_write;

    bytes_to_write
  }

  #[allow(dead_code)]
  fn read(&mut self, len: SizeT) -> String {
    let capacity = self.capacity;
    let bytes_to_read = cmp::min(self.buffer_size(), len);
    if bytes_to_read == 0 {
      return String::from("");
    }

    let mut r = String::with_capacity(bytes_to_read);

    if bytes_to_read <= (capacity - self.read_pos){
      // todo: to_vec() by clone may hereby suffer a perf penalty
      let readable = self.buffer[self.read_pos..(self.read_pos+bytes_to_read)].to_vec();
      r.push_str(&(String::from_utf8(readable).unwrap()));
      self.read_pos = (self.read_pos + bytes_to_read) % capacity;
      self.total_read_count = self.total_read_count + bytes_to_read;
    } else {
      let size_1 = capacity - self.read_pos;
      let readable1 = self.buffer[self.read_pos..(self.read_pos+size_1)].to_vec();
      r.push_str(&(String::from_utf8(readable1).unwrap()));
      self.read_pos = (self.read_pos + size_1) % capacity;

      let size_2 = bytes_to_read - size_1;
      let readable2 = self.buffer[self.read_pos..(self.read_pos+size_2)].to_vec();
      r.push_str(&(String::from_utf8(readable2).unwrap()));
      self.read_pos = (self.read_pos + size_2) % capacity;

      self.total_read_count = self.total_read_count + bytes_to_read;
    }
    self.avail = self.avail + bytes_to_read;

    r
  }

  #[allow(dead_code)]
  fn peek_output(&self, len: SizeT) -> String {
    let capacity = self.capacity;
    let bytes_to_read = cmp::min(self.buffer_size(), len);
    if bytes_to_read == 0 {
      return String::from("");
    }

    let mut r = String::with_capacity(bytes_to_read);

    if bytes_to_read <= (capacity - self.read_pos){
      let readable = &self.buffer[self.read_pos..(self.read_pos+bytes_to_read)];
      r.push_str(&(String::from_utf8(Vec::from(readable)).unwrap()));
    } else {
      let mut read_pos = self.read_pos;

      let size_1 = capacity - read_pos;
      let readable1 = &self.buffer[read_pos..(read_pos+size_1)];
      r.push_str(&(String::from_utf8(Vec::from(readable1)).unwrap()));
      read_pos = (read_pos + size_1) % capacity;

      let size_2 = bytes_to_read - size_1;
      let readable2 = &self.buffer[read_pos..(read_pos+size_2)];
      r.push_str(&(String::from_utf8(Vec::from(readable2)).unwrap()));
    }

    r
  }

  #[allow(dead_code)]
  fn pop_output(&mut self, len: SizeT) {
    self.read(len);
  }

  #[allow(dead_code)]
  fn end_input(&mut self) {
    self.input_ended = true;
  }

  #[allow(dead_code)]
  fn input_ended(&self) -> bool {
    self.input_ended
  }

  #[allow(dead_code)]
  fn buffer_size(&self) -> SizeT {
    self.total_write_count - self.total_read_count
  }

  #[allow(dead_code)]
  fn buffer_empty(&self) -> bool {
    self.total_write_count == self.total_read_count
  }

  #[allow(dead_code)]
  fn eof(&self) -> bool {
    self.input_ended && (self.total_read_count == self.total_write_count)
  }

  #[allow(dead_code)]
  fn bytes_written(&self) -> SizeT {
    self.total_write_count
  }

  #[allow(dead_code)]
  fn bytes_read(&self) -> SizeT {
    self.total_read_count
  }

  #[allow(dead_code)]
  fn remaining_capacity(&self) -> SizeT {
    self.avail
  }

  #[allow(dead_code)]
  fn set_error(&mut self) {
    self.error = true;
  }

  #[allow(dead_code)]
  fn error(&self) -> bool {
    self.error
  }
}

#[cfg(test)]
mod tests {
  use super::*;

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
      println!("  step: {}",  ByteStreamAction::into(self));
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
    bytes_written: Option<SizeT>
  }
  impl ByteStreamTestStep for Write {
    fn execute(&self, bs: &mut ByteStream) {
      println!("  step: {}",  ByteStreamAction::into(self));

      let written = bs.write(&self.data);
      match self.bytes_written {
        Some(v) => {assert_eq!(v, written)},
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
      Write{data, bytes_written}
    }

    #[allow(dead_code)]
    fn with_bytes_written(&mut self, bytes_written: SizeT) -> &Write {
      let _a = self.bytes_written.insert(bytes_written);
      self
    }
  }

  pub struct Pop {
    len: SizeT
  }
  impl ByteStreamTestStep for Pop {
    fn execute(&self, bs: &mut ByteStream) {
      println!("  step: {}",  ByteStreamAction::into(self));
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
      Pop{len: _len}
    }
  }

  pub struct InputEnded {
    input_ended: bool
  }
  impl ByteStreamTestStep for InputEnded {
    fn execute(&self, bs: &mut ByteStream) {
      println!("  step: {}",  ByteStreamExpectation::into(self));
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
      InputEnded{input_ended: _input_ended}
    }
  }

  pub struct BufferEmpty {
    buffer_empty: bool
  }
  impl ByteStreamTestStep for BufferEmpty {
    fn execute(&self, bs: &mut ByteStream) {
      println!("  step: {}",  ByteStreamExpectation::into(self));
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
      BufferEmpty{buffer_empty: _buffer_empty}
    }
  }

  pub struct Eof {
    eof: bool
  }
  impl ByteStreamTestStep for Eof {
    fn execute(&self, bs: &mut ByteStream) {
      println!("  step: {}",  ByteStreamExpectation::into(self));
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
      Eof{eof: _eof}
    }
  }

  pub struct BufferSize {
    buffer_size: SizeT
  }
  impl ByteStreamTestStep for BufferSize {
    fn execute(&self, bs: &mut ByteStream) {
      println!("  step: {}",  ByteStreamExpectation::into(self));
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
      BufferSize{buffer_size: _buffer_size}
    }
  }

  pub struct BytesWritten {
    bytes_written: SizeT
  }
  impl ByteStreamTestStep for BytesWritten {
    fn execute(&self, bs: &mut ByteStream) {
      println!("  step: {}",  ByteStreamExpectation::into(self));
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
      BytesWritten{bytes_written: _bytes_written}
    }
  }

  pub struct BytesRead {
    bytes_read: SizeT
  }
  impl ByteStreamTestStep for BytesRead {
    fn execute(&self, bs: &mut ByteStream) {
      println!("  step: {}",  ByteStreamExpectation::into(self));
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
      BytesRead{bytes_read: _bytes_read}
    }
  }

  pub struct RemainingCapacity {
    remaining_capacity: SizeT
  }
  impl ByteStreamTestStep for RemainingCapacity {
    fn execute(&self, bs: &mut ByteStream) {
      println!("  step: {}",  ByteStreamExpectation::into(self));
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
      RemainingCapacity{remaining_capacity: _remaining_capacity}
    }
  }

  pub struct Peek {
    output: String
  }
  impl ByteStreamTestStep for Peek {
    fn execute(&self, bs: &mut ByteStream) {
      println!("  step: {}",  ByteStreamExpectation::into(self));
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
      Peek{output: _output}
    }
  }

  pub struct ByteStreamTestHarness {
    test_name: String,
    byte_stream: ByteStream
  }
  impl ByteStreamTestHarness {
    #[allow(dead_code)]
    fn new(_test_name: String, capacity: SizeT) -> ByteStreamTestHarness {
      let harness = ByteStreamTestHarness{test_name: _test_name, byte_stream: ByteStream::new(capacity)};
      println!("test:=> {} start with capacity {}", harness.test_name, capacity);

      harness
    }

    #[allow(dead_code)]
    fn execute(&mut self, step: &dyn ByteStreamTestStep) {
      step.execute(&mut self.byte_stream)
    }
  }

  #[test]
  fn byte_stream_one_write() {
    {
      let mut test = ByteStreamTestHarness::new(String::from("write-end-pop"),15);

      test.execute(&Write::new(String::from("cat"), None));

      test.execute(&InputEnded::new(false));
      test.execute(&BufferEmpty::new(false));
      test.execute(&Eof::new(false));
      test.execute(&BytesRead::new(0));
      test.execute(&BytesWritten::new(3));
      test.execute(&RemainingCapacity::new(12));
      test.execute(&BufferSize::new(3));
      test.execute(&Peek::new(String::from("cat")));

      test.execute(&EndInput{});

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
      let mut test = ByteStreamTestHarness::new(String::from("write-pop-end"),15);

      test.execute(&Write::new(String::from("cat"), None));

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

      test.execute(&EndInput{});

      test.execute(&InputEnded::new(true));
      test.execute(&BufferEmpty::new(true));
      test.execute(&Eof::new(true));
      test.execute(&BytesRead::new(3));
      test.execute(&BytesWritten::new(3));
      test.execute(&RemainingCapacity::new(15));
      test.execute(&BufferSize::new(0));
    }

    {
      let mut test = ByteStreamTestHarness::new(String::from("write-pop2-end"),15);

      test.execute(&Write::new(String::from("cat"), None));

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

      test.execute(&EndInput{});

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
  fn byte_stream_capacity() {
    {
      let mut test = ByteStreamTestHarness::new(String::from("overwrite"),2);

      test.execute(Write::new(String::from("cat"), None).with_bytes_written(2));

      test.execute(&InputEnded{input_ended: false });
      test.execute(&BufferEmpty{buffer_empty: false });
      test.execute(&Eof{eof: false });
      test.execute(&BytesRead{ bytes_read: 0});
      test.execute(&BytesWritten{ bytes_written: 2 });
      test.execute(&RemainingCapacity{ remaining_capacity: 0 });
      test.execute(&BufferSize{ buffer_size: 2 });
      test.execute(&Peek::new(String::from("ca")));

      test.execute(Write::new(String::from("t"), None).with_bytes_written(0));

      test.execute(&InputEnded{input_ended: false });
      test.execute(&BufferEmpty{buffer_empty: false });
      test.execute(&Eof{eof: false });
      test.execute(&BytesRead{bytes_read: 0 });
      test.execute(&BytesWritten{bytes_written: 2});
      test.execute(&RemainingCapacity{remaining_capacity: 0});
      test.execute(&BufferSize{ buffer_size: 2});
      test.execute(&Peek::new(String::from("ca")));
    }

    {
      let mut test = ByteStreamTestHarness::new(String::from("overwrite-clear-overwrite"),2);

      test.execute(Write::new(String::from("cat"), None).with_bytes_written(2));
      test.execute(&Pop{len: 2});
      test.execute(Write{data: "tac".to_string(), bytes_written: None }.with_bytes_written(2));

      test.execute(&InputEnded{input_ended: false });
      test.execute(&BufferEmpty{ buffer_empty: false });
      test.execute(&Eof{ eof: false });
      test.execute(&BytesRead{ bytes_read: 2 });
      test.execute(&BytesWritten{ bytes_written: 4});
      test.execute(&RemainingCapacity{ remaining_capacity: 0});
      test.execute(&BufferSize{ buffer_size: 2});
      test.execute(&Peek{output: "ta".to_string() });
    }

    {
      let mut test = ByteStreamTestHarness::new(String::from("overwrite-pop-overwrite"),2);

      test.execute(Write{data: "cat".to_string(), bytes_written: None }.with_bytes_written(2));
      test.execute(&Pop{len: 1});
      test.execute(Write{data: "tac".to_string(), bytes_written: None }.with_bytes_written(1));

      test.execute(&InputEnded{input_ended: false });
      test.execute(&BufferEmpty{buffer_empty: false });
      test.execute(&Eof{eof: false });
      test.execute(&BytesRead{ bytes_read: 1});
      test.execute(&BytesWritten{ bytes_written: 3});
      test.execute(&RemainingCapacity{ remaining_capacity: 0});
      test.execute(&BufferSize{ buffer_size: 2});
      test.execute(&Peek{output: "at".to_string() });
    }

    {
      let mut test = ByteStreamTestHarness::new(String::from("long-stream"),3);

      test.execute(Write{data: "abcdef".to_string(), bytes_written: None }.with_bytes_written(3));
      test.execute(&Peek{output: "abc".to_string() });
      test.execute(&Pop{ len: 1});

      for _i in 0..99997 {
        test.execute(&RemainingCapacity{ remaining_capacity: 1});
        test.execute(&BufferSize{ buffer_size: 2});
        test.execute(Write{data: "abc".to_string(), bytes_written: None }.with_bytes_written(1));
        test.execute(&RemainingCapacity{ remaining_capacity: 0});
        test.execute(&Peek{output: "bca".to_string() });
        test.execute(&Pop{ len: 1});

        test.execute(&RemainingCapacity{ remaining_capacity: 1});
        test.execute(&BufferSize{ buffer_size: 2});
        test.execute(Write{data: "bca".to_string(), bytes_written: None }.with_bytes_written(1));
        test.execute(&RemainingCapacity{ remaining_capacity: 0});
        test.execute(&Peek{output: "cab".to_string() });
        test.execute(&Pop{ len: 1});

        test.execute(&RemainingCapacity{ remaining_capacity: 1});
        test.execute(&BufferSize{ buffer_size: 2});
        test.execute(Write{data: "cab".to_string(), bytes_written: None }.with_bytes_written(1));
        test.execute(&RemainingCapacity{ remaining_capacity: 0});
        test.execute(&Peek{output: "abc".to_string() });
        test.execute(&Pop{ len: 1});
      }

      test.execute(&EndInput{});
      test.execute(&Peek{output: "bc".to_string() });
      test.execute(&Pop{ len: 2});
      test.execute(&Eof{eof: true });
    }
  }
}