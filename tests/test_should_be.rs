macro_rules! test_should_be {
    ($actual:expr, $expected:expr) => {{
        _test_should_be(
            &$actual,
            &$expected,
            $actual.to_string(),
            $expected.to_string(),
            line!(),
        );
    }};
}

pub fn _test_should_be<T: std::fmt::Display + std::cmp::PartialEq>(
    actual: &T,
    expected: &T,
    actual_s: String,
    expected_s: String,
    linen: u32,
) {
    if actual != expected {
        let ss = format!("`{}` should have been `{}`, but the former is\n\t{}\nand the latter is\n\t{} (difference of {})\n (at line {})\n",
                         actual_s, expected_s, actual.to_string(), expected.to_string(), "expected - actual", linen);
        assert!(false, "{}", ss);
    }
}
