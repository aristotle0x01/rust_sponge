use crate::util::file_descriptor::{AsFileDescriptor, AsFileDescriptorMut, FileDescriptor};
use crate::util::util::system_call;
use crate::SizeT;
use libc::{c_int, size_t, sockaddr, socklen_t, AF_INET, SOCK_STREAM};
use nix::sys::socket::{setsockopt, Shutdown, SockaddrIn};
use std::ffi::c_void;
use std::fmt::Debug;
use std::mem;
use std::mem::size_of_val;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::ptr::null_mut;
use std::str::FromStr;

#[derive(Debug)]
pub struct Socket {
    fd: FileDescriptor,
}
impl AsFileDescriptor for Socket {
    fn as_file_descriptor(&self) -> &FileDescriptor {
        &self.fd
    }
}
impl AsFileDescriptorMut for Socket {
    fn as_file_descriptor_mut(&mut self) -> &mut FileDescriptor {
        &mut self.fd
    }
}
impl Socket {
    #[allow(dead_code)]
    pub fn new(domain_: i32, type_: i32) -> Socket {
        let t_ = unsafe { libc::socket(domain_, type_, 0) };
        let fd_ = system_call("socket", t_ as i32, 0);
        let fd_desc = FileDescriptor::new(fd_);
        Socket { fd: fd_desc }
    }

    #[allow(dead_code)]
    pub fn new2(fd_: FileDescriptor, domain_: i32, type_: i32) -> Socket {
        let mut actual_value: i32 = 0;
        let mut len = mem::size_of::<i32>() as socklen_t;
        // ref: https://segmentfault.com/a/1190000018871370
        let t_ = unsafe {
            libc::getsockopt(
                fd_.fd_num(),
                libc::SOL_SOCKET,
                libc::SO_DOMAIN,
                &mut actual_value as *mut i32 as *mut c_void,
                &mut len,
            )
        };
        system_call("getsockopt", t_ as i32, 0);
        if len != size_of_val(&actual_value) as socklen_t || actual_value != domain_ {
            assert!(false, "socket domain mismatch");
        }

        len = mem::size_of::<i32>() as socklen_t;
        let t_ = unsafe {
            libc::getsockopt(
                fd_.fd_num(),
                libc::SOL_SOCKET,
                libc::SO_TYPE,
                &mut actual_value as *mut i32 as *mut c_void,
                &mut len,
            )
        };
        system_call("getsockopt", t_ as i32, 0);
        if len != size_of_val(&actual_value) as socklen_t || actual_value != type_ {
            assert!(false, "socket type mismatch");
        }

        Socket { fd: fd_ }
    }

    #[allow(dead_code)]
    pub fn bind2(&self, address_: &SockaddrIn) {
        let _ = nix::sys::socket::bind(self.fd_num(), address_);
    }

    #[allow(dead_code)]
    pub fn bind(&self, _host: &str, _port: u16) {
        let sin = SockaddrIn::from(SocketAddrV4::new(Ipv4Addr::from_str(_host).unwrap(), _port));
        let _ = nix::sys::socket::bind(self.fd_num(), &sin);
    }

    #[allow(dead_code)]
    pub fn connect2(&self, address_: &SockaddrIn) {
        let _ = nix::sys::socket::connect(self.fd_num(), address_);
    }

    #[allow(dead_code)]
    pub fn connect(&self, _host: &str, _port: u16) {
        let sin = SockaddrIn::from(SocketAddrV4::new(Ipv4Addr::from_str(_host).unwrap(), _port));
        let _ = nix::sys::socket::connect(self.fd_num(), &sin);
    }

    #[allow(dead_code)]
    pub fn shutdown(&mut self, how_: i32) {
        match how_ {
            libc::SHUT_RDWR => {
                nix::sys::socket::shutdown(self.fd_num(), Shutdown::Both).unwrap();
                self.register_read();
                self.register_write();
            }
            libc::SHUT_RD => {
                nix::sys::socket::shutdown(self.fd_num(), Shutdown::Read).unwrap();
                self.register_read();
            }
            libc::SHUT_WR => {
                nix::sys::socket::shutdown(self.fd_num(), Shutdown::Write).unwrap();
                self.register_write();
            }
            _ => {
                assert!(false, "Socket::shutdown() called with invalid `how`")
            }
        }
    }

    #[allow(dead_code)]
    pub fn local_address(&self) -> SockaddrIn {
        nix::sys::socket::getsockname(self.fd_num()).unwrap()
    }

    #[allow(dead_code)]
    pub fn peer_address(&self) -> SockaddrIn {
        nix::sys::socket::getpeername(self.fd_num()).unwrap()
    }

    #[allow(dead_code)]
    pub fn set_reuseaddr(&self) {
        setsockopt(self.fd_num(), nix::sys::socket::sockopt::ReuseAddr, &true).unwrap();
    }
}

pub trait AsSocket: AsFileDescriptor {
    fn as_socket(&self) -> &Socket;
    fn connect(&self, _host: &str, _port: u16) {
        self.as_socket().connect(_host, _port);
    }
    fn bind(&self, _host: &str, _port: u16) {
        self.as_socket().bind(_host, _port);
    }
}
pub trait AsSocketMut: AsFileDescriptorMut {
    fn as_socket_mut(&mut self) -> &mut Socket;
    fn set_reuseaddr(&mut self) {
        self.as_socket_mut().set_reuseaddr();
    }
    fn shutdown(&mut self, how_: i32) {
        self.as_socket_mut().shutdown(how_);
    }
}

#[derive(Debug)]
pub struct ReceivedDatagram {
    pub source_address: sockaddr,
    pub payload: Vec<u8>,
}

#[derive(Debug)]
pub struct UDPSocket(Socket);
impl AsFileDescriptor for UDPSocket {
    fn as_file_descriptor(&self) -> &FileDescriptor {
        self.as_socket().as_file_descriptor()
    }
}
impl AsSocket for UDPSocket {
    fn as_socket(&self) -> &Socket {
        &self.0
    }
}
impl AsFileDescriptorMut for UDPSocket {
    fn as_file_descriptor_mut(&mut self) -> &mut FileDescriptor {
        self.as_socket_mut().as_file_descriptor_mut()
    }
}
impl AsSocketMut for UDPSocket {
    fn as_socket_mut(&mut self) -> &mut Socket {
        &mut self.0
    }
}
impl UDPSocket {
    #[allow(dead_code)]
    pub fn new() -> UDPSocket {
        UDPSocket {
            0: Socket::new(libc::AF_INET, libc::SOCK_DGRAM),
        }
    }

    #[allow(dead_code)]
    pub fn recv(&mut self, _mtu: SizeT) -> ReceivedDatagram {
        let mut dg = ReceivedDatagram {
            source_address: unsafe { mem::zeroed() },
            payload: vec![],
        };

        self.recv_into(&mut dg, _mtu);

        dg
    }

    #[allow(dead_code)]
    pub fn recv_into(&mut self, datagram: &mut ReceivedDatagram, _mtu: SizeT) {
        datagram.payload.resize(_mtu, 0);

        let rev_len = unsafe {
            // ref: https://users.rust-lang.org/t/help-understanding-libc-call/17308/12
            let addr_ptr = &mut datagram.source_address as *mut sockaddr;
            let mut addrlen = mem::size_of_val(&datagram.source_address);
            let addrlen_ptr = &mut addrlen as *mut usize as *mut socklen_t;
            let buf_ptr = datagram.payload.as_mut_ptr() as *mut c_void;
            let buf_cap = datagram.payload.capacity() as size_t;

            libc::recvfrom(
                self.0.fd_num(),
                buf_ptr,
                buf_cap,
                libc::MSG_TRUNC,
                addr_ptr,
                addrlen_ptr,
            )
        };
        system_call("recvfrom", rev_len as i32, 0);
        if rev_len > _mtu as isize {
            assert!(false, "recvfrom (oversized datagram)");
        }

        self.register_read();

        datagram.payload.resize(rev_len as usize, 0);
    }

    #[allow(dead_code)]
    pub fn sendto2(&mut self, _destination: &mut sockaddr, _payload: &mut Vec<u8>) {
        let vecs = [libc::iovec {
            iov_base: _payload.as_mut_ptr() as *mut c_void,
            iov_len: _payload.len(),
        }; 1];
        let msg = libc::msghdr {
            msg_name: _destination as *mut _ as *mut c_void,
            msg_namelen: size_of_val(_destination) as socklen_t,
            msg_iov: vecs.as_ptr() as *mut libc::iovec,
            msg_iovlen: (vecs.len() as c_int) as usize,
            msg_control: null_mut(),
            msg_controllen: 0,
            msg_flags: 0,
        };

        let sent = unsafe { libc::sendmsg(self.fd_num(), &msg, 0) };
        system_call("sendmsg", sent as i32, 0);
        assert_eq!(
            sent,
            _payload.len() as isize,
            "datagram payload too big for sendmsg()"
        );

        self.register_write();
    }

    #[allow(dead_code)]
    pub fn sendto(&mut self, _destination: &SocketAddrV4, _payload: &mut Vec<u8>) {
        let mut sin = SockaddrIn::from(*_destination);

        let vecs = [libc::iovec {
            iov_base: _payload.as_mut_ptr() as *mut c_void,
            iov_len: _payload.len(),
        }; 1];
        let msg = libc::msghdr {
            msg_name: &mut sin as *mut _ as *mut c_void,
            msg_namelen: size_of_val(&sin) as socklen_t,
            msg_iov: vecs.as_ptr() as *mut libc::iovec,
            msg_iovlen: (vecs.len() as c_int) as usize,
            msg_control: null_mut(),
            msg_controllen: 0,
            msg_flags: 0,
        };

        let sent = unsafe { libc::sendmsg(self.fd_num(), &msg, 0) };
        system_call("sendmsg", sent as i32, 0);
        assert_eq!(
            sent,
            _payload.len() as isize,
            "datagram payload too big for sendmsg()"
        );

        self.register_write();
    }

    #[allow(dead_code)]
    pub fn send(&mut self, _payload: &mut Vec<u8>) {
        let vecs = [libc::iovec {
            iov_base: _payload.as_mut_ptr() as *mut c_void,
            iov_len: _payload.len(),
        }; 1];
        let msg = libc::msghdr {
            msg_name: null_mut(),
            msg_namelen: 0,
            msg_iov: vecs.as_ptr() as *mut libc::iovec,
            msg_iovlen: (vecs.len() as c_int) as usize,
            msg_control: null_mut(),
            msg_controllen: 0,
            msg_flags: 0,
        };

        let sent = unsafe { libc::sendmsg(self.fd_num(), &msg, 0) };
        system_call("sendmsg", sent as i32, 0);
        assert_eq!(
            sent,
            _payload.len() as isize,
            "datagram payload too big for sendmsg()"
        );

        self.register_write();
    }
}

#[derive(Debug)]
pub struct TCPSocket(Socket);
impl AsFileDescriptor for TCPSocket {
    fn as_file_descriptor(&self) -> &FileDescriptor {
        self.as_socket().as_file_descriptor()
    }
}
impl AsSocket for TCPSocket {
    fn as_socket(&self) -> &Socket {
        &self.0
    }
}
impl AsFileDescriptorMut for TCPSocket {
    fn as_file_descriptor_mut(&mut self) -> &mut FileDescriptor {
        self.as_socket_mut().as_file_descriptor_mut()
    }
}
impl AsSocketMut for TCPSocket {
    fn as_socket_mut(&mut self) -> &mut Socket {
        &mut self.0
    }
}
impl TCPSocket {
    #[allow(dead_code)]
    pub fn new() -> TCPSocket {
        TCPSocket {
            0: Socket::new(libc::AF_INET, libc::SOCK_STREAM),
        }
    }

    #[allow(dead_code)]
    pub fn listen(&self, _backlog: i32) {
        let r_ = unsafe { libc::listen(self.0.fd_num(), _backlog) };
        system_call("listen", r_ as i32, 0);
    }

    #[allow(dead_code)]
    pub fn accept(&mut self) -> TCPSocket {
        self.register_read();

        let r_ = unsafe { libc::accept(self.0.fd_num(), null_mut(), null_mut()) };
        system_call("accept", r_ as i32, 0);

        TCPSocket(Socket::new2(FileDescriptor::new(r_), AF_INET, SOCK_STREAM))
    }
}

#[derive(Debug)]
pub struct LocalStreamSocket(Socket);
impl AsFileDescriptor for LocalStreamSocket {
    fn as_file_descriptor(&self) -> &FileDescriptor {
        self.as_socket().as_file_descriptor()
    }
}
impl AsSocket for LocalStreamSocket {
    fn as_socket(&self) -> &Socket {
        &self.0
    }
}
impl AsFileDescriptorMut for LocalStreamSocket {
    fn as_file_descriptor_mut(&mut self) -> &mut FileDescriptor {
        self.as_socket_mut().as_file_descriptor_mut()
    }
}
impl AsSocketMut for LocalStreamSocket {
    fn as_socket_mut(&mut self) -> &mut Socket {
        &mut self.0
    }
}
impl LocalStreamSocket {
    #[allow(dead_code)]
    pub fn new(fd: FileDescriptor) -> LocalStreamSocket {
        LocalStreamSocket {
            0: Socket::new2(fd, libc::AF_UNIX, libc::SOCK_STREAM),
        }
    }
}
