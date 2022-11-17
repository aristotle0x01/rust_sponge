use crate::SizeT;

#[derive(Debug)]
pub struct TcpTimer {
    retransmission_timeout: u32,
    ms_start_tick: SizeT,
    started: bool,
}
impl TcpTimer {
    #[allow(dead_code)]
    pub fn new(rtx_timeout: u32) -> TcpTimer {
        TcpTimer {
            retransmission_timeout: rtx_timeout,
            ms_start_tick: 0,
            started: false,
        }
    }

    pub fn start(&mut self, _ms_start_tick: SizeT, rtx_timeout: u32) {
        if self.started {
            return;
        }

        self.started = true;
        self.ms_start_tick = _ms_start_tick;
        self.retransmission_timeout = rtx_timeout;
    }

    pub fn stop(&mut self) {
        self.started = false;
    }

    pub fn restart(&mut self, _ms_start_tick: SizeT, rtx_timeout: u32) {
        self.stop();
        self.start(_ms_start_tick, rtx_timeout);
    }

    pub fn expire(&self, tick: SizeT) -> bool {
        if self.started && tick >= self.ms_start_tick {
            if (tick - self.ms_start_tick) >= self.retransmission_timeout as SizeT {
                return true;
            }
        }

        return false;
    }
}
