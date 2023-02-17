use crate::util::eventloop::Direction::In;
use crate::util::file_descriptor::FileDescriptor;
use libc::{c_short, nfds_t};
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Direction {
    In = libc::POLLIN as isize,
    Out = libc::POLLOUT as isize,
}
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Result {
    Success = 0,
    Timeout = 1,
    Exit = 2,
}

pub type CallbackT<'a> = Box<dyn FnMut() + 'a>;
pub type InterestT<'a> = Box<dyn Fn() -> bool + 'a>;

#[allow(dead_code)]
pub struct Rule<'a> {
    fd: Rc<RefCell<FileDescriptor>>,
    direction: Direction,
    callback: CallbackT<'a>,
    interest: InterestT<'a>,
    cancel: CallbackT<'a>,
    remove: bool,
}
impl Debug for Rule<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "fd:{} care for {} event",
            self.fd.borrow().fd_num(),
            if self.direction == In {
                "read"
            } else {
                "write"
            }
        )
    }
}
impl Rule<'_> {
    #[allow(dead_code)]
    pub fn service_count(&self) -> u32 {
        if self.direction == Direction::In {
            self.fd.borrow().read_count()
        } else {
            self.fd.borrow().write_count()
        }
    }
}

#[derive(Debug)]
pub struct EventLoop<'a> {
    rules: Vec<Rule<'a>>,
}
impl<'a> EventLoop<'a> {
    #[allow(dead_code)]
    pub fn new() -> EventLoop<'static> {
        EventLoop { rules: Vec::new() }
    }

    #[allow(dead_code)]
    pub fn add_rule(
        &mut self,
        _fd: Rc<RefCell<FileDescriptor>>,
        _direction: Direction,
        _callback: CallbackT<'a>,
        _interest: InterestT<'a>,
        _cancel: CallbackT<'a>,
    ) {
        self.rules.push(Rule {
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
            if r.direction == Direction::In && r.fd.borrow().eof() {
                (r.cancel)();
                r.remove = true;
                continue;
            }

            if r.fd.borrow().closed() {
                (r.cancel)();
                r.remove = true;
                continue;
            }

            if (r.interest)() {
                pollfds.push(libc::pollfd {
                    fd: r.fd.borrow().fd_num(),
                    events: r.direction as c_short,
                    revents: 0,
                });
                something_to_poll = true;
            } else {
                pollfds.push(libc::pollfd {
                    fd: r.fd.borrow().fd_num(),
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
