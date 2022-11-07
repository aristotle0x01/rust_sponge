use crate::byte_stream_harness::*;
use rust_sponge::SizeT;

mod byte_stream_harness;

#[test]
fn byte_stream_many_writes() {
    use rand::Rng;

    const NREPS: SizeT = 1000;
    const MIN_WRITE: SizeT = 10;
    const MAX_WRITE: SizeT = 200;
    const CAPACITY: SizeT = MAX_WRITE * NREPS;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

    {
        let mut test = ByteStreamTestHarness::new(String::from("many writes"), CAPACITY);

        let mut acc: SizeT = 0;
        for _i in 0..NREPS {
            let rd1 = rand::thread_rng().gen_range(1..=1000000);
            let size = MIN_WRITE + (rd1 % (MAX_WRITE - MIN_WRITE));
            // https://github.com/HKarimiA/rust-generate-random-string
            let d: String = (0..size)
                .map(|_| {
                    let idx = rand::thread_rng().gen_range(0..CHARSET.len());
                    CHARSET[idx] as char
                })
                .collect();

            test.execute(
                Write {
                    data: d,
                    bytes_written: None,
                }
                .with_bytes_written(size),
            );
            acc += size;

            test.execute(&InputEnded { input_ended: false });
            test.execute(&BufferEmpty {
                buffer_empty: false,
            });
            test.execute(&Eof { eof: false });
            test.execute(&BytesRead { bytes_read: 0 });
            test.execute(&BytesWritten { bytes_written: acc });
            test.execute(&RemainingCapacity {
                remaining_capacity: CAPACITY - acc,
            });
            test.execute(&BufferSize { buffer_size: acc });
        }
    }
}
