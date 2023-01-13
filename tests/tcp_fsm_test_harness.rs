use rust_sponge::tcp_connection::TCPConnection;
use rust_sponge::tcp_helpers::fd_adapter::FdAdapterBase;
use rust_sponge::tcp_helpers::tcp_config::TCPConfig;
use rust_sponge::tcp_helpers::tcp_header::TCPHeader;
use rust_sponge::tcp_helpers::tcp_segment::TCPSegment;
use rust_sponge::tcp_helpers::tcp_state::TCPState;
use rust_sponge::util::buffer::Buffer;
use rust_sponge::util::file_descriptor::FileDescriptor;
use rust_sponge::util::parser::ParseResult;
use rust_sponge::util::util::system_call;
use rust_sponge::wrapping_integers::WrappingInt32;
use rust_sponge::SizeT;
use std::ffi::{c_int, c_void};
use std::ptr::null_mut;

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
}

pub struct TestRFD {
    file_descriptor: FileDescriptor,
}
impl AsFileDescriptor for TestRFD {
    fn as_file_descriptor(&self) -> &FileDescriptor {
        &self.file_descriptor
    }
}
impl AsFileDescriptorMut for TestRFD {
    fn as_file_descriptor_mut(&mut self) -> &mut FileDescriptor {
        &mut self.file_descriptor
    }
}
impl TestRFD {
    pub fn can_read(&self) -> bool {
        let mut tmp = [0u8; 1];

        let ret = unsafe {
            libc::recv(
                self.fd_num(),
                (&mut tmp[..]).as_mut_ptr() as *mut c_void,
                1,
                libc::MSG_PEEK | libc::MSG_DONTWAIT,
            )
        };
        let r = system_call("recv", ret as i32, libc::EAGAIN);
        r >= 0
    }

    pub fn read(&mut self) -> Vec<u8> {
        let mut ret = vec![0u8; TestFD::MAX_RECV];
        let ret_read = unsafe {
            libc::recv(
                self.fd_num(),
                ret.as_ptr() as *mut c_void,
                ret.capacity(),
                libc::MSG_TRUNC,
            )
        };
        let r = system_call("recv", ret_read as i32, 0);
        if r > ret.len() as i32 {
            panic!(
                "{} {} {} {}",
                "TestFD unexpectedly got truncated packet.",
                r,
                ret.len(),
                ret_read
            );
        }

        ret.resize(ret_read as usize, 0);
        self.register_read();

        ret
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Message {
    None,
    Request,
}
pub struct TestFD {
    recv_fd: TestRFD,
    file_descriptor: FileDescriptor,
}
impl TestFD {
    pub const MAX_RECV: SizeT = TCPConfig::MAX_PAYLOAD_SIZE + TCPHeader::LENGTH + 16;

    #[allow(dead_code)]
    pub fn new(fd: FileDescriptor, rfd: TestRFD) -> TestFD {
        TestFD {
            file_descriptor: fd,
            recv_fd: rfd,
        }
    }

    #[allow(dead_code)]
    pub fn new_pair() -> TestFD {
        let mut socks = [0; 2];
        let ret =
            unsafe { libc::socketpair(libc::AF_UNIX, libc::SOCK_SEQPACKET, 0, socks.as_mut_ptr()) };
        system_call("socketpair", ret as i32, 0);

        TestFD {
            file_descriptor: FileDescriptor::new(socks[0]),
            recv_fd: TestRFD {
                file_descriptor: FileDescriptor::new(socks[1]),
            },
        }
    }

    #[allow(dead_code)]
    pub fn write(&self, s: &mut String) {
        let vecs = [libc::iovec {
            iov_base: s.as_mut_ptr() as *mut c_void,
            iov_len: s.len(),
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
        let ret = unsafe { libc::sendmsg(self.file_descriptor.fd_num(), &msg, libc::MSG_EOR) };
        system_call("sendmsg", ret as i32, 0);
    }

    #[allow(dead_code)]
    pub fn write_u8(&self, s: &mut Vec<u8>) {
        let vecs = [libc::iovec {
            iov_base: s.as_mut_ptr() as *mut c_void,
            iov_len: s.len(),
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
        let ret = unsafe { libc::sendmsg(self.file_descriptor.fd_num(), &msg, libc::MSG_EOR) };
        system_call("sendmsg", ret as i32, 0);
    }

    #[allow(dead_code)]
    pub fn can_read(&self) -> bool {
        self.recv_fd.can_read()
    }

    #[allow(dead_code)]
    pub fn read(&mut self) -> Vec<u8> {
        self.recv_fd.read()
    }
}
impl AsFileDescriptor for TestFD {
    fn as_file_descriptor(&self) -> &FileDescriptor {
        &self.file_descriptor
    }
}
impl AsFileDescriptorMut for TestFD {
    fn as_file_descriptor_mut(&mut self) -> &mut FileDescriptor {
        &mut self.file_descriptor
    }
}

pub struct TestFdAdapter {
    test_fd: TestFD,
    base: FdAdapterBase,
}
impl TestFdAdapter {
    pub const MAX_RECV: SizeT = TCPConfig::MAX_PAYLOAD_SIZE + TCPHeader::LENGTH + 16;

    #[allow(dead_code)]
    pub fn new(fd: FileDescriptor, rfd: TestRFD) -> TestFD {
        TestFD {
            file_descriptor: fd,
            recv_fd: rfd,
        }
    }

    #[allow(dead_code)]
    pub fn write(&self, s: &mut String) {
        let vecs = [libc::iovec {
            iov_base: s.as_mut_ptr() as *mut c_void,
            iov_len: s.len(),
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
        let ret = unsafe { libc::sendmsg(self.fd_num(), &msg, libc::MSG_EOR) };
        system_call("sendmsg", ret as i32, 0);
    }

    pub fn write_seg(&mut self, seg: &mut TCPSegment) {
        self.config_segment(seg);

        let mut s = seg.serialize_u8(0);
        self.test_fd.write_u8(&mut s);
    }

    pub fn config_segment(&self, seg: &mut TCPSegment) {
        let cfg = self.base.config();
        let tcp_hdr = seg.header_mut();
        tcp_hdr.sport = cfg.source.port();
        tcp_hdr.dport = cfg.destination.port();
    }

    pub fn can_read(&self) -> bool {
        self.test_fd.can_read()
    }

    pub fn read(&mut self) -> Vec<u8> {
        self.test_fd.read()
    }
}
impl AsFileDescriptor for TestFdAdapter {
    fn as_file_descriptor(&self) -> &FileDescriptor {
        &self.test_fd.file_descriptor
    }
}
impl AsFileDescriptorMut for TestFdAdapter {
    fn as_file_descriptor_mut(&mut self) -> &mut FileDescriptor {
        &mut self.test_fd.file_descriptor
    }
}

pub trait TCPTestStep {
    fn execute(&mut self, h: &mut TCPTestHarness);
}
pub trait TCPExpectation: TCPTestStep {
    fn description(&self) -> String;
    fn to_string(&self) -> String {
        String::from(format!("Expectation: {}", self.description()))
    }
    fn expect_seg(&mut self, _h: &mut TCPTestHarness) -> TCPSegment {
        TCPSegment::new(TCPHeader::new(), Buffer::new(vec![]))
    }
}

pub trait TCPAction: TCPTestStep {
    fn description(&self) -> String;
    fn to_string(&self) -> String {
        String::from(format!("Action: {}", self.description()))
    }
}

pub struct ExpectNoData {}
impl TCPTestStep for ExpectNoData {
    fn execute(&mut self, h: &mut TCPTestHarness) {
        println!("  step: {}", TCPExpectation::to_string(self));

        assert!(
            !h.can_read(),
            "The TCP produced data when it should not have"
        );
    }
}
impl TCPExpectation for ExpectNoData {
    fn description(&self) -> String {
        "no (more) data available".to_string()
    }
}

pub struct ExpectData {
    data: Option<String>,
}
impl TCPTestStep for ExpectData {
    fn execute(&mut self, h: &mut TCPTestHarness) {
        println!("  step: {}", TCPExpectation::to_string(self));

        assert!(
            !h.fsm.inbound_stream().buffer_empty(),
            "The TCP should have data for the user, but does not"
        );

        let bytes_avail: SizeT = h.fsm.inbound_stream().buffer_size();
        let actual_data = h.fsm.inbound_stream_mut().read(bytes_avail);
        if self.data.is_some() {
            if actual_data.len() != self.data.as_ref().unwrap().len() {
                assert!(
                    false,
                    "{}",
                    format!(
                        "The TCP produced {} bytes, but should have produced {} bytes",
                        actual_data.len(),
                        self.data.as_ref().unwrap().len()
                    )
                );
            }
            if actual_data != self.data.as_ref().unwrap().clone() {
                assert!(
                    false,
                    "{}",
                    format!(
                        "The TCP produced data {}, but should have produced {} bytes",
                        actual_data,
                        self.data.as_ref().unwrap()
                    )
                );
            }
        }
    }
}
impl TCPExpectation for ExpectData {
    fn description(&self) -> String {
        if self.data.is_some() {
            format!(
                "data available ({} bytes starting with {})",
                self.data.as_ref().unwrap().len(),
                append_data(&self.data.as_ref().unwrap())
            )
        } else {
            "data available".to_string()
        }
    }
}
impl ExpectData {
    #[allow(dead_code)]
    pub fn new() -> ExpectData {
        ExpectData { data: None }
    }

    #[allow(dead_code)]
    pub fn with_data(&mut self, data_: String) -> &mut ExpectData {
        let _ = self.data.insert(data_);
        self
    }
}

pub struct ExpectSegmentAvailable {}
impl TCPTestStep for ExpectSegmentAvailable {
    fn execute(&mut self, h: &mut TCPTestHarness) {
        println!("  step: {}", TCPExpectation::to_string(self));

        assert!(
            h.can_read(),
            "The TCP should have produces a segment, but did not"
        )
    }
}
impl TCPExpectation for ExpectSegmentAvailable {
    fn description(&self) -> String {
        "segment sent".to_string()
    }
}

pub struct ExpectNoSegment {}
impl TCPTestStep for ExpectNoSegment {
    fn execute(&mut self, h: &mut TCPTestHarness) {
        println!("  step: {}", TCPExpectation::to_string(self));

        assert!(
            !h.can_read(),
            "The TCP produced a segment when it should not have"
        )
    }
}
impl TCPExpectation for ExpectNoSegment {
    fn description(&self) -> String {
        "no (more) segments sent".to_string()
    }
}

pub struct ExpectSegment {
    ack: Option<bool>,
    rst: Option<bool>,
    syn: Option<bool>,
    fin: Option<bool>,
    seqno: Option<WrappingInt32>,
    ackno: Option<WrappingInt32>,
    win: Option<u16>,
    payload_size: Option<SizeT>,
    data: Option<String>,
}
impl TCPTestStep for ExpectSegment {
    fn execute(&mut self, h: &mut TCPTestHarness) {
        println!("  step: {}", TCPExpectation::to_string(self));

        self.expect_seg(h);
    }
}
impl TCPExpectation for ExpectSegment {
    fn description(&self) -> String {
        format!("segment sent with {}", self.segment_description())
    }

    fn expect_seg(&mut self, harness: &mut TCPTestHarness) -> TCPSegment {
        if !harness.can_read() {
            assert!(false, "{}", self.violated_verb("existed"));
        }

        let mut seg = TCPSegment::new(TCPHeader::new(), Buffer::new(vec![]));
        let bytes = harness.flt.read();
        let ret = seg.parse_u8(&bytes, 0);
        if ParseResult::NoError != ret {
            assert!(
                false,
                "{} with result {:?}",
                self.violated_verb("was parsable"),
                ret
            );
        }

        if self.ack.is_some() && seg.header().ack != self.ack.unwrap() {
            assert!(
                false,
                "{}",
                self.violated_field(
                    "ack",
                    self.ack.unwrap().to_string().as_str(),
                    seg.header().ack.to_string().as_str()
                )
            );
        }
        if self.rst.is_some() && seg.header().rst != self.rst.unwrap() {
            assert!(
                false,
                "{}",
                self.violated_field(
                    "rst",
                    self.rst.unwrap().to_string().as_str(),
                    seg.header().rst.to_string().as_str()
                )
            );
        }
        if self.syn.is_some() && seg.header().syn != self.syn.unwrap() {
            assert!(
                false,
                "{}",
                self.violated_field(
                    "syn",
                    self.syn.unwrap().to_string().as_str(),
                    seg.header().syn.to_string().as_str()
                )
            );
        }
        if self.fin.is_some() && seg.header().fin != self.fin.unwrap() {
            assert!(
                false,
                "{}",
                self.violated_field(
                    "fin",
                    self.fin.unwrap().to_string().as_str(),
                    seg.header().fin.to_string().as_str()
                )
            );
        }
        if self.seqno.is_some() && seg.header().seqno != self.seqno.unwrap() {
            assert!(
                false,
                "{}",
                self.violated_field(
                    "seqno",
                    self.seqno.unwrap().to_string().as_str(),
                    seg.header().seqno.to_string().as_str()
                )
            );
        }
        if self.ackno.is_some() && seg.header().ackno != self.ackno.unwrap() {
            assert!(
                false,
                "{}",
                self.violated_field(
                    "ackno",
                    self.ackno.unwrap().to_string().as_str(),
                    seg.header().ackno.to_string().as_str()
                )
            );
        }
        if self.win.is_some() && seg.header().win != self.win.unwrap() {
            assert!(
                false,
                "{}",
                self.violated_field(
                    "win",
                    self.win.unwrap().to_string().as_str(),
                    seg.header().win.to_string().as_str()
                )
            );
        }
        if self.payload_size.is_some() && seg.payload().size() != self.payload_size.unwrap() {
            assert!(
                false,
                "{}",
                self.violated_field(
                    "payload_size",
                    self.payload_size.unwrap().to_string().as_str(),
                    seg.payload().size().to_string().as_str()
                )
            );
        }
        if seg.length_in_sequence_space() > TCPConfig::MAX_PAYLOAD_SIZE {
            assert!(
                false,
                "{}",
                format!(
                    "packet has length_including_flags ({}) greater than the maximum",
                    seg.length_in_sequence_space()
                )
            );
        }
        if self.data.is_some()
            && String::from_utf8(Vec::from(seg.payload().str())).unwrap()
                != self.data.as_ref().unwrap().clone()
        {
            assert!(false, "payloads differ");
        }

        seg
    }
}
impl ExpectSegment {
    #[allow(dead_code)]
    pub fn new() -> ExpectSegment {
        ExpectSegment {
            ack: None,
            rst: None,
            syn: None,
            fin: None,
            seqno: None,
            ackno: None,
            win: None,
            payload_size: None,
            data: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_ack(&mut self, ack_: bool) -> &mut ExpectSegment {
        let _ = self.ack.insert(ack_);
        self
    }

    #[allow(dead_code)]
    pub fn with_rst(&mut self, rst_: bool) -> &mut ExpectSegment {
        let _ = self.rst.insert(rst_);
        self
    }

    #[allow(dead_code)]
    pub fn with_syn(&mut self, syn_: bool) -> &mut ExpectSegment {
        let _ = self.syn.insert(syn_);
        self
    }

    #[allow(dead_code)]
    pub fn with_fin(&mut self, fin_: bool) -> &mut ExpectSegment {
        let _ = self.fin.insert(fin_);
        self
    }

    #[allow(dead_code)]
    pub fn with_no_flags(&mut self) -> &mut ExpectSegment {
        let _ = self.ack.insert(false);
        let _ = self.rst.insert(false);
        let _ = self.syn.insert(false);
        let _ = self.fin.insert(false);
        self
    }

    #[allow(dead_code)]
    pub fn with_seqno(&mut self, seqno_: WrappingInt32) -> &mut ExpectSegment {
        let _ = self.seqno.insert(seqno_);
        self
    }

    #[allow(dead_code)]
    pub fn with_seqno_32(&mut self, seqno_: u32) -> &mut ExpectSegment {
        self.with_seqno(WrappingInt32::new(seqno_))
    }

    #[allow(dead_code)]
    pub fn with_ackno(&mut self, ackno_: WrappingInt32) -> &mut ExpectSegment {
        let _ = self.ackno.insert(ackno_);
        self
    }

    #[allow(dead_code)]
    pub fn with_ackno_32(&mut self, ackno_: u32) -> &mut ExpectSegment {
        let _ = self.ackno.insert(WrappingInt32::new(ackno_));
        self
    }

    #[allow(dead_code)]
    pub fn with_win(&mut self, win_: u16) -> &mut ExpectSegment {
        let _ = self.win.insert(win_);
        self
    }

    #[allow(dead_code)]
    pub fn with_payload_size(&mut self, payload_size_: SizeT) -> &mut ExpectSegment {
        let _ = self.payload_size.insert(payload_size_);
        self
    }

    #[allow(dead_code)]
    pub fn with_data(&mut self, data_: String) -> &mut ExpectSegment {
        let _ = self.data.insert(data_);
        self
    }

    #[allow(dead_code)]
    pub fn segment_description(&self) -> String {
        let mut o = String::new();
        o.push_str("(");
        if self.ack.is_some() {
            o.push_str(if self.ack.unwrap() { "A=1," } else { "A=0," });
        }
        if self.rst.is_some() {
            o.push_str(if self.rst.unwrap() { "R=1," } else { "R=0," });
        }
        if self.syn.is_some() {
            o.push_str(if self.syn.unwrap() { "S=1," } else { "S=0," });
        }
        if self.fin.is_some() {
            o.push_str(if self.fin.unwrap() { "F=1," } else { "F=0," });
        }
        if self.ackno.is_some() {
            o.push_str(format!("ackno={},", self.ackno.unwrap()).as_str());
        }
        if self.win.is_some() {
            o.push_str(format!("win={},", self.win.unwrap()).as_str());
        }
        if self.seqno.is_some() {
            o.push_str(format!("seqno={},", self.seqno.unwrap()).as_str());
        }
        if self.payload_size.is_some() {
            o.push_str(format!("payload_size={},", self.payload_size.unwrap()).as_str());
        }
        if self.data.is_some() {
            o.push_str(format!("data={},", append_data(&self.data.as_ref().unwrap())).as_str());
        }
        o.push_str(")");
        o.to_string()
    }

    pub fn violated_field(
        &self,
        field_name: &str,
        expected_value: &str,
        actual_value: &str,
    ) -> String {
        format!(
            "The TCP produced a segment with `{} = {}`, but {} was expected to be `{}`",
            field_name, actual_value, field_name, expected_value
        )
    }

    pub fn violated_verb(&self, msg: &str) -> String {
        format!(
            "The TCP should have produced a segment that {}, but it did not",
            msg
        )
    }
}

pub struct ExpectOneSegment {
    base: ExpectSegment,
}
impl TCPTestStep for ExpectOneSegment {
    fn execute(&mut self, h: &mut TCPTestHarness) {
        println!("  step: {}", TCPExpectation::to_string(self));

        self.base.execute(h);
        assert!(
            !h.can_read(),
            "The TCP an extra segment when it should not have"
        );
    }
}
impl TCPExpectation for ExpectOneSegment {
    fn description(&self) -> String {
        format!(
            "exactly one segment sent with {}",
            self.base.segment_description()
        )
    }

    fn expect_seg(&mut self, harness: &mut TCPTestHarness) -> TCPSegment {
        let seg = self.base.expect_seg(harness);
        assert!(
            !harness.can_read(),
            "The TCP an extra segment when it should not have"
        );

        seg
    }
}
impl ExpectOneSegment {
    #[allow(dead_code)]
    pub fn new() -> ExpectOneSegment {
        ExpectOneSegment {
            base: ExpectSegment::new(),
        }
    }

    #[allow(dead_code)]
    pub fn base_mut(&mut self) -> &mut ExpectSegment {
        &mut self.base
    }
}

pub struct ExpectState {
    state: TCPState,
}
impl TCPTestStep for ExpectState {
    fn execute(&mut self, h: &mut TCPTestHarness) {
        println!("  step: {}", TCPExpectation::to_string(self));

        let actual_state = h.fsm.state();
        assert_eq!(
            actual_state,
            self.state,
            "{}",
            format!(
                "The TCP was in state `{}`, but it was expected to be in state `{}`",
                actual_state.name(),
                self.state.name()
            )
        );
    }
}
impl TCPExpectation for ExpectState {
    fn description(&self) -> String {
        format!("TCP in state {}", self.state.name())
    }
}
impl ExpectState {
    #[allow(dead_code)]
    pub fn new(state_: TCPState) -> ExpectState {
        ExpectState { state: state_ }
    }
}

pub struct ExpectNotInState {
    state: TCPState,
}
impl TCPTestStep for ExpectNotInState {
    fn execute(&mut self, h: &mut TCPTestHarness) {
        println!("  step: {}", TCPExpectation::to_string(self));

        let actual_state = h.fsm.state();
        assert_ne!(
            actual_state,
            self.state,
            "{}",
            make_not("state", self.state.name())
        );
    }
}
impl TCPExpectation for ExpectNotInState {
    fn description(&self) -> String {
        format!("TCP **not** in state {}", self.state.name())
    }
}
impl ExpectNotInState {
    #[allow(dead_code)]
    pub fn new(state_: TCPState) -> ExpectNotInState {
        ExpectNotInState { state: state_ }
    }
}

pub struct ExpectBytesInFlight {
    bytes: u64,
}
impl TCPTestStep for ExpectBytesInFlight {
    fn execute(&mut self, h: &mut TCPTestHarness) {
        println!("  step: {}", TCPExpectation::to_string(self));

        let actual_bytes = h.fsm.bytes_in_flight();
        assert_eq!(
            actual_bytes,
            self.bytes as SizeT,
            "{}",
            format!(
                "{}",
                make(
                    "bytes_in_flight",
                    self.bytes.to_string(),
                    actual_bytes.to_string()
                )
            )
        );
    }
}
impl TCPExpectation for ExpectBytesInFlight {
    fn description(&self) -> String {
        format!("TCP has {} bytes in flight", self.bytes)
    }
}
impl ExpectBytesInFlight {
    #[allow(dead_code)]
    pub fn new(b: u64) -> ExpectBytesInFlight {
        ExpectBytesInFlight { bytes: b }
    }
}

pub struct ExpectUnassembledBytes {
    bytes: u64,
}
impl TCPTestStep for ExpectUnassembledBytes {
    fn execute(&mut self, h: &mut TCPTestHarness) {
        println!("  step: {}", TCPExpectation::to_string(self));

        let actual_unassembled_bytes = h.fsm.unassembled_bytes();
        assert_eq!(
            actual_unassembled_bytes,
            self.bytes as SizeT,
            "{}",
            format!(
                "{}",
                make(
                    "unassembled_bytes",
                    self.bytes.to_string(),
                    actual_unassembled_bytes.to_string()
                )
            )
        );
    }
}
impl TCPExpectation for ExpectUnassembledBytes {
    fn description(&self) -> String {
        format!("TCP has {} unassembled bytes", self.bytes)
    }
}
impl ExpectUnassembledBytes {
    #[allow(dead_code)]
    pub fn new(b: u64) -> ExpectUnassembledBytes {
        ExpectUnassembledBytes { bytes: b }
    }
}

pub struct ExpectLingerTimer {
    ms: u64,
}
impl TCPTestStep for ExpectLingerTimer {
    fn execute(&mut self, h: &mut TCPTestHarness) {
        println!("  step: {}", TCPExpectation::to_string(self));

        let actual_ms = h.fsm.time_since_last_segment_received();
        assert_eq!(
            actual_ms,
            self.ms as SizeT,
            "{}",
            format!(
                "{}",
                make(
                    "time_since_last_segment_received",
                    self.ms.to_string(),
                    actual_ms.to_string()
                )
            )
        );
    }
}
impl TCPExpectation for ExpectLingerTimer {
    fn description(&self) -> String {
        format!("Most recent incoming segment was {} ms ago", self.ms)
    }
}
impl ExpectLingerTimer {
    #[allow(dead_code)]
    pub fn new(b: u64) -> ExpectLingerTimer {
        ExpectLingerTimer { ms: b }
    }
}

pub struct SendSegment {
    ack: bool,
    rst: bool,
    syn: bool,
    fin: bool,
    seqno: WrappingInt32,
    ackno: WrappingInt32,
    win: u16,
    payload_size: SizeT,
    data: Buffer,
}
impl TCPTestStep for SendSegment {
    fn execute(&mut self, h: &mut TCPTestHarness) {
        println!("  step: {}", TCPAction::to_string(self));

        let seg = self.get_segment();
        h.fsm.segment_received(&seg);
    }
}
impl TCPAction for SendSegment {
    fn description(&self) -> String {
        let seg = self.get_segment();
        let mut o = String::new();
        o.push_str("packet arrives: ");
        o.push_str(seg.header().summary().as_str());
        if seg.payload().size() > 0 {
            o.push_str(format!(" with {} data bytes ", seg.payload().size()).as_str());
            let a = Vec::from(seg.payload().str());
            o.push_str(append_data(&String::from_utf8(a).unwrap()).as_str());
        } else {
            o.push_str(" with no payload");
        }
        o.to_string()
    }
}
impl SendSegment {
    #[allow(dead_code)]
    pub fn new(seg: &TCPSegment) -> SendSegment {
        SendSegment {
            ack: seg.header().ack,
            rst: seg.header().rst,
            syn: seg.header().syn,
            fin: seg.header().fin,
            seqno: seg.header().seqno.clone(),
            ackno: seg.header().ackno.clone(),
            win: seg.header().win,
            payload_size: 0,
            data: seg.payload().clone(),
        }
    }

    #[allow(dead_code)]
    pub fn with_ack(&mut self, ack_: bool) -> &mut SendSegment {
        self.ack = ack_;
        self
    }

    #[allow(dead_code)]
    pub fn with_rst(&mut self, rst_: bool) -> &mut SendSegment {
        self.rst = rst_;
        self
    }

    #[allow(dead_code)]
    pub fn with_syn(&mut self, syn_: bool) -> &mut SendSegment {
        self.syn = syn_;
        self
    }

    #[allow(dead_code)]
    pub fn with_fin(&mut self, fin_: bool) -> &mut SendSegment {
        self.fin = fin_;
        self
    }

    #[allow(dead_code)]
    pub fn with_seqno(&mut self, seqno_: WrappingInt32) -> &mut SendSegment {
        self.seqno = seqno_;
        self
    }

    #[allow(dead_code)]
    pub fn with_ackno(&mut self, ackno_: WrappingInt32) -> &mut SendSegment {
        self.ackno = ackno_;
        self
    }

    #[allow(dead_code)]
    pub fn with_win(&mut self, win_: u16) -> &mut SendSegment {
        self.win = win_;
        self
    }

    #[allow(dead_code)]
    pub fn with_payload_size(&mut self, payload_size_: SizeT) -> &mut SendSegment {
        self.payload_size = payload_size_;
        self
    }

    #[allow(dead_code)]
    pub fn with_data(&mut self, data_: String) -> &mut SendSegment {
        self.data = Buffer::new(data_.into_bytes());
        self
    }

    #[allow(dead_code)]
    fn get_segment(&self) -> TCPSegment {
        let mut data_hdr = TCPHeader::new();
        data_hdr.ack = self.ack;
        data_hdr.rst = self.rst;
        data_hdr.syn = self.syn;
        data_hdr.fin = self.fin;
        data_hdr.ackno = self.ackno.clone();
        data_hdr.seqno = self.seqno.clone();
        data_hdr.win = self.win;

        TCPSegment::new(data_hdr, self.data.clone())
    }
}
impl Default for SendSegment {
    fn default() -> SendSegment {
        SendSegment {
            ack: false,
            rst: false,
            syn: false,
            fin: false,
            seqno: WrappingInt32::new(0),
            ackno: WrappingInt32::new(0),
            win: 0,
            payload_size: 0,
            data: Buffer::new(vec![]),
        }
    }
}

pub struct Write {
    data: String,
    bytes_written: Option<SizeT>,
}
impl TCPTestStep for Write {
    fn execute(&mut self, h: &mut TCPTestHarness) {
        println!("  step: {}", TCPAction::to_string(self));

        let bytes_written = h.fsm.write(&self.data);
        if self.bytes_written.is_some() && bytes_written != self.bytes_written.unwrap() {
            let s = format!(
                "{} bytes should have been written but {} were",
                self.bytes_written.unwrap(),
                bytes_written
            );
            assert!(false, "{}", s);
        }
    }
}
impl TCPAction for Write {
    fn description(&self) -> String {
        let mut o = String::new();
        o.push_str(format!("write ({} bytes", self.data.len()).as_str());
        if self.bytes_written.is_some() {
            o.push_str(format!(" with {} accepted", self.bytes_written.unwrap()).as_str());
        }
        o.push_str(") [");
        o.push_str(append_data(&self.data).as_str());
        o.push_str("]");
        o.to_string()
    }
}
impl Write {
    #[allow(dead_code)]
    pub fn new(data_: String) -> Write {
        Write {
            data: data_,
            bytes_written: Option::None,
        }
    }

    #[allow(dead_code)]
    pub fn with_bytes_written(&mut self, bytes_written_: SizeT) -> &mut Write {
        let _ = self.bytes_written.insert(bytes_written_);
        self
    }
}

pub struct Tick {
    ms_since_last_tick: SizeT,
}
impl TCPTestStep for Tick {
    fn execute(&mut self, h: &mut TCPTestHarness) {
        println!("  step: {}", TCPAction::to_string(self));

        h.fsm.tick(self.ms_since_last_tick);
    }
}
impl TCPAction for Tick {
    fn description(&self) -> String {
        format!("{}ms pass", self.ms_since_last_tick)
    }
}
impl Tick {
    #[allow(dead_code)]
    pub fn new(b: SizeT) -> Tick {
        Tick {
            ms_since_last_tick: b,
        }
    }
}

pub struct Connect {}
impl TCPTestStep for Connect {
    fn execute(&mut self, h: &mut TCPTestHarness) {
        println!("  step: {}", TCPAction::to_string(self));

        h.fsm.connect();
    }
}
impl TCPAction for Connect {
    fn description(&self) -> String {
        "Connect".to_string()
    }
}

pub struct Listen {}
impl TCPTestStep for Listen {
    fn execute(&mut self, _h: &mut TCPTestHarness) {
        println!("  step: {}", TCPAction::to_string(self));
    }
}
impl TCPAction for Listen {
    fn description(&self) -> String {
        "listen".to_string()
    }
}

pub struct Close {}
impl TCPTestStep for Close {
    fn execute(&mut self, h: &mut TCPTestHarness) {
        println!("  step: {}", TCPAction::to_string(self));

        h.fsm.end_input_stream();
    }
}
impl TCPAction for Close {
    fn description(&self) -> String {
        "close".to_string()
    }
}

pub struct TCPTestHarness {
    fsm: TCPConnection,
    flt: TestFdAdapter,
}
impl TCPTestHarness {
    pub const DEFAULT_TEST_WINDOW: u32 = 137;

    #[allow(dead_code)]
    pub fn new(c_fsm: &TCPConfig) -> TCPTestHarness {
        let harness = TCPTestHarness {
            fsm: TCPConnection::new(c_fsm.clone()),
            flt: TestFdAdapter {
                test_fd: TestFD::new_pair(),
                base: FdAdapterBase::new(),
            },
        };
        println!("test:=> Initialized (config = {})", c_fsm);
        harness
    }

    #[allow(dead_code)]
    pub fn send_fin(&mut self, seqno: WrappingInt32, ackno: Option<WrappingInt32>) {
        let mut step = SendSegment {
            ..Default::default()
        };
        if ackno.is_some() {
            step.with_ack(true).with_ackno(ackno.unwrap());
        }
        step.with_fin(true)
            .with_seqno(seqno)
            .with_win(TCPTestHarness::DEFAULT_TEST_WINDOW as u16);
        self.execute(&mut step, "".to_string());
    }

    #[allow(dead_code)]
    pub fn send_ack(&mut self, seqno: WrappingInt32, ackno: WrappingInt32, swin: Option<u16>) {
        let mut win = TCPTestHarness::DEFAULT_TEST_WINDOW as u16;
        if swin.is_some() {
            win = swin.unwrap();
        }
        let mut step = SendSegment {
            ..Default::default()
        };
        self.execute(
            step.with_ack(true)
                .with_ackno(ackno)
                .with_seqno(seqno)
                .with_win(win),
            "".to_string(),
        );
    }

    #[allow(dead_code)]
    pub fn send_rst(&mut self, seqno: WrappingInt32, ackno: Option<WrappingInt32>) {
        let mut step = SendSegment {
            ..Default::default()
        };
        if ackno.is_some() {
            step.with_ack(true).with_ackno(ackno.unwrap());
        }
        step.with_rst(true)
            .with_seqno(seqno)
            .with_win(TCPTestHarness::DEFAULT_TEST_WINDOW as u16);
        self.execute(&mut step, "".to_string());
    }

    #[allow(dead_code)]
    pub fn send_syn(&mut self, seqno: WrappingInt32, ackno: Option<WrappingInt32>) {
        let mut step = SendSegment {
            ..Default::default()
        };
        if ackno.is_some() {
            step.with_ack(true).with_ackno(ackno.unwrap());
        }
        step.with_syn(true)
            .with_seqno(seqno)
            .with_win(TCPTestHarness::DEFAULT_TEST_WINDOW as u16);
        self.execute(&mut step, "".to_string());
    }

    #[allow(dead_code)]
    pub fn send_byte(&mut self, seqno: WrappingInt32, ackno: Option<WrappingInt32>, val: u8) {
        let mut step = SendSegment {
            ..Default::default()
        };
        if ackno.is_some() {
            step.with_ack(true).with_ackno(ackno.unwrap());
        }
        step.with_payload_size(1)
            .with_data(String::from_utf8(vec![val]).unwrap())
            .with_seqno(seqno)
            .with_win(TCPTestHarness::DEFAULT_TEST_WINDOW as u16);
        self.execute(&mut step, "".to_string());
    }

    #[allow(dead_code)]
    pub fn send_data(&mut self, seqno: WrappingInt32, ackno: WrappingInt32, data: &str) {
        let mut step = SendSegment {
            ..Default::default()
        };
        step.with_ack(true)
            .with_ackno(ackno)
            .with_payload_size(1)
            .with_data(data.to_string())
            .with_seqno(seqno)
            .with_win(TCPTestHarness::DEFAULT_TEST_WINDOW as u16);
        self.execute(&mut step, "".to_string());
    }

    #[allow(dead_code)]
    pub fn can_read(&self) -> bool {
        self.flt.can_read()
    }

    #[allow(dead_code)]
    pub fn execute(&mut self, step: &mut dyn TCPTestStep, _note: String) {
        step.execute(self);
        while !self.fsm.segments_out_mut().is_empty() {
            let seg = self.fsm.segments_out_mut().front().unwrap();
            let mut t = TCPSegment::new(seg.header().clone(), seg.payload().clone());
            self.flt.write_seg(&mut t);
            self.fsm.segments_out_mut().pop_front();
        }
    }

    #[allow(dead_code)]
    pub fn expect_seg(&mut self, expectation: &mut ExpectSegment, _note: String) -> TCPSegment {
        expectation.expect_seg(self)
    }

    #[allow(dead_code)]
    pub fn expect_one_seg(
        &mut self,
        expectation: &mut ExpectOneSegment,
        _note: String,
    ) -> TCPSegment {
        expectation.expect_seg(self)
    }

    #[allow(dead_code)]
    pub fn in_listen(cfg: &TCPConfig) -> TCPTestHarness {
        let mut h = TCPTestHarness::new(cfg);
        h.execute(&mut Listen {}, "".to_string());
        h
    }

    #[allow(dead_code)]
    pub fn in_syn_sent(cfg: &TCPConfig, tx_isn: WrappingInt32) -> TCPTestHarness {
        let mut c = cfg.clone();
        c.fixed_isn = Option::from(tx_isn);
        let mut h = TCPTestHarness::new(&c);
        h.execute(&mut Connect {}, "".to_string());
        let mut expect_one_segment = ExpectOneSegment::new();
        expect_one_segment
            .base
            .with_no_flags()
            .with_syn(true)
            .with_seqno(tx_isn)
            .with_payload_size(0);
        h.execute(&mut expect_one_segment, "".to_string());

        h
    }

    #[allow(dead_code)]
    pub fn in_established(
        cfg: &TCPConfig,
        tx_isn: WrappingInt32,
        rx_isn: WrappingInt32,
    ) -> TCPTestHarness {
        let mut h = TCPTestHarness::in_syn_sent(cfg, tx_isn);
        h.send_syn(rx_isn, Option::Some(tx_isn + 1));
        let mut expect_one_segment = ExpectOneSegment::new();
        expect_one_segment
            .base
            .with_no_flags()
            .with_ack(true)
            .with_ackno(rx_isn + 1)
            .with_payload_size(0);
        h.execute(&mut expect_one_segment, "".to_string());
        h
    }

    #[allow(dead_code)]
    pub fn in_close_wait(
        cfg: &TCPConfig,
        tx_isn: WrappingInt32,
        rx_isn: WrappingInt32,
    ) -> TCPTestHarness {
        let mut h = TCPTestHarness::in_established(cfg, tx_isn, rx_isn);
        h.send_fin(rx_isn + 1, Option::Some(tx_isn + 1));
        let mut expect_one_segment = ExpectOneSegment::new();
        expect_one_segment
            .base
            .with_no_flags()
            .with_ack(true)
            .with_ackno(rx_isn + 2);
        h.execute(&mut expect_one_segment, "".to_string());
        h
    }

    #[allow(dead_code)]
    pub fn in_last_ack(
        cfg: &TCPConfig,
        tx_isn: WrappingInt32,
        rx_isn: WrappingInt32,
    ) -> TCPTestHarness {
        let mut h = TCPTestHarness::in_close_wait(cfg, tx_isn, rx_isn);
        h.execute(&mut Close {}, "".to_string());
        let mut expect_one_segment = ExpectOneSegment::new();
        expect_one_segment
            .base
            .with_no_flags()
            .with_fin(true)
            .with_ack(true)
            .with_seqno(tx_isn + 1)
            .with_ackno(rx_isn + 2);
        h.execute(&mut expect_one_segment, "".to_string());
        h
    }

    #[allow(dead_code)]
    pub fn in_fin_wait_1(
        cfg: &TCPConfig,
        tx_isn: WrappingInt32,
        rx_isn: WrappingInt32,
    ) -> TCPTestHarness {
        let mut h = TCPTestHarness::in_established(cfg, tx_isn, rx_isn);
        h.execute(&mut Close {}, "".to_string());
        let mut expect_one_segment = ExpectOneSegment::new();
        expect_one_segment
            .base
            .with_no_flags()
            .with_fin(true)
            .with_ack(true)
            .with_ackno(rx_isn + 1)
            .with_seqno(tx_isn + 1);
        h.execute(&mut expect_one_segment, "".to_string());
        h
    }

    #[allow(dead_code)]
    pub fn in_fin_wait_2(
        cfg: &TCPConfig,
        tx_isn: WrappingInt32,
        rx_isn: WrappingInt32,
    ) -> TCPTestHarness {
        let mut h = TCPTestHarness::in_fin_wait_1(cfg, tx_isn, rx_isn);
        h.send_ack(rx_isn + 1, tx_isn + 2, None);
        h
    }

    #[allow(dead_code)]
    pub fn in_closing(
        cfg: &TCPConfig,
        tx_isn: WrappingInt32,
        rx_isn: WrappingInt32,
    ) -> TCPTestHarness {
        let mut h = TCPTestHarness::in_fin_wait_1(cfg, tx_isn, rx_isn);
        h.send_fin(rx_isn + 1, Option::Some(tx_isn + 1));
        let mut expect_one_segment = ExpectOneSegment::new();
        expect_one_segment
            .base
            .with_no_flags()
            .with_ack(true)
            .with_ackno(rx_isn + 2);
        h.execute(&mut expect_one_segment, "".to_string());
        h
    }

    #[allow(dead_code)]
    pub fn in_time_wait(
        cfg: &TCPConfig,
        tx_isn: WrappingInt32,
        rx_isn: WrappingInt32,
    ) -> TCPTestHarness {
        let mut h = TCPTestHarness::in_fin_wait_1(cfg, tx_isn, rx_isn);
        h.send_fin(rx_isn + 1, Option::Some(tx_isn + 2));

        let mut expect_one_segment = ExpectOneSegment::new();
        expect_one_segment
            .base
            .with_no_flags()
            .with_ack(true)
            .with_ackno(rx_isn + 2);
        h.execute(&mut expect_one_segment, "".to_string());
        h
    }
}

fn append_data(data: &String) -> String {
    format!(
        "\"{}{}\"",
        if data.len() >= 16 {
            data[..16].to_string()
        } else {
            data[..data.len()].to_string()
        },
        if data.len() > 16 { "..." } else { "" },
    )
}

pub fn check_segment(test: &mut TCPTestHarness, data: &String, multiple: bool, lineno: u32) {
    println!("  check_segment");
    test.execute(
        ExpectSegment::new()
            .with_ack(true)
            .with_payload_size(data.len())
            .with_data(data.clone()),
        "".to_string(),
    );
    if !multiple {
        test.execute(
            &mut ExpectNoSegment {},
            "test failed: multiple re-tx?".to_string(),
        );
    }
}

fn make(property_name: &str, expected_value: String, actual_value: String) -> String {
    format!(
        "The TCP has `{} = {}`, but {} was expected to be `{}`",
        property_name, actual_value, property_name, expected_value
    )
}

fn make_not(property_name: &str, expected_non_value: String) -> String {
    format!(
        "The TCP has `{} = {}`, but {} was expected to **not** be `{}`",
        property_name, expected_non_value, property_name, expected_non_value
    )
}
