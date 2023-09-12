use assert_float_eq::afe_is_relative_eq;
use halo2_base::{
    gates::{builder::GateThreadBuilder, GateChip, GateInstructions},
    utils::ScalarField,
    QuantumCell,
    QuantumCell::Witness,
};
use halo2_proofs::halo2curves::bn256::Fr;

#[macro_use]
extern crate assert_float_eq;

mod similarities;

pub fn test_add<F: ScalarField>(inputs: &[QuantumCell<F>; 2]) -> F {
    let mut builder = GateThreadBuilder::mock();
    let ctx = builder.main(0);
    let chip = GateChip::default();
    let a = chip.add(ctx, inputs[0], inputs[1]);
    *a.value()
}

// sample test
#[test]
fn add() {
    let res: Fr = test_add(&[Witness(Fr::from(2)), Witness(Fr::from(4))]);
    println!("{:?}", res);
}

#[test]
fn euclidean_distance() {
    let a = vec![0.123, 0.456, 1.789];
    let b = vec![1.123, 0.456, 0.789];
    let dist_native = similarities::euclidean_distance(a, b);

    let other = 1.4142129712272435;
    assert_float_relative_eq!(other, dist_native);
}
