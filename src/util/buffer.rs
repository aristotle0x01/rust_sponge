use crate::SizeT;
use std::collections::VecDeque;

// semantics of c++ std::move() std::string &str
// https://stackoverflow.com/questions/3413470/what-is-stdmove-and-when-should-it-be-used
// https://stackoverflow.com/questions/5816719/difference-between-function-arguments-declared-with-and-in-c

// Convenient and idiomatic conversions in Rust
// https://ricardomartins.cc/2016/08/03/convenient_and_idiomatic_conversions_in_rust

// What is the difference between [u8] and Vec<u8> on rust?
// https://stackoverflow.com/questions/71377731/what-is-the-difference-between-u8-and-vecu8-on-rust
#[derive(Debug)]
pub struct Buffer {
    storage: Vec<u8>,
    starting_offset: SizeT,
}
impl Buffer {
    pub const EMPTY: &'static str = "";
    pub const EMPTY_VEC: &'static Vec<u8> = &Vec::new();

    #[allow(dead_code)]
    pub fn new(_bytes: Vec<u8>) -> Buffer {
        Buffer {
            storage: _bytes,
            starting_offset: 0,
        }
    }

    // since rust char is 4 bytes instead of one in c/c++
    // hereby [u8] array will be used in place of c++ String
    // besides, for network protocols it's mainly a bytes thing,
    // not a string thing
    #[allow(dead_code)]
    pub fn str(&self) -> &[u8] {
        if self.storage.is_empty() {
            return Buffer::EMPTY_VEC;
        }
        &self.storage[self.starting_offset..self.storage.len()]
    }

    #[allow(dead_code)]
    pub fn at(&self, n: SizeT) -> u8 {
        *self.str().get(n).unwrap()
    }

    #[allow(dead_code)]
    pub fn size(&self) -> SizeT {
        self.str().len()
    }

    #[allow(dead_code)]
    pub fn copy(&self) -> String {
        String::from_utf8(self.str().to_vec()).unwrap()
    }

    #[allow(dead_code)]
    pub fn remove_prefix(&mut self, n: SizeT) {
        if n > self.str().len() {
            panic!("Buffer::remove_prefix");
        }
        self.starting_offset += n;
        if !self.storage.is_empty() && self.starting_offset == self.storage.len() {
            // todo: is move possible? clear may suffice
            self.storage.clear();
        }
    }
}

#[derive(Debug)]
pub struct BufferList {
    buffers: VecDeque<Buffer>,
}
impl BufferList {
    #[allow(dead_code)]
    pub fn new(_buffer: Buffer) -> BufferList {
        let mut t: VecDeque<Buffer> = VecDeque::new();
        t.push_back(_buffer);
        BufferList { buffers: t }
    }

    #[allow(dead_code)]
    pub fn new0() -> BufferList {
        BufferList {
            buffers: Default::default(),
        }
    }

    #[allow(dead_code)]
    pub fn new_from_str(s: String) -> BufferList {
        let buffer = Buffer::new(s.as_bytes().to_vec());
        let mut t: VecDeque<Buffer> = VecDeque::new();
        t.push_back(buffer);
        BufferList { buffers: t }
    }

    #[allow(dead_code)]
    pub fn buffers(&self) -> &VecDeque<Buffer> {
        &self.buffers
    }

    #[allow(dead_code)]
    pub fn remove_prefix(&mut self, _n: SizeT) {
        let mut n = _n;

        loop {
            if n <= 0 {
                break;
            }

            if self.buffers.is_empty() {
                panic!("BufferList::remove_prefix")
            }

            if n < self.buffers.front().unwrap().str().len() {
                let mut buf = self.buffers.pop_front().unwrap();
                buf.remove_prefix(n);
                self.buffers.push_front(buf);
                n = 0
            } else {
                n -= self.buffers.front().unwrap().str().len();
                self.buffers.pop_front();
            }
        }
    }

    #[allow(dead_code)]
    pub fn size(&self) -> SizeT {
        let mut size: SizeT = 0;
        for _buf in self.buffers.iter() {
            size += _buf.size()
        }
        size
    }

    #[allow(dead_code)]
    pub fn concatenate(&self) -> String {
        let mut s = String::new();
        for _buf in self.buffers.iter() {
            s.push_str(String::from_utf8(_buf.str().to_vec()).unwrap().as_str());
        }

        s
    }

    #[allow(dead_code)]
    pub fn append(&mut self, other: &BufferList) {
        for buf in other.buffers() {
            // https://en.cppreference.com/w/cpp/container/deque/push_back
            // push_back( const T& value ): The new element is initialized as a copy of value
            // todo: copy is plausible
            self.buffers.push_back(Buffer::new(
                String::from_utf8(buf.str().to_vec())
                    .unwrap()
                    .as_bytes()
                    .to_vec(),
            ));
        }
    }
}
impl From<Buffer> for BufferList {
    fn from(buf: Buffer) -> Self {
        BufferList::new(buf)
    }
}
impl AsRef<Buffer> for BufferList {
    fn as_ref(&self) -> &Buffer {
        match self.buffers.len() {
            1 => self.buffers().front().unwrap(),
            _ => panic!("BufferList: please use concatenate() to combine a multi-Buffer BufferList into one Buffer"),
        }
    }
}
