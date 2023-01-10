use rand::Rng;
use rust_sponge::wrapping_integers::WrappingInt32;

#[macro_use]
mod test_should_be;

fn check_roundtrip(isn: &WrappingInt32, value: &u64, checkpoint: &u64) {
    let wrapped = WrappingInt32::wrap(*value, &isn);
    assert_eq!(WrappingInt32::unwrap(&wrapped, &isn, *checkpoint), *value);
}

#[test]
fn wrapping_integers_roundtrip() {
    let mut rd = rand::thread_rng();

    let big_offset = ((1 as u64) << 31) - 1;

    for _i in 0..1000000 {
        let offset: u64 = rd.gen_range(0..=(((1 as u32) << 31) - 1)) as u64;
        let isn = WrappingInt32::new(rd.gen_range(0..=u32::MAX));
        let val = rd.gen_range(0..=((1 as u64) << 63));

        check_roundtrip(&isn, &val, &val);
        check_roundtrip(&isn, &(val + 1), &val);
        check_roundtrip(&isn, &(val - 1), &val);
        check_roundtrip(&isn, &(val + offset), &val);
        check_roundtrip(&isn, &(val - offset), &val);
        check_roundtrip(&isn, &(val + big_offset), &val);
        check_roundtrip(&isn, &(val - big_offset), &val);
    }
}
