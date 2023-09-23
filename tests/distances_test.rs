pub mod distances;

#[macro_use]
extern crate assert_float_eq;
use assert_float_eq::afe_is_relative_eq;
use halo2_base::{gates::builder::GateThreadBuilder, utils::ScalarField, AssignedValue};
use halo2_proofs::halo2curves::bn256::Fr;
use halo2_scaffold::gadget::{
    distance::{DistanceChip, DistanceInstructions},
    fixed_point::FixedPointChip,
    fixed_point_vec::FixedPointVectorInstructions,
};

const LOOKUP_BITS: usize = 13;
const PRECISION_BITS: u32 = 48;

fn chip_euclidean<F: ScalarField>(a: &Vec<f64>, b: &Vec<f64>) -> f64 {
    let mut builder = GateThreadBuilder::mock();
    let ctx = builder.main(0);
    let fixed_point_chip = FixedPointChip::<F, PRECISION_BITS>::default(LOOKUP_BITS);
    let distance_chip = DistanceChip::default(fixed_point_chip.clone());

    let qa: Vec<AssignedValue<F>> = ctx.assign_witnesses(fixed_point_chip.quantize_vector(a));
    let qb: Vec<AssignedValue<F>> = ctx.assign_witnesses(fixed_point_chip.quantize_vector(b));
    let dist: AssignedValue<F> = distance_chip.euclidean_distance(ctx, &qa, &qb);
    fixed_point_chip.dequantization(*dist.value())
}

#[test]
fn test_euclidean_distance() {
    let a = vec![0.123, 0.456, 1.789];
    let b = vec![1.123, 0.456, 0.789];

    let dist_native = distances::euclidean_distance(&a, &b);
    let dist_chip = chip_euclidean::<Fr>(&a, &b);
    assert_float_relative_eq!(dist_native, dist_chip);
}

fn chip_manhattan<F: ScalarField>(a: &Vec<f64>, b: &Vec<f64>) -> f64 {
    let mut builder = GateThreadBuilder::mock();
    let ctx = builder.main(0);
    let fixed_point_chip = FixedPointChip::<F, PRECISION_BITS>::default(LOOKUP_BITS);
    let distance_chip = DistanceChip::default(fixed_point_chip.clone());

    let qa: Vec<AssignedValue<F>> = ctx.assign_witnesses(fixed_point_chip.quantize_vector(a));
    let qb: Vec<AssignedValue<F>> = ctx.assign_witnesses(fixed_point_chip.quantize_vector(b));
    let dist: AssignedValue<F> = distance_chip.manhattan_distance(ctx, &qa, &qb);
    fixed_point_chip.dequantization(*dist.value())
}

#[test]
fn test_manhattan_distance() {
    let a = vec![0.123, 0.456, 1.789];
    let b = vec![1.123, 0.456, 0.789];

    let dist_native = distances::manhattan_distance(&a, &b);
    let dist_chip = chip_manhattan::<Fr>(&a, &b);
    assert_float_relative_eq!(dist_native, dist_chip);
}

fn chip_cosine<F: ScalarField>(a: &Vec<f64>, b: &Vec<f64>) -> f64 {
    let mut builder = GateThreadBuilder::mock();
    let ctx = builder.main(0);
    let fixed_point_chip = FixedPointChip::<F, PRECISION_BITS>::default(LOOKUP_BITS);
    let distance_chip = DistanceChip::default(fixed_point_chip.clone());

    let qa: Vec<AssignedValue<F>> = ctx.assign_witnesses(fixed_point_chip.quantize_vector(a));
    let qb: Vec<AssignedValue<F>> = ctx.assign_witnesses(fixed_point_chip.quantize_vector(b));
    let dist: AssignedValue<F> = distance_chip.cosine_distance(ctx, &qa, &qb);
    fixed_point_chip.dequantization(*dist.value())
}

#[test]
fn test_cosine_distance() {
    let a = vec![0.123, 0.456, 1.789];
    let b = vec![1.123, 0.456, 0.789];

    let dist_native = distances::cosine_distance(&a, &b);
    let dist_chip = chip_cosine::<Fr>(&a, &b);
    assert_float_relative_eq!(dist_native, dist_chip);
}

fn chip_hamming<F: ScalarField>(a: &Vec<f64>, b: &Vec<f64>) -> f64 {
    let mut builder = GateThreadBuilder::mock();
    let ctx = builder.main(0);
    let fixed_point_chip = FixedPointChip::<F, PRECISION_BITS>::default(LOOKUP_BITS);
    let distance_chip = DistanceChip::default(fixed_point_chip.clone());

    let qa: Vec<AssignedValue<F>> = ctx.assign_witnesses(fixed_point_chip.quantize_vector(a));
    let qb: Vec<AssignedValue<F>> = ctx.assign_witnesses(fixed_point_chip.quantize_vector(b));
    let dist: AssignedValue<F> = distance_chip.hamming_distance(ctx, &qa, &qb);
    fixed_point_chip.dequantization(*dist.value())
}
#[test]
fn test_hamming_distance() {
    let a = vec![0.123, 0.456, 1.789];
    let b = vec![1.123, 0.456, 0.789];

    let dist_native = distances::hamming_distance(&a, &b);
    let dist_chip = chip_hamming::<Fr>(&a, &b);
    assert_float_relative_eq!(dist_native, dist_chip);
}
