use crate::tcp_receiver::TCPReceiver;
use crate::tcp_sender::TCPSender;

#[derive(Debug)]
pub struct TCPState;
impl TCPState {
    pub fn state_summary(receiver: &TCPReceiver) -> &str {
        if receiver.stream_out().error() {
            TCPReceiverStateSummary::ERROR
        } else if receiver.ackno().is_none() {
            TCPReceiverStateSummary::LISTEN
        } else if receiver.stream_out().input_ended() {
            TCPReceiverStateSummary::FIN_RECV
        } else {
            TCPReceiverStateSummary::SYN_RECV
        }
    }

    pub fn state_summary_sender(sender: &TCPSender) -> &str {
        if sender.stream_in().error() {
            TCPSenderStateSummary::ERROR
        } else if sender.next_seqno_absolute() == 0 {
            TCPSenderStateSummary::CLOSED
        } else if sender.next_seqno_absolute() == sender.bytes_in_flight() as u64 {
            TCPSenderStateSummary::SYN_SENT
        } else if !sender.stream_in().eof() {
            TCPSenderStateSummary::SYN_ACKED
        } else if sender.next_seqno_absolute() < (sender.stream_in().bytes_written() + 2) as u64 {
            TCPSenderStateSummary::SYN_ACKED
        } else if sender.bytes_in_flight() != 0 {
            TCPSenderStateSummary::FIN_SENT
        } else {
            TCPSenderStateSummary::FIN_ACKED
        }
    }
}

// ref: https://stackoverflow.com/questions/36928569/how-can-i-create-enums-with-constant-values-in-rust
#[derive(Debug)]
pub struct TCPReceiverStateSummary;
impl TCPReceiverStateSummary {
    pub const ERROR: &'static str = "error (connection was reset)";
    pub const LISTEN: &'static str = "waiting for SYN: ackno is empty";
    pub const SYN_RECV: &'static str =
        "SYN received (ackno exists), and input to stream hasn't ended";
    pub const FIN_RECV: &'static str = "input to stream has ended";
}

#[derive(Debug)]
pub struct TCPSenderStateSummary;
impl TCPSenderStateSummary {
    pub const ERROR: &'static str = "error (connection was reset)";
    pub const CLOSED: &'static str = "waiting for stream to begin (no SYN sent)";
    pub const SYN_SENT: &'static str = "stream started but nothing acknowledged";
    pub const SYN_ACKED: &'static str = "stream ongoing";
    pub const FIN_SENT: &'static str = "stream finished (FIN sent) but not fully acknowledged";
    pub const FIN_ACKED: &'static str = "stream finished and fully acknowledged";
}
