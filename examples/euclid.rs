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
    fixed_point_vec::FixedPointVectorInstructions,
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

// TODO: this example is for bugfix purposes
fn euclid_bug<F: ScalarField>(
    ctx: &mut Context<F>,
    input: CircuitInput,
    _make_public: &mut Vec<AssignedValue<F>>,
) {
    assert_eq!(input.a.len(), input.b.len());

    let lookup_bits =
        var("LOOKUP_BITS").unwrap_or_else(|_| panic!("LOOKUP_BITS not set")).parse().unwrap();
    const PRECISION_BITS: u32 = 48;
    let fixed_point_chip = FixedPointChip::<F, PRECISION_BITS>::default(lookup_bits);
    let distance_chip = DistanceChip::default(&fixed_point_chip);

    let a: Vec<AssignedValue<F>> = ctx.assign_witnesses(fixed_point_chip.quantize_vector(&input.a));
    let b: Vec<AssignedValue<F>> = ctx.assign_witnesses(fixed_point_chip.quantize_vector(&input.b));

    // let mut dists: Vec<AssignedValue<F>> = vec![];
    for _ in 0..10 {
        let _dist: AssignedValue<F> = distance_chip.euclidean_distance(ctx, &a, &b);
    }
}

fn main() {
    env_logger::init();

    let args = Cli::parse();
    run(euclid_bug, args);
}
