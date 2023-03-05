use crate::SizeT;
use std::collections::VecDeque;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

// type conversion and Deref Trait
// https://doc.rust-lang.org/book/ch15-02-deref.html

// semantics of c++ std::move() std::string &str
// https://stackoverflow.com/questions/3413470/what-is-stdmove-and-when-should-it-be-used
// https://stackoverflow.com/questions/5816719/difference-between-function-arguments-declared-with-and-in-c

// Convenient and idiomatic conversions in Rust
// https://ricardomartins.cc/2016/08/03/convenient_and_idiomatic_conversions_in_rust

// What is the difference between [u8] and Vec<u8> on rust?
// https://stackoverflow.com/questions/71377731/what-is-the-difference-between-u8-and-vecu8-on-rust

// Returning a mutable reference to a value behind Arc and Mutex
// https://stackoverflow.com/questions/66726259/returning-a-mutable-reference-to-a-value-behind-arc-and-mutex
#[derive(Debug)]
pub struct Buffer {
    storage: Vec<u8>,
    starting_offset: SizeT,
}
impl Buffer {
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
            &self.storage[0..0]
        } else {
            &self.storage[self.starting_offset..self.storage.len()]
        }
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
    pub fn len(&self) -> SizeT {
        self.str().len()
    }

    #[allow(dead_code)]
    pub fn remove_prefix(&mut self, n: SizeT) {
        if n > self.str().len() {
            panic!("Buffer::remove_prefix");
        }
        self.starting_offset += n;
        if !self.storage.is_empty() && self.starting_offset == self.storage.len() {
            self.storage.clear();
            assert!(self.storage.is_empty());
        }
    }
}
impl Clone for Buffer {
    fn clone(&self) -> Buffer {
        Buffer {
            storage: self.storage.clone(),
            starting_offset: self.starting_offset,
        }
    }
}
impl From<String> for Buffer {
    fn from(s: String) -> Self {
        Buffer {
            storage: Vec::from(s),
            starting_offset: 0,
        }
    }
}
impl From<Vec<u8>> for Buffer {
    fn from(v: Vec<u8>) -> Self {
        Buffer {
            storage: v,
            starting_offset: 0,
        }
    }
}
impl Deref for Buffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.storage[self.starting_offset..self.storage.len()]
    }
}
impl DerefMut for Buffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let len = self.storage.len() as SizeT;
        &mut self.storage[self.starting_offset..len]
    }
}

#[derive(Debug)]
pub struct BufferList {
    buffers: VecDeque<Arc<Buffer>>,
}
impl BufferList {
    #[allow(dead_code)]
    pub fn new(_buffer: Buffer) -> BufferList {
        let mut t: VecDeque<Arc<Buffer>> = VecDeque::new();
        t.push_back(Arc::new(_buffer));
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
        let buffer = Arc::new(Buffer::new(s.as_bytes().to_vec()));
        let mut t: VecDeque<Arc<Buffer>> = VecDeque::new();
        t.push_back(buffer);
        BufferList { buffers: t }
    }

    #[allow(dead_code)]
    pub fn buffers(&self) -> &VecDeque<Arc<Buffer>> {
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
                Arc::make_mut(&mut buf).remove_prefix(n);
                self.buffers.push_front(buf);
                n = 0;
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
            // but c++ Buffer inner storage is a shared_ptr, thus is a shallow-copy
            self.buffers.push_back(buf.clone());
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

// Test Organization
// https://web.mit.edu/rust-lang_v1.25/arch/amd64_ubuntu1404/share/doc/rust/html/book/second-edition/ch11-03-test-organization.html
// https://doc.rust-lang.org/cargo/commands/cargo-test.html
#[cfg(test)]
mod tests {
    use crate::util::buffer::Buffer;

    fn deref_(b: &[u8]) {
        println!("{}", String::from_utf8_lossy(b));
    }

    fn deref_mut_(b: &mut [u8]) {
        println!("before:{}", String::from_utf8_lossy(b));
        let c = vec![49; b.len()];
        b.copy_from_slice(&c);
        println!("after:{}", String::from_utf8_lossy(b));
    }

    // cargo test --lib test_deref
    #[test]
    fn test_deref() {
        let b = Buffer::new("123".to_string().into_bytes());
        deref_(&b);
    }

    #[test]
    fn test_deref_mut() {
        let mut b = Buffer::new("123".to_string().into_bytes());
        deref_mut_(&mut b);
    }

    #[test]
    fn test_empty_str() {
        let b = Buffer::new(Vec::new());
        let s = b.str();
        assert_eq!(b.starting_offset, 0);
        assert_eq!(b.storage.len(), 0);
        assert!(s.is_empty());
    }

    // cargo test --lib test_remove_prefix -- --show-output
    #[test]
    #[should_panic]
    fn test_remove_prefix() {
        let mut b = Buffer::new("123".to_string().into_bytes());
        b.remove_prefix(1);
        assert_eq!(b.str().len(), 2);
        b.remove_prefix(2);
        assert_eq!(b.str().len(), 0);
        b.remove_prefix(1);
    }
}
