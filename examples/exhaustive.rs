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
    vectordb::{self, VectorDBChip, VectorDBInstructions},
};
use halo2_scaffold::scaffold::cmd::Cli;
use halo2_scaffold::scaffold::run;
use serde::{Deserialize, Serialize};
use std::env::var;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircuitInput {
    pub query: Vec<f64>,
    pub database: Vec<Vec<f64>>,
}

fn exhaustive<F: ScalarField>(
    ctx: &mut Context<F>,
    input: CircuitInput,
    make_public: &mut Vec<AssignedValue<F>>,
) {
    assert!(input.database.iter().all(|vec| vec.len() == input.query.len()));

    let lookup_bits =
        var("LOOKUP_BITS").unwrap_or_else(|_| panic!("LOOKUP_BITS not set")).parse().unwrap();
    const PRECISION_BITS: u32 = 32;
    let fixed_point_chip = FixedPointChip::<F, PRECISION_BITS>::default(lookup_bits);
    let distance_chip = DistanceChip::default(fixed_point_chip.clone());
    let vectordb_chip = VectorDBChip::default(fixed_point_chip);

    let query: Vec<AssignedValue<F>> =
        ctx.assign_witnesses(vectordb_chip.quantize_vector(&input.query));
    let database: Vec<Vec<AssignedValue<F>>> = input
        .database
        .iter()
        .map(|v| ctx.assign_witnesses(vectordb_chip.quantize_vector(&v)))
        .collect();

    // compute distance to each vector
    let result = vectordb_chip.nearest_vector(ctx, &query, &database, &|ctx, a, b| {
        distance_chip.euclidean_distance(ctx, a, b)
    });
    make_public.extend(result.iter());
    for e in result {
        println!("{:?}", vectordb_chip.dequantize(*e.value()));
    }
}

fn main() {
    env_logger::init();

    let args = Cli::parse();
    run(exhaustive, args);
}
