use crate::tcp_receiver::TCPReceiver;
use crate::tcp_sender::TCPSender;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum State {
    LISTEN = 0,
    SynRcvd,
    SynSent,
    ESTABLISHED,
    CloseWait,
    LastAck,
    FinWait1,
    FinWait2,
    CLOSING,
    TimeWait,
    CLOSED,
    RESET,
}

#[derive(Debug)]
pub struct TCPState {
    sender: String,
    receiver: String,
    active: bool,
    linger_after_streams_finish: bool,
}
impl TCPState {
    #[allow(dead_code)]
    pub fn new(
        sender_: &TCPSender,
        receiver_: &TCPReceiver,
        active_: bool,
        linger_: bool,
    ) -> TCPState {
        TCPState {
            sender: TCPState::state_summary_sender(sender_).to_string(),
            receiver: TCPState::state_summary(receiver_).to_string(),
            active: active_,
            linger_after_streams_finish: if active_ { linger_ } else { false },
        }
    }

    #[allow(dead_code)]
    pub fn name(&self) -> String {
        format!(
            "sender=`{}`, receiver=`{}`, active={}, linger_after_streams_finish={}",
            self.sender, self.receiver, self.active, self.linger_after_streams_finish
        )
    }

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
impl From<State> for TCPState {
    fn from(stat: State) -> Self {
        match stat {
            State::LISTEN => TCPState {
                receiver: TCPReceiverStateSummary::LISTEN.to_string(),
                sender: TCPSenderStateSummary::CLOSED.to_string(),
                active: true,
                linger_after_streams_finish: true,
            },
            State::SynRcvd => TCPState {
                receiver: TCPReceiverStateSummary::SYN_RECV.to_string(),
                sender: TCPSenderStateSummary::SYN_SENT.to_string(),
                active: true,
                linger_after_streams_finish: true,
            },
            State::SynSent => TCPState {
                receiver: TCPReceiverStateSummary::LISTEN.to_string(),
                sender: TCPSenderStateSummary::SYN_SENT.to_string(),
                active: true,
                linger_after_streams_finish: true,
            },
            State::ESTABLISHED => TCPState {
                receiver: TCPReceiverStateSummary::SYN_RECV.to_string(),
                sender: TCPSenderStateSummary::SYN_ACKED.to_string(),
                active: true,
                linger_after_streams_finish: true,
            },
            State::CloseWait => TCPState {
                receiver: TCPReceiverStateSummary::FIN_RECV.to_string(),
                sender: TCPSenderStateSummary::SYN_ACKED.to_string(),
                active: true,
                linger_after_streams_finish: false,
            },
            State::LastAck => TCPState {
                receiver: TCPReceiverStateSummary::FIN_RECV.to_string(),
                sender: TCPSenderStateSummary::FIN_SENT.to_string(),
                active: true,
                linger_after_streams_finish: false,
            },
            State::CLOSING => TCPState {
                receiver: TCPReceiverStateSummary::FIN_RECV.to_string(),
                sender: TCPSenderStateSummary::FIN_SENT.to_string(),
                active: true,
                linger_after_streams_finish: true,
            },
            State::FinWait1 => TCPState {
                receiver: TCPReceiverStateSummary::SYN_RECV.to_string(),
                sender: TCPSenderStateSummary::FIN_SENT.to_string(),
                active: true,
                linger_after_streams_finish: true,
            },
            State::FinWait2 => TCPState {
                receiver: TCPReceiverStateSummary::SYN_RECV.to_string(),
                sender: TCPSenderStateSummary::FIN_ACKED.to_string(),
                active: true,
                linger_after_streams_finish: true,
            },
            State::TimeWait => TCPState {
                receiver: TCPReceiverStateSummary::FIN_RECV.to_string(),
                sender: TCPSenderStateSummary::FIN_ACKED.to_string(),
                active: true,
                linger_after_streams_finish: true,
            },
            State::RESET => TCPState {
                receiver: TCPReceiverStateSummary::ERROR.to_string(),
                sender: TCPSenderStateSummary::ERROR.to_string(),
                active: false,
                linger_after_streams_finish: false,
            },
            State::CLOSED => TCPState {
                receiver: TCPReceiverStateSummary::FIN_RECV.to_string(),
                sender: TCPSenderStateSummary::FIN_ACKED.to_string(),
                active: false,
                linger_after_streams_finish: false,
            },
        }
    }
}
impl PartialEq<Self> for TCPState {
    fn eq(&self, other: &Self) -> bool {
        self.active == other.active
            && self.linger_after_streams_finish == other.linger_after_streams_finish
            && self.sender == other.sender
            && self.receiver == other.receiver
    }
}
impl Eq for TCPState {}
impl Clone for TCPState {
    fn clone(&self) -> TCPState {
        TCPState {
            sender: self.sender.to_string(),
            receiver: self.receiver.to_string(),
            active: self.active,
            linger_after_streams_finish: self.linger_after_streams_finish,
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
