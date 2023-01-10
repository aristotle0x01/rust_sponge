use crate::test_should_be::_test_should_be;
use rust_sponge::wrapping_integers::WrappingInt32;

#[macro_use]
mod test_should_be;

#[test]
fn wrapping_integers_unwrap() {
    // Unwrap the first byte after ISN
    test_should_be!(
        WrappingInt32::unwrap(&WrappingInt32::new(1), &WrappingInt32::new(0), 0),
        1
    );
    // Unwrap the first byte after the first wrap
    test_should_be!(
        WrappingInt32::unwrap(
            &WrappingInt32::new(1),
            &WrappingInt32::new(0),
            u32::MAX as u64
        ),
        ((1 as u64) << 32) + 1
    );
    // Unwrap the last byte before the third wrap
    test_should_be!(
        WrappingInt32::unwrap(
            &WrappingInt32::new(u32::MAX - 1),
            &WrappingInt32::new(0),
            3 * ((1 as u64) << 32)
        ),
        3 * ((1 as u64) << 32) - 2
    );
    // Unwrap the 10th from last byte before the third wrap
    test_should_be!(
        WrappingInt32::unwrap(
            &WrappingInt32::new(u32::MAX - 10),
            &WrappingInt32::new(0),
            3 * ((1 as u64) << 32)
        ),
        3 * ((1 as u64) << 32) - 11
    );
    // Non-zero ISN
    test_should_be!(
        WrappingInt32::unwrap(
            &WrappingInt32::new(u32::MAX),
            &WrappingInt32::new(10),
            3 * ((1 as u64) << 32)
        ),
        3 * ((1 as u64) << 32) - 11
    );
    // Big unwrap
    test_should_be!(
        WrappingInt32::unwrap(&WrappingInt32::new(u32::MAX), &WrappingInt32::new(0), 0),
        u32::MAX as u64
    );
    // Unwrap a non-zero ISN
    test_should_be!(
        WrappingInt32::unwrap(&WrappingInt32::new(16), &WrappingInt32::new(16), 0),
        0
    );

    // Big unwrap with non-zero ISN
    test_should_be!(
        WrappingInt32::unwrap(&WrappingInt32::new(15), &WrappingInt32::new(16), 0),
        u32::MAX as u64
    );
    // Big unwrap with non-zero ISN
    test_should_be!(
        WrappingInt32::unwrap(
            &WrappingInt32::new(0),
            &WrappingInt32::new((i32::MAX) as u32),
            0
        ),
        (i32::MAX as u64) + 2
    );
    // Barely big unwrap with non-zero ISN
    test_should_be!(
        WrappingInt32::unwrap(
            &WrappingInt32::new(u32::MAX),
            &WrappingInt32::new((i32::MAX) as u32),
            0
        ),
        ((1 as u64) << 31)
    );
    // Nearly big unwrap with non-zero ISN
    test_should_be!(
        WrappingInt32::unwrap(
            &WrappingInt32::new(u32::MAX),
            &WrappingInt32::new((1 << 31) as u32),
            0
        ),
        (u32::MAX as u64) >> 1
    );
}
