use crate::tcp_connection::TCPConnection;
use crate::tcp_helpers::fd_adapter::AsFdAdapterBaseMut;
use crate::tcp_helpers::tcp_config::{FdAdapterConfig, TCPConfig};
use crate::tcp_helpers::tcp_state::{State, TCPState};
use crate::util::aeventloop::{AEventLoop, AInterestT};
use crate::util::eventloop::Direction;
use crate::util::eventloop::Result::Exit;
use crate::util::file_descriptor::{AsFileDescriptor, AsFileDescriptorMut, FileDescriptor};
use crate::util::socket::{AsSocketMut, LocalStreamSocket};
use crate::util::util::{system_call, timestamp_ms};
use crate::SizeT;
use libc::{SHUT_RDWR, SHUT_WR};
use std::cmp::min;
use std::fmt::Debug;
use std::ops::DerefMut;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

// Mutate from multiple threads without interior mutability?
//      let file = Arc::new(Mutex::new(File::create("foo.txt").unwrap()));
// https://users.rust-lang.org/t/mutate-from-multiple-threads-without-interior-mutability/68896

#[derive(Debug)]
pub struct TCPSpongeSocket<AdapterT> {
    main_thread_data: Arc<Mutex<LocalStreamSocket>>,
    thread_data: Arc<Mutex<LocalStreamSocket>>,
    datagram_adapter: Arc<Mutex<AdapterT>>,
    tcp: Arc<Mutex<Option<TCPConnection>>>,
    event_loop: Arc<Mutex<AEventLoop>>,
    tcp_thread: Option<JoinHandle<()>>,
    abort: Arc<AtomicBool>,
    inbound_shutdown: Arc<AtomicBool>,
    outbound_shutdown: Arc<AtomicBool>,
    fully_acked: Arc<AtomicBool>,
}
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
impl<AdapterT> Drop for TCPSpongeSocket<AdapterT> {
    fn drop(&mut self) {
        if self.abort.load(Ordering::SeqCst) == false {
            eprintln!("Warning: unclean shutdown of TCPSpongeSocket");
            self.abort.store(true, Ordering::SeqCst);

            let j = self.tcp_thread.take();
            j.unwrap().join().expect("TCPSpongeSocket join during Drop");
        }
    }
}
impl<AdapterT> TCPSpongeSocket<AdapterT>
where
    AdapterT: AsFdAdapterBaseMut + AsFileDescriptorMut + Send + 'static,
{
    pub const TCP_TICK_MS: SizeT = 10;

    #[allow(dead_code)]
    pub fn new(_adapter: AdapterT) -> TCPSpongeSocket<AdapterT> {
        // socketpair: https://stackoverflow.com/questions/11461106/socketpair-in-c-unix
        let mut socks = [0; 2];
        let ret =
            unsafe { libc::socketpair(libc::AF_UNIX, libc::SOCK_STREAM, 0, socks.as_mut_ptr()) };
        system_call("socketpair", ret as i32, 0);

        TCPSpongeSocket::new2(
            (FileDescriptor::new(socks[0]), FileDescriptor::new(socks[1])),
            _adapter,
        )
    }

    #[allow(dead_code)]
    fn new2(
        pair: (FileDescriptor, FileDescriptor),
        _adapter: AdapterT,
    ) -> TCPSpongeSocket<AdapterT> {
        let t = TCPSpongeSocket {
            main_thread_data: Arc::new(Mutex::new(LocalStreamSocket::new(pair.0))),
            thread_data: Arc::new(Mutex::new(LocalStreamSocket::new(pair.1))),
            datagram_adapter: Arc::new(Mutex::new(_adapter)),
            tcp: Arc::new(Mutex::new(None)),
            event_loop: Arc::new(Mutex::new(AEventLoop::new())),
            tcp_thread: None,
            abort: Arc::new(AtomicBool::new(false)),
            inbound_shutdown: Arc::new(AtomicBool::new(false)),
            outbound_shutdown: Arc::new(AtomicBool::new(false)),
            fully_acked: Arc::new(AtomicBool::new(false)),
        };
        t.thread_data.lock().unwrap().set_blocking(false);

        t
    }

    #[allow(dead_code)]
    fn initialize_tcp(&mut self, config: &TCPConfig) {
        let _ = self
            .tcp
            .lock()
            .unwrap()
            .insert(TCPConnection::new(config.clone()));

        let mut event_loop_ = self.event_loop.lock().unwrap();

        let datagram_adapter_rc = Arc::new(Mutex::new(
            self.datagram_adapter
                .lock()
                .unwrap()
                .as_file_descriptor()
                .clone(),
        ));
        let thread_data_rc = Arc::new(Mutex::new(
            self.thread_data
                .lock()
                .unwrap()
                .as_file_descriptor()
                .clone(),
        ));

        // rule 1: read from filtered packet stream and dump into TCPConnection
        let adapter_ = self.datagram_adapter.clone();
        let tcp_ = self.tcp.clone();
        let tcp_1 = self.tcp.clone();
        let thread_data_ = self.thread_data.clone();
        let fully_acked_ = self.fully_acked.clone();
        event_loop_.add_rule(
            datagram_adapter_rc.clone(),
            Direction::In,
            Box::new(move || {
                let mut l = tcp_.lock().unwrap();
                let seg = adapter_.lock().unwrap().read_adp();
                if seg.is_some() {
                    l.as_mut().unwrap().segment_received(&seg.unwrap());
                }

                if thread_data_.lock().unwrap().eof()
                    && l.as_ref().unwrap().bytes_in_flight() == 0
                    && !fully_acked_.load(Ordering::SeqCst)
                {
                    eprintln!(
                        "DEBUG: Outbound stream to {} has been fully acknowledged.",
                        adapter_.lock().unwrap().config().destination.to_string()
                    );
                    fully_acked_.store(true, Ordering::SeqCst);
                }
            }),
            Box::new(move || {
                let l = tcp_1.lock().unwrap();
                l.as_ref().unwrap().active()
            }),
            Box::new(|| {}),
        );

        // rule 2: read from pipe into outbound buffer
        let adapter_ = self.datagram_adapter.clone();
        let tcp_ = self.tcp.clone();
        let tcp_1 = self.tcp.clone();
        let tcp_2 = self.tcp.clone();
        let thread_data_ = self.thread_data.clone();
        let outbound_shutdown_ = self.outbound_shutdown.clone();
        let outbound_shutdown_1 = self.outbound_shutdown.clone();
        let outbound_shutdown_2 = self.outbound_shutdown.clone();
        event_loop_.add_rule(
            thread_data_rc.clone(),
            Direction::In,
            Box::new(move || {
                let mut l = tcp_.lock().unwrap();

                let data = thread_data_
                    .lock()
                    .as_mut()
                    .unwrap()
                    .read(l.as_ref().unwrap().remaining_outbound_capacity() as u32);
                let len = data.len();
                let amount_written = l.as_mut().unwrap().write(&data);
                assert_eq!(
                    amount_written, len,
                    "TCPConnection::write() accepted less than advertised length"
                );

                if thread_data_.lock().unwrap().eof() {
                    l.as_mut().unwrap().end_input_stream();
                    outbound_shutdown_.store(true, Ordering::SeqCst);

                    eprintln!(
                        "DEBUG: Outbound stream to {} finished ({} byte{} still in flight).",
                        adapter_.lock().unwrap().config().destination.to_string(),
                        l.as_ref().unwrap().bytes_in_flight(),
                        if 1 == l.as_ref().unwrap().bytes_in_flight() {
                            ""
                        } else {
                            "s"
                        }
                    );
                }
            }),
            Box::new(move || {
                let l = tcp_1.lock().unwrap();

                l.as_ref().unwrap().active()
                    && !outbound_shutdown_1.load(Ordering::SeqCst)
                    && (l.as_ref().unwrap().remaining_outbound_capacity() > 0)
            }),
            Box::new(move || {
                let mut l = tcp_2.lock().unwrap();

                l.as_mut().unwrap().end_input_stream();
                outbound_shutdown_2.store(true, Ordering::SeqCst)
            }),
        );

        // rule 3: read from inbound buffer into pipe
        let adapter_ = self.datagram_adapter.clone();
        let tcp_ = self.tcp.clone();
        let tcp_1 = self.tcp.clone();
        let thread_data_ = self.thread_data.clone();
        let inbound_shutdown_ = self.inbound_shutdown.clone();
        let inbound_shutdown_1 = self.inbound_shutdown.clone();
        event_loop_.add_rule(
            thread_data_rc.clone(),
            Direction::Out,
            Box::new(move || {
                let mut l = tcp_.lock().unwrap();

                let inbound = l.as_mut().unwrap().inbound_stream_mut();
                let amount_to_write = min(65536, inbound.buffer_size());
                let buffer = inbound.peek_output(amount_to_write);
                let bytes_written = thread_data_.lock().unwrap().write(&buffer, false);
                inbound.pop_output(bytes_written);

                if inbound.eof() || inbound.error() {
                    thread_data_.lock().unwrap().shutdown(SHUT_WR);
                    inbound_shutdown_.store(true, Ordering::SeqCst);

                    eprintln!("DEBUG: Inbound stream from {} finished {}", adapter_.lock().unwrap().config().destination.to_string(), if inbound.error() {"with an error/reset."} else {"cleanly."});
                    if l.as_ref().unwrap().state() == TCPState::from(State::TimeWait) {
                        eprintln!("DEBUG: Waiting for lingering segments (e.g. retransmissions of FIN) from peer...");
                    }
                }
            }),
            Box::new(move || {
                let l = tcp_1.lock().unwrap();

                let b1 = !l.as_ref().unwrap().inbound_stream().buffer_empty();
                let b2 = l.as_ref().unwrap().inbound_stream().eof() || l.as_ref().unwrap().inbound_stream().error();
                let b3 = !inbound_shutdown_1.load(Ordering::SeqCst);

                b1 || (b2 && b3)
            }),
            Box::new(|| {}),
        );

        // rule 4: read outbound segments from TCPConnection and send as datagrams
        let adapter_ = self.datagram_adapter.clone();
        let tcp_ = self.tcp.clone();
        let tcp_1 = self.tcp.clone();
        event_loop_.add_rule(
            datagram_adapter_rc.clone(),
            Direction::Out,
            Box::new(move || {
                let mut l = tcp_.lock().unwrap();

                while !l.as_mut().unwrap().segments_out_mut().is_empty() {
                    let t_ = l.as_mut().unwrap().segments_out_mut().pop_front().unwrap();
                    let mut t_seg = t_.lock().unwrap();
                    adapter_.lock().unwrap().write_adp(t_seg.deref_mut());
                }
            }),
            Box::new(move || {
                let mut l = tcp_1.lock().unwrap();
                !l.as_mut().unwrap().segments_out_mut().is_empty()
            }),
            Box::new(|| {}),
        );
    }

    #[allow(dead_code)]
    pub fn wait_until_closed(&mut self) {
        self.main_thread_data.lock().unwrap().shutdown(SHUT_RDWR);
        eprintln!("DEBUG: Waiting for clean shutdown... ");

        let j = self.tcp_thread.take();
        j.unwrap().join().expect("TCPSpongeSocket thread joined");
        eprintln!("done.");
    }

    #[allow(dead_code)]
    pub fn connect(&mut self, c_tcp: &TCPConfig, c_ad: FdAdapterConfig) {
        assert!(
            self.tcp.lock().unwrap().is_none(),
            "connect() with TCPConnection already initialized"
        );

        self.initialize_tcp(c_tcp);

        self.datagram_adapter.lock().unwrap().set_config(c_ad);

        eprintln!("DEBUG: Connecting to {}...", c_ad.destination.to_string());
        self.tcp.lock().unwrap().as_mut().unwrap().connect();

        let expected_state = TCPState::from(State::SynSent);
        assert_eq!(
            self.tcp.lock().unwrap().as_ref().unwrap().state(),
            expected_state,
            "{}",
            format!(
                "After TCPConnection::connect(), state was {} but expected {}",
                self.tcp.lock().unwrap().as_ref().unwrap().state().name(),
                expected_state.name()
            )
        );

        let tcp_ = self.tcp.clone();
        let tcp_1 = self.tcp.clone();
        let event_loop_ = self.event_loop.clone();
        let abort_ = self.abort.clone();
        let adapter_ = self.datagram_adapter.clone();
        tcp_loop(
            Box::new(move || {
                tcp_.lock().unwrap().as_ref().unwrap().state() == TCPState::from(State::SynSent)
            }),
            event_loop_,
            abort_,
            tcp_1,
            adapter_,
        );
        eprintln!(
            "Successfully connected to {}.",
            c_ad.destination.to_string()
        );

        let tcp_ = self.tcp.clone();
        let main_thread_data_ = self.main_thread_data.clone();
        let event_loop_ = self.event_loop.clone();
        let abort_ = self.abort.clone();
        let datagram_adapter_ = self.datagram_adapter.clone();
        let _ = self.tcp_thread.insert(
            thread::Builder::new()
                .name("thread1".to_string())
                .spawn(Box::new(move || {
                    tcp_main(
                        tcp_,
                        main_thread_data_,
                        event_loop_,
                        abort_,
                        datagram_adapter_,
                    )
                }))
                .unwrap(),
        );
    }

    #[allow(dead_code)]
    pub fn listen_and_accept(&mut self, c_tcp: &TCPConfig, c_ad: FdAdapterConfig) {
        assert!(
            self.tcp.lock().unwrap().is_none(),
            "listen_and_accept() with TCPConnection already initialized"
        );

        self.initialize_tcp(c_tcp);

        self.datagram_adapter.lock().unwrap().set_config(c_ad);
        self.datagram_adapter.lock().unwrap().set_listening(true);

        eprintln!("DEBUG: Listening for incoming connection...");
        let tcp_ = self.tcp.clone();
        let tcp_1 = self.tcp.clone();
        let event_loop_ = self.event_loop.clone();
        let abort_ = self.abort.clone();
        let adapter_ = self.datagram_adapter.clone();
        tcp_loop(
            Box::new(move || {
                let s = tcp_.lock().unwrap().as_ref().unwrap().state();
                s == TCPState::from(State::LISTEN)
                    || s == TCPState::from(State::SynRcvd)
                    || s == TCPState::from(State::SynSent)
            }),
            event_loop_,
            abort_,
            tcp_1,
            adapter_,
        );
        eprintln!(
            "New connection from {}.",
            self.datagram_adapter
                .lock()
                .unwrap()
                .config()
                .destination
                .to_string()
        );

        let tcp_ = self.tcp.clone();
        let main_thread_data_ = self.main_thread_data.clone();
        let event_loop_ = self.event_loop.clone();
        let abort_ = self.abort.clone();
        let datagram_adapter_ = self.datagram_adapter.clone();
        let _ = self.tcp_thread.insert(
            thread::Builder::new()
                .name("thread1".to_string())
                .spawn(move || {
                    tcp_main(
                        tcp_,
                        main_thread_data_,
                        event_loop_,
                        abort_,
                        datagram_adapter_,
                    )
                })
                .unwrap(),
        );
    }
}

#[allow(dead_code)]
fn tcp_loop<AdapterT>(
    condition: AInterestT,
    event_loop: Arc<Mutex<AEventLoop>>,
    abort: Arc<AtomicBool>,
    tcp: Arc<Mutex<Option<TCPConnection>>>,
    adapter: Arc<Mutex<AdapterT>>,
) where
    AdapterT: AsFdAdapterBaseMut + AsFileDescriptorMut + Send + 'static,
{
    let mut base_time = timestamp_ms();

    while condition() {
        let ret = event_loop
            .lock()
            .unwrap()
            .wait_next_event(TCPSpongeSocket::<AdapterT>::TCP_TICK_MS as i32);
        if ret == Exit || abort.load(Ordering::SeqCst) {
            break;
        }

        let mut tcp_ = tcp.lock().unwrap();
        if tcp_.as_ref().unwrap().active() {
            let next_time = timestamp_ms();
            tcp_.as_mut()
                .unwrap()
                .tick((next_time - base_time) as SizeT);
            adapter
                .lock()
                .unwrap()
                .tick((next_time - base_time) as SizeT);
            base_time = next_time;
        }
    }
}

#[allow(dead_code)]
fn tcp_main<AdapterT>(
    tcp: Arc<Mutex<Option<TCPConnection>>>,
    main_thread_data: Arc<Mutex<LocalStreamSocket>>,
    event_loop: Arc<Mutex<AEventLoop>>,
    abort: Arc<AtomicBool>,
    adapter: Arc<Mutex<AdapterT>>,
) where
    AdapterT: AsFdAdapterBaseMut + AsFileDescriptorMut + Send + 'static,
{
    assert!(tcp.lock().unwrap().is_some(), "no TCP");
    tcp_loop(
        Box::new(move || {
            return true;
        }),
        event_loop.clone(),
        abort.clone(),
        tcp.clone(),
        adapter.clone(),
    );
    main_thread_data.lock().unwrap().shutdown(SHUT_RDWR);

    let mut tcp_ = tcp.lock().unwrap();
    if !tcp_.as_ref().unwrap().active() {
        eprintln!(
            "DEBUG: TCP connection finished {}",
            if tcp_.as_ref().unwrap().state() == TCPState::from(State::RESET) {
                "uncleanly"
            } else {
                "cleanly."
            }
        );
    }
    tcp_.take();
}
