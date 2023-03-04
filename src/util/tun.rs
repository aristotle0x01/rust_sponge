use crate::util::file_descriptor::{AsFileDescriptor, AsFileDescriptorMut, FileDescriptor};
use crate::util::util::system_call;
use libc::{c_char, c_int, c_short};
use nix::ioctl_write_ptr;
use std::fmt::{Debug, Formatter};

pub const IFF_TAP: c_short = 0x0002;
pub const IFF_NO_PI: c_short = 0x1000;

const INTERFACE_NAME_SIZE: usize = 16;
const INTERFACE_REQUEST_UNION_SIZE: usize = 24;

// ref: https://gist.github.com/tjamaan/37350a418f37b5a7fc77e4a6956b0cee
//      https://users.rust-lang.org/t/cant-get-tun-set-iff-ioctl-to-execute-properly/14485/3
//      https://hermanradtke.com/2016/03/17/unions-rust-ffi.html/
//      https://github.com/hjr3/carp-rs/blob/5d56a62b1a698949a7252db637d3fbeadbb62e3b/src/mac.rs

#[repr(C)]
#[derive(Default, Debug)]
pub struct InterfaceRequest {
    pub interface_name: [u8; INTERFACE_NAME_SIZE],
    pub union: InterfaceRequestUnion,
}
impl InterfaceRequest {
    pub fn with_interface_name(name: &str) -> Self {
        let mut interface_request: Self = Default::default();
        interface_request.set_interface_name(name);
        interface_request
    }

    pub fn set_interface_name(&mut self, _name: &str) {
        let mut name = Vec::from(_name);
        if name.len() < INTERFACE_NAME_SIZE {
            name.resize(INTERFACE_NAME_SIZE, 0);
        } else {
            panic!("interface name too long");
        }
        name[INTERFACE_NAME_SIZE - 1] = 0;

        assert_eq!(name.len(), INTERFACE_NAME_SIZE);
        self.interface_name.clone_from_slice(&name);
    }
}

#[repr(C)]
pub union InterfaceRequestUnion {
    pub data: [u8; INTERFACE_REQUEST_UNION_SIZE],
    pub flags: c_short,
}
impl Default for InterfaceRequestUnion {
    fn default() -> Self {
        InterfaceRequestUnion {
            data: Default::default(),
        }
    }
}
impl Debug for InterfaceRequestUnion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Flags {}", unsafe { self.flags })
    }
}

const TUN_IOC_MAGIC: u8 = 'T' as u8;
const TUN_IOC_SET_IFF: u8 = 202;
ioctl_write_ptr!(
    tun_set_iff,
    TUN_IOC_MAGIC,
    TUN_IOC_SET_IFF,
    InterfaceRequest
);

// A FileDescriptor to a [Linux TUN/TAP](https://www.kernel.org/doc/Documentation/networking/tuntap.txt) device
#[derive(Debug)]
pub struct TunTapFD {
    fd: FileDescriptor,
}
impl AsFileDescriptor for TunTapFD {
    fn as_file_descriptor(&self) -> &FileDescriptor {
        &self.fd
    }
}
impl AsFileDescriptorMut for TunTapFD {
    fn as_file_descriptor_mut(&mut self) -> &mut FileDescriptor {
        &mut self.fd
    }
}

const CLONEDEV: &str = "/dev/net/tun\0";
impl TunTapFD {
    #[allow(dead_code)]
    pub fn new(devname_: &String, is_tun_: bool) -> TunTapFD {
        let t_ = unsafe { libc::open(CLONEDEV.as_ptr() as *const c_char, libc::O_RDWR) };

        let fd_ = system_call("open", t_ as i32, 0);
        let fd_desc = FileDescriptor::new(fd_);

        let mut ifr = InterfaceRequest::with_interface_name(devname_);
        ifr.union.flags = (if is_tun_ {
            libc::IFF_TUN
        } else {
            libc::IFF_TAP
        } | libc::IFF_NO_PI) as i16;
        let ctl_ = unsafe {
            libc::ioctl(
                fd_desc.fd_num(),
                nix::request_code_write!(
                    TUN_IOC_MAGIC,
                    TUN_IOC_SET_IFF,
                    std::mem::size_of::<c_int>()
                ),
                // req,
                &mut ifr as *mut _,
            )
        };
        system_call("ioctl", ctl_ as i32, 0);

        TunTapFD { fd: fd_desc }
    }
}

pub trait AsTunTapFD: AsFileDescriptor {
    fn as_tun_tap(&self) -> &TunTapFD;

    fn as_file_descriptor(&self) -> &FileDescriptor {
        &self.as_tun_tap().fd
    }
}
pub trait AsTunTapFDMut: AsFileDescriptorMut {
    fn as_tun_tap_mut(&mut self) -> &mut TunTapFD;

    fn as_file_descriptor_mut(&mut self) -> &mut FileDescriptor {
        self.as_tun_tap_mut().as_file_descriptor_mut()
    }
}

#[derive(Debug)]
pub struct TunFD {
    tun_fd: TunTapFD,
}

impl AsFileDescriptor for TunFD {
    fn as_file_descriptor(&self) -> &FileDescriptor {
        &self.tun_fd.fd
    }
}
impl AsTunTapFD for TunFD {
    fn as_tun_tap(&self) -> &TunTapFD {
        &self.tun_fd
    }
}
impl AsFileDescriptorMut for TunFD {
    fn as_file_descriptor_mut(&mut self) -> &mut FileDescriptor {
        &mut self.tun_fd.fd
    }
}
impl AsTunTapFDMut for TunFD {
    fn as_tun_tap_mut(&mut self) -> &mut TunTapFD {
        &mut self.tun_fd
    }
}
impl TunFD {
    #[allow(dead_code)]
    pub fn new(devname_: &str) -> TunFD {
        TunFD {
            tun_fd: TunTapFD::new(&devname_.to_string(), true),
        }
    }
}

#[derive(Debug)]
pub struct TapFD {
    tap_fd: TunTapFD,
}
impl AsFileDescriptor for TapFD {
    fn as_file_descriptor(&self) -> &FileDescriptor {
        &self.tap_fd.fd
    }
}
impl AsTunTapFD for TapFD {
    fn as_tun_tap(&self) -> &TunTapFD {
        &self.tap_fd
    }
}
impl AsFileDescriptorMut for TapFD {
    fn as_file_descriptor_mut(&mut self) -> &mut FileDescriptor {
        &mut self.tap_fd.fd
    }
}
impl AsTunTapFDMut for TapFD {
    fn as_tun_tap_mut(&mut self) -> &mut TunTapFD {
        &mut self.tap_fd
    }
}
impl TapFD {
    #[allow(dead_code)]
    pub fn new(devname_: &str) -> TapFD {
        TapFD {
            tap_fd: TunTapFD::new(&devname_.to_string(), false),
        }
    }
}
