// use crate::tcp_connection::TCPConnection;
// use crate::tcp_helpers::fd_adapter::AsFdAdapterBaseMut;
// use crate::tcp_helpers::tcp_config::{FdAdapterConfig, TCPConfig};
// use crate::tcp_helpers::tcp_state::{State, TCPState};
// use crate::util::eventloop::{Direction, InterestT};
// use crate::util::file_descriptor::{AsFileDescriptor, AsFileDescriptorMut, FileDescriptor};
// use crate::util::socket::{AsSocketMut, LocalStreamSocket, Socket};
// use crate::util::util::{system_call, timestamp_ms};
// use crate::SizeT;
// use libc::{SHUT_RDWR, SHUT_WR};
// use std::cmp::min;
// use std::fmt::Debug;
// use std::sync::{Arc, Mutex};
// use std::thread;
// use std::thread::JoinHandle;
// use crate::util::aeventloop::AEventLoop;
// use crate::util::eventloop::Result::Exit;
//
// // Mutate from multiple threads without interior mutability?
// //      let file = Arc::new(Mutex::new(File::create("foo.txt").unwrap()));
// // https://users.rust-lang.org/t/mutate-from-multiple-threads-without-interior-mutability/68896
//
// #[derive(Debug)]
// pub struct TCPSpongeSocket<'a, AdapterT> {
//     main_thread_data: LocalStreamSocket,
//     thread_data: LocalStreamSocket,
//     datagram_adapter: AdapterT,
//     tcp: Option<TCPConnection>,
//     eventloop: AEventLoop<'a>,
//     tcp_thread: Option<JoinHandle<()>>,
//     abort: bool,
//     inbound_shutdown: bool,
//     outbound_shutdown: bool,
//     fully_acked: bool,
// }
//
// impl<AdapterT> AsFileDescriptor for TCPSpongeSocket<'_, AdapterT> {
//     fn as_file_descriptor(&self) -> &FileDescriptor {
//         self.main_thread_data.as_file_descriptor()
//     }
// }
// impl<AdapterT> AsFileDescriptorMut for TCPSpongeSocket<'_, AdapterT> {
//     fn as_file_descriptor_mut(&mut self) -> &mut FileDescriptor {
//         self.main_thread_data.as_file_descriptor_mut()
//     }
// }
// impl<AdapterT> AsSocketMut for TCPSpongeSocket<'_, AdapterT> {
//     fn as_socket_mut(&mut self) -> &mut Socket {
//         self.main_thread_data.as_socket_mut()
//     }
//
//     fn set_reuseaddr(&mut self) {
//         self.as_socket_mut().set_reuseaddr();
//     }
//
//     fn shutdown(&mut self, how_: i32) {
//         self.as_socket_mut().shutdown(how_);
//     }
// }
// impl<AdapterT> Drop for TCPSpongeSocket<'_, AdapterT> {
//     fn drop(&mut self) {
//         if self.abort == false {
//             eprintln!("Warning: unclean shutdown of TCPSpongeSocket");
//             self.abort = true;
//             self.tcp_thread
//                 .unwrap()
//                 .join()
//                 .expect("TCPSpongeSocket join during Drop");
//         }
//     }
// }
// impl<AdapterT> TCPSpongeSocket<'_, AdapterT>
// where
//     AdapterT: AsFdAdapterBaseMut + AsFileDescriptorMut + Send,
// {
//     pub const TCP_TICK_MS: SizeT = 10;
//
//     #[allow(dead_code)]
//     pub fn new(_adapter: AdapterT) -> TCPSpongeSocket<'static, AdapterT> {
//         // socketpair: https://stackoverflow.com/questions/11461106/socketpair-in-c-unix
//         let mut socks = [0; 2];
//         let ret =
//             unsafe { libc::socketpair(libc::AF_UNIX, libc::SOCK_STREAM, 0, socks.as_mut_ptr()) };
//         system_call("socketpair", ret as i32, 0);
//
//         TCPSpongeSocket::new2(
//             (FileDescriptor::new(socks[0]), FileDescriptor::new(socks[1])),
//             _adapter,
//         )
//     }
//
//     #[allow(dead_code)]
//     fn new2(
//         pair: (FileDescriptor, FileDescriptor),
//         _adapter: AdapterT,
//     ) -> TCPSpongeSocket<'static, AdapterT> {
//         let mut t = TCPSpongeSocket {
//             main_thread_data: LocalStreamSocket::new(pair.0),
//             thread_data: LocalStreamSocket::new(pair.1),
//             datagram_adapter: _adapter,
//             tcp: None,
//             eventloop: AEventLoop::new(),
//             tcp_thread: None,
//             abort: false,
//             inbound_shutdown: false,
//             outbound_shutdown: false,
//             fully_acked: false,
//         };
//         t.thread_data.set_blocking(false);
//
//         t
//     }
//
//     #[allow(dead_code)]
//     fn initialize_tcp(&mut self, config: &TCPConfig) {
//         let _ = self.tcp.insert(TCPConnection::new(config.clone()));
//
//         let datagram_adapter_rc = Arc::new(Mutex::new(
//             self.datagram_adapter.as_file_descriptor().clone(),
//         ));
//         let thread_data_rc = Arc::new(Mutex::new(self.thread_data.as_file_descriptor().clone()));
//
//         // rule 1: read from filtered packet stream and dump into TCPConnection
//         self.eventloop.add_rule(
//             datagram_adapter_rc.clone(),
//             Direction::In,
//             Box::new(|| {
//                 let seg = <AdapterT as AsFdAdapterBaseMut>::read(&mut self.datagram_adapter);
//                 if seg.is_some() {
//                     self.tcp.unwrap().segment_received(&seg.unwrap());
//                 }
//
//                 if self.thread_data.eof()
//                     && self.tcp.unwrap().bytes_in_flight() == 0
//                     && !self.fully_acked
//                 {
//                     eprintln!(
//                         "DEBUG: Outbound stream to {} has been fully acknowledged.",
//                         self.datagram_adapter.config().destination.to_string()
//                     );
//                     self.fully_acked = true;
//                 }
//             }),
//             Box::new(|| self.tcp.unwrap().active()),
//             Box::new(|| {}),
//         );
//
//         // rule 2: read from pipe into outbound buffer
//         self.eventloop.add_rule(
//             thread_data_rc.clone(),
//             Direction::In,
//             Box::new(|| {
//                 let data = self
//                     .thread_data
//                     .read(self.tcp.unwrap().remaining_outbound_capacity() as u32);
//                 let len = data.len();
//                 let amount_written = self.tcp.unwrap().write(&data);
//                 assert_eq!(
//                     amount_written, len,
//                     "TCPConnection::write() accepted less than advertised length"
//                 );
//
//                 if self.thread_data.eof() {
//                     self.tcp.unwrap().end_input_stream();
//                     self.outbound_shutdown = true;
//
//                     eprintln!(
//                         "DEBUG: Outbound stream to {} finished ({} byte{} still in flight).",
//                         self.datagram_adapter.config().destination.to_string(),
//                         self.tcp.unwrap().bytes_in_flight(),
//                         if 1 == self.tcp.unwrap().bytes_in_flight() {
//                             ""
//                         } else {
//                             "s"
//                         }
//                     );
//                 }
//             }),
//             Box::new(|| {
//                 self.tcp.unwrap().active()
//                     && !self.outbound_shutdown
//                     && (self.tcp.unwrap().remaining_outbound_capacity() > 0)
//             }),
//             Box::new(|| {
//                 self.tcp.unwrap().end_input_stream();
//                 self.outbound_shutdown = true;
//             }),
//         );
//
//         // rule 3: read from inbound buffer into pipe
//         self.eventloop.add_rule(
//             thread_data_rc.clone(),
//             Direction::Out,
//             Box::new(|| {
//                 let inbound = self.tcp.unwrap().inbound_stream_mut();
//                 let amount_to_write = min(65536, inbound.buffer_size());
//                 let buffer = inbound.peek_output(amount_to_write);
//                 let bytes_written = self.thread_data.write(&buffer, false);
//                 inbound.pop_output(bytes_written);
//
//                 if inbound.eof() || inbound.error() {
//                     self.thread_data.shutdown(SHUT_WR);
//                     self.inbound_shutdown = true;
//
//                     eprintln!("DEBUG: Inbound stream from {} finished {}", self.datagram_adapter.config().destination.to_string(), if inbound.error() {"with an error/reset."} else {"cleanly."});
//                     if self.tcp.unwrap().state() == TCPState::from(State::TimeWait) {
//                         eprintln!("DEBUG: Waiting for lingering segments (e.g. retransmissions of FIN) from peer...");
//                     }
//                 }
//             }),
//             Box::new(|| {
//                 let b1 = !self.tcp.unwrap().inbound_stream().buffer_empty();
//                 let b2 = self.tcp.unwrap().inbound_stream().eof() || self.tcp.unwrap().inbound_stream().error();
//                 let b3 = !self.inbound_shutdown;
//
//                 b1 || (b2 && b3)
//             }),
//             Box::new(|| {}),
//         );
//
//         // rule 4: read outbound segments from TCPConnection and send as datagrams
//         self.eventloop.add_rule(
//             datagram_adapter_rc.clone(),
//             Direction::Out,
//             Box::new(|| {
//                 while !self.tcp.unwrap().segments_out_mut().is_empty() {
//                     <AdapterT as AsFdAdapterBaseMut>::write(&mut self.datagram_adapter, &mut *self.tcp.unwrap().segments_out_mut().pop_front().unwrap());
//                 }
//             }),
//             Box::new(|| !self.tcp.unwrap().segments_out_mut().is_empty()),
//             Box::new(|| {}),
//         );
//     }
//
//     #[allow(dead_code)]
//     fn tcp_loop(&mut self, condition: InterestT<'_>) {
//         let mut base_time = timestamp_ms();
//
//         while condition() {
//             let ret = self
//                 .eventloop
//                 .wait_next_event(TCPSpongeSocket::<AdapterT>::TCP_TICK_MS as i32);
//             if ret == Exit || self.abort {
//                 break;
//             }
//
//             if self.tcp.unwrap().active() {
//                 let next_time = timestamp_ms();
//                 self.tcp.unwrap().tick((next_time - base_time) as SizeT);
//                 self.datagram_adapter.tick((next_time - base_time) as SizeT);
//                 base_time = next_time;
//             }
//         }
//     }
//
//     #[allow(dead_code)]
//     fn tcp_main(&mut self) {
//         assert!(self.tcp.is_some(), "no TCP");
//         self.tcp_loop(Box::new(|| {
//             return true;
//         }));
//         self.shutdown(SHUT_RDWR);
//         if !self.tcp.unwrap().active() {
//             eprintln!(
//                 "DEBUG: TCP connection finished {}",
//                 if self.tcp.unwrap().state() == TCPState::from(State::RESET) {
//                     "uncleanly"
//                 } else {
//                     "cleanly."
//                 }
//             );
//         }
//         self.tcp = None;
//     }
//
//     #[allow(dead_code)]
//     pub fn wait_until_closed(&mut self) {
//         self.shutdown(SHUT_RDWR);
//         eprintln!("DEBUG: Waiting for clean shutdown... ");
//         self.tcp_thread
//             .unwrap()
//             .join()
//             .expect("TCPSpongeSocket thread joined");
//         eprintln!("done.");
//     }
//
//     #[allow(dead_code)]
//     pub fn connect(&mut self, c_tcp: &TCPConfig, c_ad: FdAdapterConfig) {
//         assert!(
//             self.tcp.is_none(),
//             "connect() with TCPConnection already initialized"
//         );
//
//         self.initialize_tcp(c_tcp);
//
//         self.datagram_adapter.set_config(c_ad);
//
//         eprintln!("DEBUG: Connecting to {}...", c_ad.destination.to_string());
//         self.tcp.unwrap().connect();
//
//         let expected_state = TCPState::from(State::SynSent);
//         assert_eq!(
//             self.tcp.unwrap().state(),
//             expected_state,
//             "{}",
//             format!(
//                 "After TCPConnection::connect(), state was {} but expected {}",
//                 self.tcp.unwrap().state().name(),
//                 expected_state.name()
//             )
//         );
//
//         self.tcp_loop(Box::new(|| {
//             self.tcp.unwrap().state() == TCPState::from(State::SynSent)
//         }));
//         eprintln!(
//             "Successfully connected to {}.",
//             c_ad.destination.to_string()
//         );
//
//         let _ = self.tcp_thread.insert(
//             thread::Builder::new()
//                 .name("thread1".to_string())
//                 .spawn(move || self.tcp_main())
//                 .unwrap(),
//         );
//     }
//
//     #[allow(dead_code)]
//     pub fn listen_and_accept(&mut self, c_tcp: &TCPConfig, c_ad: FdAdapterConfig) {
//         assert!(
//             self.tcp.is_none(),
//             "listen_and_accept() with TCPConnection already initialized"
//         );
//
//         self.initialize_tcp(c_tcp);
//
//         self.datagram_adapter.set_config(c_ad);
//         self.datagram_adapter.set_listening(true);
//
//         eprintln!("DEBUG: Listening for incoming connection...");
//         self.tcp_loop(Box::new(|| {
//             let s = self.tcp.unwrap().state();
//             s == TCPState::from(State::LISTEN)
//                 || s == TCPState::from(State::SynRcvd)
//                 || s == TCPState::from(State::SynSent)
//         }));
//         eprintln!(
//             "New connection from {}.",
//             self.datagram_adapter.config().destination.to_string()
//         );
//
//         let _ = self.tcp_thread.insert(
//             thread::Builder::new()
//                 .name("thread1".to_string())
//                 .spawn(move || self.tcp_main())
//                 .unwrap(),
//         );
//     }
// }
