use clap::Parser;
use halo2_base::utils::ScalarField;
use halo2_base::AssignedValue;
#[allow(unused_imports)]
use halo2_base::{
    Context,
    QuantumCell::{Constant, Existing, Witness},
};
use halo2_scaffold::gadget::distance::{DistanceChip, DistanceInstructions};
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
    let distance_chip = DistanceChip::<F, PRECISION_BITS>::default(lookup_bits);

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
}

fn main() {
    env_logger::init();

    let args = Cli::parse();
    run(distance_functions, args);
}
