use clap::Parser;
use halo2_base::gates::{GateChip, GateInstructions};
use halo2_base::utils::ScalarField;
use halo2_base::{AssignedValue, QuantumCell};
#[allow(unused_imports)]
use halo2_base::{
    Context,
    QuantumCell::{Constant, Existing, Witness},
};
use halo2_scaffold::gadget::fixed_point::{FixedPointChip, FixedPointInstructions};
use halo2_scaffold::scaffold::cmd::Cli;
use halo2_scaffold::scaffold::run;
use serde::{Deserialize, Serialize};
use std::env::var;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircuitInput {
    pub a: Vec<f64>,
    pub b: Vec<f64>,
}

fn euclidean_distance<F: ScalarField>(
    ctx: &mut Context<F>,
    input: CircuitInput,
    make_public: &mut Vec<AssignedValue<F>>,
) {
    assert_eq!(input.a.len(), input.b.len());

    // setup fixed-point chip
    let lookup_bits =
        var("LOOKUP_BITS").unwrap_or_else(|_| panic!("LOOKUP_BITS not set")).parse().unwrap();
    const PRECISION_BITS: u32 = 32;
    let fixed_point_chip = FixedPointChip::<F, PRECISION_BITS>::default(lookup_bits);

    // quantize vectors
    let a: Vec<F> = input.a.iter().map(|a_i| fixed_point_chip.quantization(*a_i)).collect();
    let b: Vec<F> = input.b.iter().map(|b_i| fixed_point_chip.quantization(*b_i)).collect();

    // assign quantizations to circuit
    let a: Vec<AssignedValue<F>> = ctx.assign_witnesses(a);
    let b: Vec<AssignedValue<F>> = ctx.assign_witnesses(b);

    // compute difference vector (a-b)
    let ab_diffs: Vec<AssignedValue<F>> =
        a.iter().zip(&b).map(|(a_i, b_i)| fixed_point_chip.qsub(ctx, *a_i, *b_i)).collect();

    // compute squares of differences
    let ab_diffsquares: Vec<AssignedValue<F>> =
        ab_diffs.iter().map(|x| fixed_point_chip.qmul(ctx, *x, *x)).collect();

    // compute sum of squares of differences
    let ab_sum: AssignedValue<F> = fixed_point_chip.qsum(ctx, ab_diffsquares);

    // euclidean distance
    let dist = fixed_point_chip.qsqrt(ctx, ab_sum);
    make_public.push(dist);

    let dist_native = fixed_point_chip.dequantization(*dist.value());
    println!("euclidean distance: {:?}", dist_native);
}

fn main() {
    env_logger::init();

    let args = Cli::parse();
    run(euclidean_distance, args);
}
