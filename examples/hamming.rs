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

    // find matching points
    let ab: Vec<AssignedValue<F>> = a
        .iter()
        .zip(&b)
        .map(|(a_i, b_i)| fixed_point_chip.gate().is_equal(ctx, *a_i, *b_i))
        .collect();
    let ab_sum: AssignedValue<F> = fixed_point_chip.gate().sum(ctx, ab);

    let len: F = fixed_point_chip.quantization(a.len() as f64);
    let len: AssignedValue<F> = ctx.load_witness(len);

    let ab_sum_q: F = fixed_point_chip.quantization(ab_sum.value().get_lower_128() as f64);
    let ab_sum_q: AssignedValue<F> = ctx.load_witness(ab_sum_q);

    let dist: AssignedValue<F> = fixed_point_chip.qdiv(ctx, ab_sum_q, len);

    make_public.push(dist);
    let dist_native = fixed_point_chip.dequantization(*dist.value());
    println!("hamming distance: {:?}", dist_native);
}

fn main() {
    env_logger::init();

    let args = Cli::parse();
    run(euclidean_distance, args);
}
