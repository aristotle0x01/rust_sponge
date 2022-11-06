use rand::thread_rng;
use rust_sponge::stream_reassembler::StreamReassembler;
use rust_sponge::SizeT;

mod fsm_stream_reassembler_harness;

#[test]
fn fsm_stream_reassembler_many() {
    use rand::seq::SliceRandom;
    use rand::Rng;

    const NREPS: SizeT = 32;
    const NSEGS: SizeT = 128;
    const MAX_SEG_LEN: SizeT = 2048;

    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

    {
        for _i in 0..NREPS {
            let mut buf = StreamReassembler::new(NSEGS * MAX_SEG_LEN);

            let mut seq_size: Vec<(SizeT, SizeT)> = Vec::new();
            let mut offset: SizeT = 0;
            for _j in 0..NSEGS {
                let size: SizeT = 1 + thread_rng().gen_range(0..(MAX_SEG_LEN - 1));
                seq_size.push((offset, size));

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

    {
        for _i in 0..NREPS {
            let mut buf = StreamReassembler::new(65000);

            let size: SizeT = 1024;
            let d: String = (0..size)
                .map(|_| {
                    let idx = thread_rng().gen_range(0..CHARSET.len());
                    CHARSET[idx] as char
                })
                .collect();

            buf.push_substring(&d, 0, false);
            let d_sub = &d[10..];
            buf.push_substring(&d_sub.to_string(), (size + 10) as u64, false);

            let len1 = buf.stream_out().buffer_size();
            let res1 = buf.stream_out_mut().read(len1);
            assert_eq!(buf.stream_out().bytes_written(), size);
            assert_eq!(&res1[0..res1.len()], &d[0..res1.len()]);

            buf.push_substring(&d[0..7].to_string(), size as u64, false);
            buf.push_substring(&d[7..8].to_string(), (size + 7) as u64, true);

            let len2 = buf.stream_out().buffer_size();
            let res2 = buf.stream_out_mut().read(len2);
            assert_eq!(buf.stream_out().bytes_written(), size + 8);
            assert_eq!(&res2[0..res2.len()], &d[0..res2.len()]);
        }
    }

    for _i in 0..NREPS {
        let mut buf = StreamReassembler::new(65000);

        let size: SizeT = 1024;
        let d: String = (0..size)
            .map(|_| {
                let idx = thread_rng().gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        buf.push_substring(&d, 0, false);
        let d_sub = &d[10..];
        buf.push_substring(&d_sub.to_string(), (size + 10) as u64, false);

        let len1 = buf.stream_out().buffer_size();
        let res1 = buf.stream_out_mut().read(len1);
        assert_eq!(buf.stream_out().bytes_written(), size);
        assert_eq!(&res1[0..res1.len()], &d[0..res1.len()]);

        buf.push_substring(&d[0..15].to_string(), size as u64, true);

        let len2 = buf.stream_out().buffer_size();
        let res2 = buf.stream_out_mut().read(len2);
        assert!(
            !(buf.stream_out().bytes_written() != 2 * size
                && buf.stream_out().bytes_written() != (size + 15))
        );
        assert_eq!(&res2[0..res2.len()], &d[0..res2.len()]);
    }
}
