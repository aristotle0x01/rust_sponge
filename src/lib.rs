#![deny(
    missing_debug_implementations,
    rust_2018_idioms,
    unused_imports,
    dead_code
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
// Disallow warnings when running tests.
#![cfg_attr(test, deny(warnings))]
// Disallow warnings in examples.
#![doc(test(attr(deny(warnings))))]

// macros used internally

use crate::tcp_helpers::fd_adapter::TCPOverUDPSocketAdapter;
use crate::tcp_helpers::ipv4_datagram::IPv4Datagram;
use crate::tcp_helpers::lossy_fd_adapter::LossyFdAdapter;
use crate::tcp_helpers::tcp_sponge_socket::TCPSpongeSocket;
use crate::tcp_helpers::tuntap_adapter::TCPOverIPv4OverTunFdAdapter;

pub type SizeT = usize;
pub type InternetDatagram = IPv4Datagram;
pub type LossyTCPOverUDPSocketAdapter = LossyFdAdapter<TCPOverUDPSocketAdapter>;
pub type LossyTCPOverUDPSpongeSocket = TCPSpongeSocket<LossyTCPOverUDPSocketAdapter>;
pub type LossyTCPOverIPv4OverTunFdAdapter = LossyFdAdapter<TCPOverIPv4OverTunFdAdapter>;
pub type LossyTCPOverIPv4SpongeSocket = TCPSpongeSocket<LossyTCPOverIPv4OverTunFdAdapter>;

pub mod byte_stream;
pub mod stream_reassembler;
pub mod tcp_connection;
pub mod tcp_helpers;
pub mod tcp_receiver;
pub mod tcp_sender;
pub mod util;
pub mod wrapping_integers;
