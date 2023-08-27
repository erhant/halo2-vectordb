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

fn cosine_distance<F: ScalarField>(
    ctx: &mut Context<F>,
    input: CircuitInput,
    make_public: &mut Vec<AssignedValue<F>>,
) {
    assert_eq!(input.a.len(), input.b.len());

    let lookup_bits =
        var("LOOKUP_BITS").unwrap_or_else(|_| panic!("LOOKUP_BITS not set")).parse().unwrap();
    const PRECISION_BITS: u32 = 32;
    let fixed_point_chip = FixedPointChip::<F, PRECISION_BITS>::default(lookup_bits);

    let a: Vec<F> = input.a.iter().map(|a_i| fixed_point_chip.quantization(*a_i)).collect();
    let b: Vec<F> = input.b.iter().map(|b_i| fixed_point_chip.quantization(*b_i)).collect();

    let a: Vec<AssignedValue<F>> = ctx.assign_witnesses(a);
    let b: Vec<AssignedValue<F>> = ctx.assign_witnesses(b);

    let ab: AssignedValue<F> = fixed_point_chip.inner_product(ctx, a.clone(), b.clone()); // sum (a.b)
    let aa = fixed_point_chip.inner_product(ctx, a.clone(), a); // sum (a^2)
    let bb = fixed_point_chip.inner_product(ctx, b.clone(), b); // sum (b^2)

    let aa_sqrt = fixed_point_chip.qsqrt(ctx, aa);
    let bb_sqrt = fixed_point_chip.qsqrt(ctx, bb);

    let denom = fixed_point_chip.qmul(ctx, aa_sqrt, bb_sqrt);
    let dist = fixed_point_chip.qdiv(ctx, ab, denom);
    make_public.push(dist);

    let dist_native = fixed_point_chip.dequantization(*dist.value());
    println!("cosine distance: {:?}", dist_native);
}

fn main() {
    env_logger::init();

    let args = Cli::parse();
    run(cosine_distance, args);
}
