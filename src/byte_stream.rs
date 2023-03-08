use crate::SizeT;
use std::cmp;

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
    pub fn write(&mut self, data: &[u8]) -> SizeT {
        if data.is_empty() {
            return 0;
        }

        let capacity = self.capacity;
        let bytes_to_write = cmp::min(self.remaining_capacity(), data.len());

        if bytes_to_write <= (capacity - self.write_pos) {
            let writable = &data[..bytes_to_write];
            // ref: https://stackoverflow.com/questions/66609964/rust-looking-for-a-c-memcpy-equivalent
            self.buffer[self.write_pos..(self.write_pos + bytes_to_write)]
                .copy_from_slice(writable);
            self.write_pos = (self.write_pos + bytes_to_write) % capacity;
            self.total_write_count = self.total_write_count + bytes_to_write;
        } else {
            let size_1 = capacity - self.write_pos;
            let writable1 = &data[0..size_1];
            self.buffer[self.write_pos..(self.write_pos + size_1)].copy_from_slice(writable1);
            self.write_pos = (self.write_pos + size_1) % capacity;

            let size_2 = bytes_to_write - size_1;
            let writable2 = &data[size_1..bytes_to_write];
            self.buffer[self.write_pos..(self.write_pos + size_2)].copy_from_slice(writable2);
            self.write_pos = (self.write_pos + size_2) % capacity;

            self.total_write_count = self.total_write_count + bytes_to_write;
        }
        self.avail = self.avail - bytes_to_write;

        bytes_to_write
    }

    #[allow(dead_code)]
    pub fn read(&mut self, len: SizeT) -> Vec<u8> {
        if len == 0 {
            return vec![];
        }

        let capacity = self.capacity;
        let bytes_to_read = cmp::min(self.buffer_size(), len);

        let mut r = vec![0u8; bytes_to_read];
        if bytes_to_read <= (capacity - self.read_pos) {
            r.copy_from_slice(&self.buffer[self.read_pos..(self.read_pos + bytes_to_read)]);
            self.read_pos = (self.read_pos + bytes_to_read) % capacity;
            self.total_read_count = self.total_read_count + bytes_to_read;
        } else {
            let size_1 = capacity - self.read_pos;
            r[0..size_1].copy_from_slice(&self.buffer[self.read_pos..(self.read_pos + size_1)]);
            self.read_pos = (self.read_pos + size_1) % capacity;

            let size_2 = bytes_to_read - size_1;
            r[size_1..bytes_to_read]
                .copy_from_slice(&self.buffer[self.read_pos..(self.read_pos + size_2)]);
            self.read_pos = (self.read_pos + size_2) % capacity;

            self.total_read_count = self.total_read_count + bytes_to_read;
        }
        self.avail = self.avail + bytes_to_read;

        r
    }

    #[allow(dead_code)]
    pub fn peek_output(&self, len: SizeT) -> Vec<u8> {
        if len == 0 {
            return vec![];
        }

        let capacity = self.capacity;
        let bytes_to_read = cmp::min(self.buffer_size(), len);

        let mut r = vec![0u8; bytes_to_read];
        if bytes_to_read <= (capacity - self.read_pos) {
            let readable = &self.buffer[self.read_pos..(self.read_pos + bytes_to_read)];
            r.copy_from_slice(readable);
        } else {
            let mut read_pos = self.read_pos;

            let size_1 = capacity - read_pos;
            r[0..size_1].copy_from_slice(&self.buffer[read_pos..(read_pos + size_1)]);
            read_pos = (read_pos + size_1) % capacity;

            let size_2 = bytes_to_read - size_1;
            r[size_1..bytes_to_read].copy_from_slice(&self.buffer[read_pos..(read_pos + size_2)]);
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
