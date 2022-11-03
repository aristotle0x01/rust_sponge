use std::cmp;

pub type SizeT = usize;

// ref: https://dean.serenevy.net/blog/2021/Feb/c-string-buffers/
#[derive(Debug)]
pub struct ByteStream {
    capacity: SizeT,
    read_pos: SizeT,
    write_pos: SizeT,
    total_read_count: SizeT,
    total_write_count: SizeT,
    avail: SizeT,
    input_ended: bool,
    error: bool,
    buffer: Vec<u8>,
}
impl ByteStream {
    #[allow(dead_code)]
    pub fn new(capacity: SizeT) -> ByteStream {
        ByteStream {
            capacity,
            read_pos: 0,
            write_pos: 0,
            total_read_count: 0,
            total_write_count: 0,
            avail: capacity,
            input_ended: false,
            error: false,
            buffer: vec![0; capacity],
        }
    }

    #[allow(dead_code)]
    pub fn write(&mut self, data: &String) -> SizeT {
        let capacity = self.capacity;
        let bytes_to_write = cmp::min(self.remaining_capacity(), data.as_bytes().len());
        if bytes_to_write == 0 {
            let w: SizeT = 0;
            return w;
        }

        if bytes_to_write <= (capacity - self.write_pos) {
            let writable = &data.as_bytes()[..bytes_to_write];
            // ref: https://stackoverflow.com/questions/66609964/rust-looking-for-a-c-memcpy-equivalent
            self.buffer[self.write_pos..(self.write_pos + bytes_to_write)]
                .copy_from_slice(writable);
            self.write_pos = (self.write_pos + bytes_to_write) % capacity;
            self.total_write_count = self.total_write_count + bytes_to_write;
        } else {
            let size_1 = capacity - self.write_pos;
            let writable1 = &data.as_bytes()[0..size_1];
            self.buffer[self.write_pos..(self.write_pos + size_1)].copy_from_slice(writable1);
            self.write_pos = (self.write_pos + size_1) % capacity;

            let size_2 = bytes_to_write - size_1;
            let writable2 = &data.as_bytes()[size_1..bytes_to_write];
            self.buffer[self.write_pos..(self.write_pos + size_2)].copy_from_slice(writable2);
            self.write_pos = (self.write_pos + size_2) % capacity;

            self.total_write_count = self.total_write_count + bytes_to_write;
        }
        self.avail = self.avail - bytes_to_write;

        bytes_to_write
    }

    #[allow(dead_code)]
    pub fn read(&mut self, len: SizeT) -> String {
        let capacity = self.capacity;
        let bytes_to_read = cmp::min(self.buffer_size(), len);
        if bytes_to_read == 0 {
            return String::from("");
        }

        let mut r = String::with_capacity(bytes_to_read);

        if bytes_to_read <= (capacity - self.read_pos) {
            // todo: to_vec() by clone may hereby suffer a perf penalty
            let readable = self.buffer[self.read_pos..(self.read_pos + bytes_to_read)].to_vec();
            r.push_str(&(String::from_utf8(readable).unwrap()));
            self.read_pos = (self.read_pos + bytes_to_read) % capacity;
            self.total_read_count = self.total_read_count + bytes_to_read;
        } else {
            let size_1 = capacity - self.read_pos;
            let readable1 = self.buffer[self.read_pos..(self.read_pos + size_1)].to_vec();
            r.push_str(&(String::from_utf8(readable1).unwrap()));
            self.read_pos = (self.read_pos + size_1) % capacity;

            let size_2 = bytes_to_read - size_1;
            let readable2 = self.buffer[self.read_pos..(self.read_pos + size_2)].to_vec();
            r.push_str(&(String::from_utf8(readable2).unwrap()));
            self.read_pos = (self.read_pos + size_2) % capacity;

            self.total_read_count = self.total_read_count + bytes_to_read;
        }
        self.avail = self.avail + bytes_to_read;

        r
    }

    #[allow(dead_code)]
    pub fn peek_output(&self, len: SizeT) -> String {
        let capacity = self.capacity;
        let bytes_to_read = cmp::min(self.buffer_size(), len);
        if bytes_to_read == 0 {
            return String::from("");
        }

        let mut r = String::with_capacity(bytes_to_read);

        if bytes_to_read <= (capacity - self.read_pos) {
            let readable = &self.buffer[self.read_pos..(self.read_pos + bytes_to_read)];
            r.push_str(&(String::from_utf8(Vec::from(readable)).unwrap()));
        } else {
            let mut read_pos = self.read_pos;

            let size_1 = capacity - read_pos;
            let readable1 = &self.buffer[read_pos..(read_pos + size_1)];
            r.push_str(&(String::from_utf8(Vec::from(readable1)).unwrap()));
            read_pos = (read_pos + size_1) % capacity;

            let size_2 = bytes_to_read - size_1;
            let readable2 = &self.buffer[read_pos..(read_pos + size_2)];
            r.push_str(&(String::from_utf8(Vec::from(readable2)).unwrap()));
        }

        r
    }

    #[allow(dead_code)]
    pub fn pop_output(&mut self, len: SizeT) {
        self.read(len);
    }

    #[allow(dead_code)]
    pub fn end_input(&mut self) {
        self.input_ended = true;
    }

    #[allow(dead_code)]
    pub fn input_ended(&self) -> bool {
        self.input_ended
    }

    #[allow(dead_code)]
    pub fn buffer_size(&self) -> SizeT {
        self.total_write_count - self.total_read_count
    }

    #[allow(dead_code)]
    pub fn buffer_empty(&self) -> bool {
        self.total_write_count == self.total_read_count
    }

    #[allow(dead_code)]
    pub fn eof(&self) -> bool {
        self.input_ended && (self.total_read_count == self.total_write_count)
    }

    #[allow(dead_code)]
    pub fn bytes_written(&self) -> SizeT {
        self.total_write_count
    }

    #[allow(dead_code)]
    pub fn bytes_read(&self) -> SizeT {
        self.total_read_count
    }

    #[allow(dead_code)]
    pub fn remaining_capacity(&self) -> SizeT {
        self.avail
    }

    #[allow(dead_code)]
    pub fn set_error(&mut self) {
        self.error = true;
    }

    #[allow(dead_code)]
    pub fn error(&self) -> bool {
        self.error
    }
}
