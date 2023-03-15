use crate::util::file_descriptor::{AsFileDescriptor, AsFileDescriptorMut, FileDescriptor};
use crate::util::util::system_call;
use libc::{c_char, c_int, c_ulong};
use std::fmt::Debug;

// ref: copied smoltcp tun implementation

pub const TUNSETIFF: c_ulong = 0x400454CA;
pub const IFF_TUN: c_int = 0x0001;
pub const IFF_TAP: c_int = 0x0002;
pub const IFF_NO_PI: c_int = 0x1000;

#[repr(C)]
#[derive(Debug)]
struct ifreq {
    ifr_name: [c_char; libc::IF_NAMESIZE],
    ifr_data: c_int, /* ifr_ifindex or ifr_mtu */
}

// A FileDescriptor to a [Linux TUN/TAP](https://www.kernel.org/doc/Documentation/networking/tuntap.txt) device
// Tun/Tap interface tutorial (https://backreference.org/2010/03/26/tuntap-interface-tutorial/index.html)
// Virtual Networking Devices - TUN, TAP and VETH Pairs Explained (https://www.packetcoders.io/virtual-networking-devices-tun-tap-and-veth-pairs-explained/)
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

impl TunTapFD {
    #[allow(dead_code)]
    pub fn new(devname_: &str, is_tun_: bool) -> TunTapFD {
        let t_ = unsafe { libc::open("/dev/net/tun\0".as_ptr() as *const c_char, libc::O_RDWR) };
        let fd_ = system_call("open", t_ as i32, 0);
        let fd_desc = FileDescriptor::new(fd_);

        let mut ifreq_ = ifreq {
            ifr_name: [0; libc::IF_NAMESIZE],
            ifr_data: 0,
        };
        ifreq_.ifr_data = (if is_tun_ { IFF_TUN } else { IFF_TAP } | IFF_NO_PI);
        for (i, byte) in devname_.as_bytes().iter().enumerate() {
            ifreq_.ifr_name[i] = *byte as c_char;
        }
        let io_ =
            unsafe { libc::ioctl(fd_desc.fd_num(), TUNSETIFF as _, &mut ifreq_ as *mut ifreq) };
        system_call("ioctl", io_ as i32, 0);

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
