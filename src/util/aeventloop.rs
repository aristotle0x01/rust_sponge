use crate::util::eventloop::Direction::In;
use crate::util::eventloop::{Direction, Result};
use crate::util::file_descriptor::FileDescriptor;
use libc::{c_short, nfds_t};
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};

pub type ACallbackT = Box<dyn FnMut() + Send>;
pub type AInterestT = Box<dyn Fn() -> bool + Send>;

#[allow(dead_code)]
pub struct ARule {
    fd: Arc<Mutex<FileDescriptor>>,
    direction: Direction,
    callback: ACallbackT,
    interest: AInterestT,
    cancel: ACallbackT,
    remove: bool,
}
impl Debug for ARule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let fd_ = self.fd.lock().unwrap();
        write!(
            f,
            "fd:{} care for {} event",
            fd_.fd_num(),
            if self.direction == In {
                "read"
            } else {
                "write"
            }
        )
    }
}
impl ARule {
    #[allow(dead_code)]
    pub fn service_count(&self) -> u32 {
        let fd_ = self.fd.lock().unwrap();
        if self.direction == Direction::In {
            fd_.read_count()
        } else {
            fd_.write_count()
        }
    }
}

#[derive(Debug)]
pub struct AEventLoop {
    rules: Vec<ARule>,
}
impl AEventLoop {
    #[allow(dead_code)]
    pub fn new() -> AEventLoop {
        AEventLoop { rules: Vec::new() }
    }

    #[allow(dead_code)]
    pub fn add_rule(
        &mut self,
        _fd: Arc<Mutex<FileDescriptor>>,
        _direction: Direction,
        _callback: ACallbackT,
        _interest: AInterestT,
        _cancel: ACallbackT,
    ) {
        self.rules.push(ARule {
            fd: _fd,
            direction: _direction,
            callback: _callback,
            interest: _interest,
            cancel: _cancel,
            remove: false,
        });
    }

    #[allow(dead_code)]
    pub fn wait_next_event(&mut self, timeout_ms: i32) -> Result {
        let mut pollfds: Vec<libc::pollfd> = Vec::with_capacity(self.rules.len());
        let mut something_to_poll = false;

        // set up the pollfd for each rule
        for (_, r) in self.rules.iter_mut().enumerate() {
            let fd_ = r.fd.lock().unwrap();

            if r.direction == Direction::In && fd_.eof() {
                (r.cancel)();
                r.remove = true;
                continue;
            }

            if fd_.closed() {
                (r.cancel)();
                r.remove = true;
                continue;
            }

            if (r.interest)() {
                pollfds.push(libc::pollfd {
                    fd: fd_.fd_num(),
                    events: r.direction as c_short,
                    revents: 0,
                });
                something_to_poll = true;
            } else {
                pollfds.push(libc::pollfd {
                    fd: fd_.fd_num(),
                    events: 0,
                    revents: 0,
                });
            }
        }
        self.rules.retain(|r| r.remove == false);

        if !something_to_poll {
            return Result::Exit;
        }

        // do the polling
        let ret_ = unsafe { libc::poll(pollfds.as_mut_ptr(), pollfds.len() as nfds_t, timeout_ms) };
        if ret_ == 0 {
            return Result::Timeout;
        }
        if ret_ == -1 && std::io::Error::last_os_error().raw_os_error().unwrap_or(0) == libc::EINTR
        {
            return Result::Exit;
        }

        // go through the poll results
        for (i, r) in self.rules.iter_mut().enumerate() {
            let this_pollfd = pollfds[i];
            let poll_error = this_pollfd.revents & (libc::POLLERR | libc::POLLNVAL);
            assert_eq!(poll_error, 0, "EventLoop: error on polled file descriptor");

            let poll_ready = this_pollfd.revents & this_pollfd.events;
            let poll_hup = this_pollfd.revents & libc::POLLHUP;
            if poll_hup != 0 && this_pollfd.events != 0 && poll_ready == 0 {
                (r.cancel)();
                r.remove = true;
                continue;
            }

            if poll_ready != 0 {
                let count_before = r.service_count();
                (r.callback)();

                if count_before == r.service_count() && (r.interest)() {
                    assert!(false, "EventLoop: busy wait detected: callback did not read/write fd and is still interested");
                }
            }
        }
        self.rules.retain(|r| r.remove == false);

        Result::Success
    }
}
