use crate::test_should_be::_test_should_be;
use rust_sponge::wrapping_integers::WrappingInt32;

#[macro_use]
mod test_should_be;

#[test]
fn wrapping_integers_wrap() {
    test_should_be!(
        WrappingInt32::wrap(3 * (1 << 32), &WrappingInt32::new(0)),
        WrappingInt32::new(0)
    );
    test_should_be!(
        WrappingInt32::wrap(3 * (1 << 32) + 17, &WrappingInt32::new(15)),
        WrappingInt32::new(32)
    );
    test_should_be!(
        WrappingInt32::wrap(7 * (1 << 32) - 2, &WrappingInt32::new(15)),
        WrappingInt32::new(13)
    );
}
