use clap::Parser;
use halo2_base::utils::ScalarField;
use halo2_base::AssignedValue;
#[allow(unused_imports)]
use halo2_base::{
    Context,
    QuantumCell::{Constant, Existing, Witness},
};
use halo2_scaffold::gadget::{
    distance::{DistanceChip, DistanceInstructions},
    fixed_point::FixedPointChip,
};
use halo2_scaffold::scaffold::cmd::Cli;
use halo2_scaffold::scaffold::run;
use serde::{Deserialize, Serialize};
use std::env::var;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircuitInput {
    pub a: Vec<f64>,
    pub b: Vec<f64>,
}

fn distance_functions<F: ScalarField>(
    ctx: &mut Context<F>,
    input: CircuitInput,
    make_public: &mut Vec<AssignedValue<F>>,
) {
    assert_eq!(input.a.len(), input.b.len());

    let lookup_bits =
        var("LOOKUP_BITS").unwrap_or_else(|_| panic!("LOOKUP_BITS not set")).parse().unwrap();
    const PRECISION_BITS: u32 = 32;
    let fixed_point_chip = FixedPointChip::<F, PRECISION_BITS>::default(lookup_bits);
    let distance_chip = DistanceChip::default(fixed_point_chip);

    let a: Vec<AssignedValue<F>> = ctx.assign_witnesses(distance_chip.quantize_vector(&input.a));
    let b: Vec<AssignedValue<F>> = ctx.assign_witnesses(distance_chip.quantize_vector(&input.b));

    let dist: AssignedValue<F> = distance_chip.euclidean_distance(ctx, &a, &b);
    let dist_native = distance_chip.dequantize(*dist.value());
    println!("euclidean distance: {:?}", dist_native);
    make_public.push(dist);

    let dist: AssignedValue<F> = distance_chip.manhattan_distance(ctx, &a, &b);
    let dist_native = distance_chip.dequantize(*dist.value());
    println!("manhattan distance: {:?}", dist_native);
    make_public.push(dist);

    let dist: AssignedValue<F> = distance_chip.cosine_distance(ctx, &a, &b);
    let dist_native = distance_chip.dequantize(*dist.value());
    println!("cosine distance: {:?}", dist_native);
    make_public.push(dist);

    let dist: AssignedValue<F> = distance_chip.hamming_distance(ctx, &a, &b);
    let dist_native = distance_chip.dequantize(*dist.value());
    println!("hamming distance: {:?}", dist_native);
    make_public.push(dist);

    // What do quantized fields for zero and one look like?
    // println!("1.0 = {:?}", distance_chip.quantize(1.0));
    // println!("0.0 = {:?}", distance_chip.quantize(0.0));
}

fn main() {
    env_logger::init();

    let args = Cli::parse();
    run(distance_functions, args);
}
