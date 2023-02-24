// use std::cmp::min;
// use crate::tcp_connection::TCPConnection;
// use crate::tcp_helpers::fd_adapter::AsFdAdapterBaseMut;
// use crate::tcp_helpers::tcp_config::{FdAdapterConfig, TCPConfig};
// use crate::tcp_helpers::tcp_state::{State, TCPState};
// use crate::util::aeventloop::AEventLoop;
// use crate::util::eventloop::Result::Exit;
// use crate::util::eventloop::{Direction, InterestT};
// use crate::util::file_descriptor::{AsFileDescriptor, AsFileDescriptorMut, FileDescriptor};
// use crate::util::socket::{AsSocketMut, LocalStreamSocket, Socket};
// use crate::util::util::{system_call, timestamp_ms};
// use crate::SizeT;
// use libc::{SHUT_RDWR, SHUT_WR};
// use std::fmt::Debug;
// use std::ops::DerefMut;
// use std::sync::{Arc, Mutex};
// use std::sync::atomic::{AtomicBool, Ordering};
// use std::thread;
// use std::thread::JoinHandle;
//
// // Mutate from multiple threads without interior mutability?
// //      let file = Arc::new(Mutex::new(File::create("foo.txt").unwrap()));
// // https://users.rust-lang.org/t/mutate-from-multiple-threads-without-interior-mutability/68896
//
// #[derive(Debug)]
// pub struct TCPSpongeSocket<'a, AdapterT> {
//     main_thread_data: Arc<Mutex<LocalStreamSocket>>,
//     thread_data: Arc<Mutex<LocalStreamSocket>>,
//     datagram_adapter: Arc<Mutex<AdapterT>>,
//     tcp: Arc<Mutex<Option<TCPConnection>>>,
//     event_loop: Arc<Mutex<Option<AEventLoop<'a>>>>,
//     tcp_thread: Option<JoinHandle<()>>,
//     abort: Arc<AtomicBool>,
//     inbound_shutdown: Arc<AtomicBool>,
//     outbound_shutdown: Arc<AtomicBool>,
//     fully_acked: Arc<AtomicBool>
// }
//
// impl<AdapterT> AsFileDescriptor for TCPSpongeSocket<'_, AdapterT> {
//     fn as_file_descriptor(&self) -> &FileDescriptor {
//         self.main_thread_data.lock().unwrap().as_file_descriptor()
//     }
// }
// impl<AdapterT> AsFileDescriptorMut for TCPSpongeSocket<'_, AdapterT> {
//     fn as_file_descriptor_mut(&mut self) -> &mut FileDescriptor {
//         self.main_thread_data.lock().unwrap().as_file_descriptor_mut()
//     }
// }
// impl<AdapterT> AsSocketMut for TCPSpongeSocket<'_, AdapterT> {
//     fn as_socket_mut(&mut self) -> &mut Socket {
//         self.main_thread_data.lock().unwrap().as_socket_mut()
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
//         if self.abort.load(Ordering::SeqCst) == false {
//             eprintln!("Warning: unclean shutdown of TCPSpongeSocket");
//             self.abort.store(true, Ordering::SeqCst);
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
//             main_thread_data: Arc::new(Mutex::new(LocalStreamSocket::new(pair.0))),
//             thread_data: Arc::new(Mutex::new(LocalStreamSocket::new(pair.1))),
//             datagram_adapter: Arc::new(Mutex::new(_adapter)),
//             tcp: Arc::new(Mutex::new(None)),
//             event_loop: Arc::new(Mutex::new(Some(AEventLoop::new()))),
//             tcp_thread: None,
//             abort: Arc::new(AtomicBool::new(false)),
//             inbound_shutdown: Arc::new(AtomicBool::new(false)),
//             outbound_shutdown: Arc::new(AtomicBool::new(false)),
//             fully_acked: Arc::new(AtomicBool::new(false))
//         };
//         t.thread_data.lock().unwrap().set_blocking(false);
//
//         t
//     }
//
//     #[allow(dead_code)]
//     fn initialize_tcp(&mut self, config: &TCPConfig) {
//         let _ = self.tcp.lock().unwrap().insert(TCPConnection::new(config.clone()));
//
//         let event_loop_ = self.event_loop.lock().unwrap();
//
//         let datagram_adapter_rc = Arc::new(Mutex::new(self.datagram_adapter.lock().unwrap().as_file_descriptor().clone()));
//         let thread_data_rc = Arc::new(Mutex::new(self.thread_data.lock().unwrap().as_file_descriptor().clone()));
//
//         // rule 1: read from filtered packet stream and dump into TCPConnection
//         event_loop_.unwrap().add_rule(
//             datagram_adapter_rc.clone(),
//             Direction::In,
//             Box::new(|| {
//                 let mut adapter_ = self.datagram_adapter.clone().lock().unwrap();
//                 let tcp_ = self.tcp.clone().lock().unwrap();
//                 let thread_data_ = self.thread_data.clone().lock().unwrap();
//
//                 let seg = <AdapterT as AsFdAdapterBaseMut>::read(&mut adapter_);
//                 if seg.is_some() {
//                     tcp_.unwrap().segment_received(&seg.unwrap());
//                 }
//
//                 if thread_data_.eof()
//                     && tcp_.unwrap().bytes_in_flight() == 0
//                     && !self.fully_acked.load(Ordering::SeqCst)
//                 {
//                     eprintln!(
//                         "DEBUG: Outbound stream to {} has been fully acknowledged.",
//                         adapter_.config().destination.to_string()
//                     );
//                     self.fully_acked.store(true, Ordering::SeqCst);
//                 }
//             }),
//             Box::new(|| {
//                 let tcp_ = self.tcp.clone().lock().unwrap();
//                 tcp_.unwrap().active()
//             }),
//             Box::new(|| {}),
//         );
//
//         // rule 2: read from pipe into outbound buffer
//         event_loop_.unwrap().add_rule(
//             thread_data_rc.clone(),
//             Direction::In,
//             Box::new(|| {
//                 let mut adapter_ = self.datagram_adapter.clone().lock().unwrap();
//                 let tcp_ = self.tcp.clone().lock().unwrap();
//                 let mut thread_data_ = self.thread_data.clone().lock().unwrap();
//
//                 let data = thread_data_.read(tcp_.unwrap().remaining_outbound_capacity() as u32);
//                 let len = data.len();
//                 let amount_written = tcp_.unwrap().write(&data);
//                 assert_eq!(
//                     amount_written, len,
//                     "TCPConnection::write() accepted less than advertised length"
//                 );
//
//                 if thread_data_.eof() {
//                     tcp_.unwrap().end_input_stream();
//                     self.outbound_shutdown.store(true, Ordering::SeqCst);
//
//                     eprintln!(
//                         "DEBUG: Outbound stream to {} finished ({} byte{} still in flight).",
//                         adapter_.config().destination.to_string(),
//                         tcp_.unwrap().bytes_in_flight(),
//                         if 1 == tcp_.unwrap().bytes_in_flight() {
//                             ""
//                         } else {
//                             "s"
//                         }
//                     );
//                 }
//             }),
//             Box::new(|| {
//                 let tcp_ = self.tcp.clone().lock().unwrap();
//
//                 tcp_.unwrap().active()
//                     && !self.outbound_shutdown.load(Ordering::SeqCst)
//                     && (tcp_.unwrap().remaining_outbound_capacity() > 0)
//             }),
//             Box::new(|| {
//                 let tcp_ = self.tcp.clone().lock().unwrap();
//
//                 tcp_.unwrap().end_input_stream();
//                 self.outbound_shutdown.store(true, Ordering::SeqCst)
//             }),
//         );
//
//         // rule 3: read from inbound buffer into pipe
//         event_loop_.unwrap().add_rule(
//             thread_data_rc.clone(),
//             Direction::Out,
//             Box::new(|| {
//                 let mut adapter_ = self.datagram_adapter.clone().lock().unwrap();
//                 let tcp_ = self.tcp.clone().lock().unwrap();
//                 let mut thread_data_ = self.thread_data.clone().lock().unwrap();
//
//                 let inbound = tcp_.unwrap().inbound_stream_mut();
//                 let amount_to_write = min(65536, inbound.buffer_size());
//                 let buffer = inbound.peek_output(amount_to_write);
//                 let bytes_written = thread_data_.write(&buffer, false);
//                 inbound.pop_output(bytes_written);
//
//                 if inbound.eof() || inbound.error() {
//                     thread_data_.shutdown(SHUT_WR);
//                     self.inbound_shutdown.store(true, Ordering::SeqCst);
//
//                     eprintln!("DEBUG: Inbound stream from {} finished {}", adapter_.config().destination.to_string(), if inbound.error() {"with an error/reset."} else {"cleanly."});
//                     if tcp_.unwrap().state() == TCPState::from(State::TimeWait) {
//                         eprintln!("DEBUG: Waiting for lingering segments (e.g. retransmissions of FIN) from peer...");
//                     }
//                 }
//             }),
//             Box::new(|| {
//                 let tcp_ = self.tcp.clone().lock().unwrap();
//
//                 let b1 = !tcp_.unwrap().inbound_stream().buffer_empty();
//                 let b2 = tcp_.unwrap().inbound_stream().eof() || tcp_.unwrap().inbound_stream().error();
//                 let b3 = !self.inbound_shutdown.load(Ordering::SeqCst);
//
//                 b1 || (b2 && b3)
//             }),
//             Box::new(|| {}),
//         );
//
//         // rule 4: read outbound segments from TCPConnection and send as datagrams
//         event_loop_.unwrap().add_rule(
//             datagram_adapter_rc.clone(),
//             Direction::Out,
//             Box::new(|| {
//                 let mut adapter_ = self.datagram_adapter.clone().lock().unwrap();
//                 let tcp_ = self.tcp.clone().lock().unwrap();
//
//                 while !tcp_.unwrap().segments_out_mut().is_empty() {
//                     let t_ = tcp_.unwrap().segments_out_mut().pop_front().unwrap();
//                     let mut t_seg = t_.lock().unwrap();
//                     <AdapterT as AsFdAdapterBaseMut>::write(
//                         &mut adapter_,
//                         t_seg.deref_mut()
//                     );
//                 }
//             }),
//             Box::new(|| {
//                 let tcp_ = self.tcp.clone().lock().unwrap();
//                 !tcp_.unwrap().segments_out_mut().is_empty()
//             }),
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
//                 .event_loop.lock().unwrap().unwrap()
//                 .wait_next_event(TCPSpongeSocket::<AdapterT>::TCP_TICK_MS as i32);
//             if ret == Exit || self.abort.load(Ordering::SeqCst) {
//                 break;
//             }
//
//             let tcp_ = self.tcp.lock().unwrap();
//             if tcp_.unwrap().active() {
//                 let next_time = timestamp_ms();
//                 tcp_.unwrap().tick((next_time - base_time) as SizeT);
//                 self.datagram_adapter.lock().unwrap().tick((next_time - base_time) as SizeT);
//                 base_time = next_time;
//             }
//         }
//     }
//
//     #[allow(dead_code)]
//     fn tcp_main(&mut self) {
//         assert!(self.tcp.lock().unwrap().is_some(), "no TCP");
//         self.tcp_loop(Box::new(|| {
//             return true;
//         }));
//         self.shutdown(SHUT_RDWR);
//
//         let mut tcp_ = self.tcp.lock().unwrap();
//         if !tcp_.unwrap().active() {
//             eprintln!(
//                 "DEBUG: TCP connection finished {}",
//                 if tcp_.unwrap().state() == TCPState::from(State::RESET) {
//                     "uncleanly"
//                 } else {
//                     "cleanly."
//                 }
//             );
//         }
//         drop(tcp_);
//         self.tcp = Arc::new(Mutex::new(None));
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
//             self.tcp.lock().unwrap().is_none(),
//             "connect() with TCPConnection already initialized"
//         );
//
//         self.initialize_tcp(c_tcp);
//
//         self.datagram_adapter.lock().unwrap().set_config(c_ad);
//
//         eprintln!("DEBUG: Connecting to {}...", c_ad.destination.to_string());
//         self.tcp.lock().unwrap().unwrap().connect();
//
//         let expected_state = TCPState::from(State::SynSent);
//         assert_eq!(
//             self.tcp.lock().unwrap().unwrap().state(),
//             expected_state,
//             "{}",
//             format!(
//                 "After TCPConnection::connect(), state was {} but expected {}",
//                 self.tcp.lock().unwrap().unwrap().state().name(),
//                 expected_state.name()
//             )
//         );
//
//         self.tcp_loop(Box::new(|| {
//             self.tcp.clone().lock().unwrap().unwrap().state() == TCPState::from(State::SynSent)
//         }));
//         eprintln!(
//             "Successfully connected to {}.",
//             c_ad.destination.to_string()
//         );
//
//         let _ = self.tcp_thread.insert(
//             thread::Builder::new()
//                 .name("thread1".to_string())
//                 .spawn(|| {
//                     assert!(self.tcp.clone().lock().unwrap().is_some(), "no TCP");
//                     self.tcp_loop(Box::new(|| {
//                         return true;
//                     }));
//                     self.shutdown(SHUT_RDWR);
//
//                     let mut tcp_ = self.tcp.clone().lock().unwrap();
//                     if !tcp_.unwrap().active() {
//                         eprintln!(
//                             "DEBUG: TCP connection finished {}",
//                             if tcp_.unwrap().state() == TCPState::from(State::RESET) {
//                                 "uncleanly"
//                             } else {
//                                 "cleanly."
//                             }
//                         );
//                     }
//                     tcp_.take();
//                 })
//                 .unwrap(),
//         );
//     }
//
//     #[allow(dead_code)]
//     pub fn listen_and_accept(&mut self, c_tcp: &TCPConfig, c_ad: FdAdapterConfig) {
//         assert!(
//             self.tcp.lock().unwrap().is_none(),
//             "listen_and_accept() with TCPConnection already initialized"
//         );
//
//         self.initialize_tcp(c_tcp);
//
//         self.datagram_adapter.lock().unwrap().set_config(c_ad);
//         self.datagram_adapter.lock().unwrap().set_listening(true);
//
//         eprintln!("DEBUG: Listening for incoming connection...");
//         self.tcp_loop(Box::new(|| {
//             let s = self.tcp.clone().lock().unwrap().unwrap().state();
//             s == TCPState::from(State::LISTEN)
//                 || s == TCPState::from(State::SynRcvd)
//                 || s == TCPState::from(State::SynSent)
//         }));
//         eprintln!(
//             "New connection from {}.",
//             self.datagram_adapter.lock().unwrap().config().destination.to_string()
//         );
//
//         let _ = self.tcp_thread.insert(
//             thread::Builder::new()
//                 .name("thread1".to_string())
//                 .spawn(|| {
//                     assert!(self.tcp.clone().lock().unwrap().is_some(), "no TCP");
//                     self.tcp_loop(Box::new(|| {
//                         return true;
//                     }));
//                     self.shutdown(SHUT_RDWR);
//
//                     let mut tcp_ = self.tcp.clone().lock().unwrap();
//                     if !tcp_.unwrap().active() {
//                         eprintln!(
//                             "DEBUG: TCP connection finished {}",
//                             if tcp_.unwrap().state() == TCPState::from(State::RESET) {
//                                 "uncleanly"
//                             } else {
//                                 "cleanly."
//                             }
//                         );
//                     }
//                     tcp_.take();
//                     // self.tcp = Arc::new(Mutex::new(None));
//                 })
//                 .unwrap(),
//         );
//     }
// }
