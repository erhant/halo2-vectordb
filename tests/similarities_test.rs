use assert_float_eq::afe_is_relative_eq;

#[macro_use]
extern crate assert_float_eq;

mod similarities;

#[test]
fn euclidean_distance() {
    let a = vec![0.123, 0.456, 1.789];
    let b = vec![1.123, 0.456, 0.789];
    let dist_native = similarities::euclidean_distance(a, b);

    let other = 1.4142129712272435;
    assert_float_relative_eq!(other, dist_native);
}
