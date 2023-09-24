#![allow(dead_code)]

const LOOKUP_BITS: usize = 13;
const PRECISION_BITS: u32 = 48;

use halo2_base::halo2_proofs::halo2curves::bn256::Fr as F;
use halo2_base::{gates::builder::GateThreadBuilder, AssignedValue};
use halo2_scaffold::gadget::{
    distance::{DistanceChip, DistanceInstructions},
    fixed_point::FixedPointChip,
    fixed_point_vec::FixedPointVectorInstructions,
};

pub fn euclidean_distance(a: &Vec<f64>, b: &Vec<f64>) -> f64 {
    assert_eq!(a.len(), b.len());
    a.iter().zip(b).map(|(a, b)| (a - b).powi(2)).sum::<f64>().sqrt()
}

pub fn chip_euclidean(a: &Vec<f64>, b: &Vec<f64>) -> f64 {
    let mut builder = GateThreadBuilder::mock();
    let ctx = builder.main(0);
    let fixed_point_chip = FixedPointChip::<F, PRECISION_BITS>::default(LOOKUP_BITS);
    let distance_chip = DistanceChip::default(&fixed_point_chip);

    let qa: Vec<AssignedValue<F>> = ctx.assign_witnesses(fixed_point_chip.quantize_vector(a));
    let qb: Vec<AssignedValue<F>> = ctx.assign_witnesses(fixed_point_chip.quantize_vector(b));
    let dist: AssignedValue<F> = distance_chip.euclidean_distance(ctx, &qa, &qb);
    fixed_point_chip.dequantization(*dist.value())
}

pub fn cosine_distance(a: &Vec<f64>, b: &Vec<f64>) -> f64 {
    assert_eq!(a.len(), b.len());

    let ab: f64 = a.iter().zip(b).map(|(a, b)| a * b).sum();
    let aa: f64 = a.iter().map(|a| a * a).sum();
    let bb: f64 = b.iter().map(|b| b * b).sum();

    1.0 - (ab / (aa.sqrt() * bb.sqrt()))
}

pub fn chip_cosine(a: &Vec<f64>, b: &Vec<f64>) -> f64 {
    let mut builder = GateThreadBuilder::mock();
    let ctx = builder.main(0);
    let fixed_point_chip = FixedPointChip::<F, PRECISION_BITS>::default(LOOKUP_BITS);
    let distance_chip = DistanceChip::default(&fixed_point_chip);

    let qa: Vec<AssignedValue<F>> = ctx.assign_witnesses(fixed_point_chip.quantize_vector(a));
    let qb: Vec<AssignedValue<F>> = ctx.assign_witnesses(fixed_point_chip.quantize_vector(b));
    let dist: AssignedValue<F> = distance_chip.cosine_distance(ctx, &qa, &qb);
    fixed_point_chip.dequantization(*dist.value())
}

pub fn hamming_distance(a: &Vec<f64>, b: &Vec<f64>) -> f64 {
    assert_eq!(a.len(), b.len());
    1.0 - a.iter().zip(b).map(|(a, b)| if a == b { 1.0 } else { 0.0 }).sum::<f64>()
        / (a.len() as f64)
}

pub fn chip_hamming(a: &Vec<f64>, b: &Vec<f64>) -> f64 {
    let mut builder = GateThreadBuilder::mock();
    let ctx = builder.main(0);
    let fixed_point_chip = FixedPointChip::<F, PRECISION_BITS>::default(LOOKUP_BITS);
    let distance_chip = DistanceChip::default(&fixed_point_chip);

    let qa: Vec<AssignedValue<F>> = ctx.assign_witnesses(fixed_point_chip.quantize_vector(a));
    let qb: Vec<AssignedValue<F>> = ctx.assign_witnesses(fixed_point_chip.quantize_vector(b));
    let dist: AssignedValue<F> = distance_chip.hamming_distance(ctx, &qa, &qb);
    fixed_point_chip.dequantization(*dist.value())
}

pub fn manhattan_distance(a: &Vec<f64>, b: &Vec<f64>) -> f64 {
    assert_eq!(a.len(), b.len());
    a.iter().zip(b).map(|(a, b)| (a - b).abs()).sum()
}

pub fn chip_manhattan(a: &Vec<f64>, b: &Vec<f64>) -> f64 {
    let mut builder = GateThreadBuilder::mock();
    let ctx = builder.main(0);
    let fixed_point_chip = FixedPointChip::<F, PRECISION_BITS>::default(LOOKUP_BITS);
    let distance_chip = DistanceChip::default(&fixed_point_chip);

    let qa: Vec<AssignedValue<F>> = ctx.assign_witnesses(fixed_point_chip.quantize_vector(a));
    let qb: Vec<AssignedValue<F>> = ctx.assign_witnesses(fixed_point_chip.quantize_vector(b));
    let dist: AssignedValue<F> = distance_chip.manhattan_distance(ctx, &qa, &qb);
    fixed_point_chip.dequantization(*dist.value())
}
