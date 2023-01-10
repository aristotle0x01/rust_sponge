use crate::test_should_be::_test_should_be;
use rand::{thread_rng, Rng, RngCore};
use rust_sponge::wrapping_integers::WrappingInt32;
use rust_sponge::SizeT;

#[macro_use]
mod test_should_be;

#[test]
fn wrapping_integers_cmp() {
    // Comparing low-number adjacent seqnos
    test_should_be!(WrappingInt32::new(3) != WrappingInt32::new(1), true);
    test_should_be!(WrappingInt32::new(3) == WrappingInt32::new(1), false);
    // assert_eq!(WrappingInt32::new(3) != WrappingInt32::new(1), true);
    // assert_eq!(WrappingInt32::new(3) == WrappingInt32::new(1), false);

    const N_REPS: SizeT = 4096;

    let mut rd = thread_rng();

    for _i in 0..N_REPS {
        let n: u32 = rd.next_u32();
        let diff: u8 = rd.gen_range(0..=255);
        let m: u32 = n + (diff as u32);
        test_should_be!(WrappingInt32::new(n) == WrappingInt32::new(m), n == m);
        test_should_be!(WrappingInt32::new(n) != WrappingInt32::new(m), n != m);
        // assert_eq!(WrappingInt32::new(n) == WrappingInt32::new(m), n == m);
        // assert_eq!(WrappingInt32::new(n) != WrappingInt32::new(m), n != m);
    }
}
