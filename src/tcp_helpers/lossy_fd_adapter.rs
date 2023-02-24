use crate::tcp_helpers::fd_adapter::AsFdAdapterBaseMut;
use crate::tcp_helpers::tcp_config::FdAdapterConfig;
use crate::tcp_helpers::tcp_segment::TCPSegment;
use crate::util::file_descriptor::{AsFileDescriptorMut, FileDescriptor};
use crate::SizeT;
use rand::rngs::ThreadRng;
use rand::Rng;

#[derive(Debug)]
pub struct LossyFdAdapter<AdapterT> {
    adapter: AdapterT,
    rand: ThreadRng,
}
impl<AdapterT> LossyFdAdapter<AdapterT>
where
    AdapterT: AsFdAdapterBaseMut + AsFileDescriptorMut,
{
    #[allow(dead_code)]
    pub fn new(_adapter: AdapterT) -> LossyFdAdapter<AdapterT> {
        LossyFdAdapter {
            adapter: _adapter,
            rand: Default::default(),
        }
    }

    #[allow(dead_code)]
    fn should_drop(&mut self, uplink: bool) -> bool {
        let cfg = self.adapter.config();
        let loss = if uplink {
            cfg.loss_rate_up
        } else {
            cfg.loss_rate_dn
        };

        return loss != 0 && self.rand.gen_range(0..=u16::MAX) < loss;
    }

    #[allow(dead_code)]
    pub fn to_file_descripter_mut(&mut self) -> &mut FileDescriptor {
        self.adapter.as_file_descriptor_mut()
    }

    #[allow(dead_code)]
    pub fn read(&mut self) -> Option<TCPSegment> {
        let ret = <AdapterT as AsFdAdapterBaseMut>::read(&mut self.adapter);
        if self.should_drop(false) {
            return None;
        }

        ret
    }

    #[allow(dead_code)]
    pub fn write(&mut self, seg: &mut TCPSegment) {
        if self.should_drop(true) {
            return;
        }

        // https://doc.rust-lang.org/beta/rust-by-example/trait/disambiguating.html
        <AdapterT as AsFdAdapterBaseMut>::write(&mut self.adapter, seg);
    }

    #[allow(dead_code)]
    pub fn set_listening(&mut self, l: bool) {
        self.adapter.set_listening(l);
    }

    #[allow(dead_code)]
    pub fn config(&self) -> &FdAdapterConfig {
        self.adapter.config()
    }

    #[allow(dead_code)]
    pub fn config_mut(&mut self) -> &mut FdAdapterConfig {
        self.adapter.config_mut()
    }

    #[allow(dead_code)]
    pub fn tick(&mut self, _t: SizeT) {
        self.adapter.tick(_t);
    }
}
