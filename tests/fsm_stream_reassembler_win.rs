use rand::thread_rng;
use rust_sponge::stream_reassembler::StreamReassembler;
use rust_sponge::SizeT;
use std::cmp::min;

mod fsm_stream_reassembler_harness;

#[test]
fn fsm_stream_reassembler_win() {
    use rand::seq::SliceRandom;
    use rand::Rng;

    const NREPS: SizeT = 32;
    const NSEGS: SizeT = 128;
    const MAX_SEG_LEN: SizeT = 2048;

    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

    for _i in 0..NREPS {
        let mut buf = StreamReassembler::new(NSEGS * MAX_SEG_LEN);

        let mut seq_size: Vec<(SizeT, SizeT)> = Vec::new();
        let mut offset: SizeT = 0;
        for _j in 0..NSEGS {
            let size: SizeT = 1 + thread_rng().gen_range(0..(MAX_SEG_LEN - 1));
            let offs: SizeT = min(offset, 1 + thread_rng().gen_range(0..1023));
            seq_size.push((offset - offs, size + offs));

            offset = offset + size;
        }
        seq_size.shuffle(&mut thread_rng());

        let d: String = (0..offset)
            .map(|_| {
                let idx = thread_rng().gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        for (off, sz) in seq_size {
            let dd = &d[off..(off + sz)];
            buf.push_substring(&dd.to_string(), off as u64, (off + sz) == offset);
        }

        let len = buf.stream_out().buffer_size();
        let result = buf.stream_out_mut().read(len);
        assert_eq!(buf.stream_out().bytes_written(), offset);
        assert_eq!(result, d);
    }
}
