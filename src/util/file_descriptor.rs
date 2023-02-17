use crate::util::util::system_call;
use crate::SizeT;
use std::cell::RefCell;
use std::cmp::min;
use std::ffi::c_void;
use std::rc::Rc;

#[derive(Debug)]
struct FDWrapper {
    fd: i32,
    eof: bool,
    closed: bool,
    read_count: u32,
    write_count: u32,
}
impl FDWrapper {
    #[allow(dead_code)]
    pub fn new(_fd: i32) -> FDWrapper {
        assert!(_fd >= 0, "{}", format!("invalid fd number:{}", _fd));
        FDWrapper {
            fd: _fd,
            eof: false,
            closed: false,
            read_count: 0,
            write_count: 0,
        }
    }

    #[allow(dead_code)]
    pub fn close(&mut self) {
        let ret = unsafe { libc::close(self.fd) };
        system_call("close", ret, 0);
        self.closed = true;
        self.eof = true;
    }
}
impl Drop for FDWrapper {
    fn drop(&mut self) {
        if self.closed {
            return;
        }
        self.close();
    }
}

#[derive(Debug)]
pub struct FileDescriptor {
    // https://web.mit.edu/rust-lang_v1.25/arch/amd64_ubuntu1404/share/doc/rust/html/book/second-edition/ch15-05-interior-mutability.html#having-multiple-owners-of-mutable-data-by-combining-rct-and-refcellt
    internal_fd: Rc<RefCell<FDWrapper>>,
}
impl FileDescriptor {
    #[allow(dead_code)]
    pub fn new(_fd: i32) -> FileDescriptor {
        FileDescriptor {
            internal_fd: Rc::new(RefCell::new(FDWrapper {
                fd: _fd,
                eof: false,
                closed: false,
                read_count: 0,
                write_count: 0,
            })),
        }
    }

    #[allow(dead_code)]
    pub fn register_read(&mut self) {
        self.internal_fd.borrow_mut().read_count += 1;
    }

    #[allow(dead_code)]
    pub fn register_write(&mut self) {
        self.internal_fd.borrow_mut().write_count += 1;
    }

    #[allow(dead_code)]
    pub fn read(&mut self, _limit: u32) -> String {
        let mut s: Vec<u8> = Vec::with_capacity(_limit as usize);
        self.read_into(&mut s, _limit);

        String::from_utf8(s).unwrap()
    }

    #[allow(dead_code)]
    pub fn read_into(&mut self, _buf: &mut Vec<u8>, _limit: u32) {
        let buffer_size: SizeT = 1024 * 1024;
        let size_to_read: SizeT = min(buffer_size, _limit as SizeT);
        _buf.shrink_to(size_to_read);

        let bytes_read = unsafe {
            libc::read(
                self.fd_num(),
                _buf.as_mut_ptr() as *mut c_void,
                size_to_read,
            )
        };
        system_call("read", bytes_read as i32, 0);
        unsafe {
            // important to set len since libc::read only write to pointer
            _buf.set_len(bytes_read as usize);
        }

        if _limit > 0 && bytes_read == 0 {
            self.internal_fd.borrow_mut().eof = true;
        }
        if bytes_read > size_to_read as isize {
            panic!("read() read more than requested");
        }
        _buf.shrink_to(bytes_read as usize);

        self.register_read();
    }

    #[allow(dead_code)]
    pub fn write(&mut self, _buf: &String, _write_all: bool) -> SizeT {
        let mut total_bytes_written = 0;

        let mut first = true;
        while first || (_write_all && total_bytes_written < _buf.len()) {
            first = false;

            let to_write = _buf.len() - total_bytes_written;
            // todo: not the original "writev"
            let bytes_written = unsafe {
                libc::write(
                    self.fd_num(),
                    _buf[total_bytes_written..].as_ptr() as *const c_void,
                    to_write,
                )
            };
            system_call("write", bytes_written as i32, 0);

            if bytes_written == 0 && to_write != 0 {
                panic!("write returned 0 given non-empty input buffer");
            }
            if bytes_written > to_write as isize {
                panic!("write wrote more than length of input buffer");
            }

            self.register_write();

            total_bytes_written += bytes_written as usize;
        }

        return total_bytes_written;
    }

    #[allow(dead_code)]
    pub fn close(&mut self) {
        self.internal_fd.borrow_mut().close();
    }

    #[allow(dead_code)]
    pub fn set_blocking(&mut self, _blocking_state: bool) {
        let mut flags = unsafe { libc::fcntl(self.fd_num(), libc::F_GETFL) };
        system_call("fcntl", flags, 0);
        if _blocking_state {
            flags ^= flags & libc::O_NONBLOCK;
        } else {
            flags |= libc::O_NONBLOCK;
        }

        let ret = unsafe { libc::fcntl(self.fd_num(), libc::F_SETFL, flags) };
        system_call("fcntl", ret, 0);
    }

    #[allow(dead_code)]
    pub fn fd_num(&self) -> i32 {
        self.internal_fd.borrow().fd
    }

    #[allow(dead_code)]
    pub fn eof(&self) -> bool {
        self.internal_fd.borrow().eof
    }

    #[allow(dead_code)]
    pub fn closed(&self) -> bool {
        self.internal_fd.borrow().closed
    }

    #[allow(dead_code)]
    pub fn read_count(&self) -> u32 {
        self.internal_fd.borrow().read_count
    }

    #[allow(dead_code)]
    pub fn write_count(&self) -> u32 {
        self.internal_fd.borrow().write_count
    }
}
impl Clone for FileDescriptor {
    fn clone(&self) -> FileDescriptor {
        FileDescriptor {
            internal_fd: self.internal_fd.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.internal_fd = source.internal_fd.clone();
    }
}

// https://users.rust-lang.org/t/how-to-implement-inheritance-like-feature-for-rust/31159/21
pub trait AsFileDescriptor {
    fn as_file_descriptor(&self) -> &FileDescriptor;

    fn fd_num(&self) -> i32 {
        self.as_file_descriptor().fd_num()
    }
}
pub trait AsFileDescriptorMut: AsFileDescriptor {
    fn as_file_descriptor_mut(&mut self) -> &mut FileDescriptor;

    fn register_read(&mut self) {
        self.as_file_descriptor_mut().register_read();
    }

    fn register_write(&mut self) {
        self.as_file_descriptor_mut().register_write();
    }

    fn read(&mut self, _limit: u32) -> String {
        self.as_file_descriptor_mut().read(_limit)
    }

    fn write(&mut self, _buf: &String, _write_all: bool) -> SizeT {
        self.as_file_descriptor_mut().write(_buf, _write_all)
    }

    fn close(&mut self) {
        self.as_file_descriptor_mut().close();
    }

    fn set_blocking(&mut self, _blocking_state: bool) {
        self.as_file_descriptor_mut().set_blocking(_blocking_state);
    }
}
