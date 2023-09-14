use clap::Parser;
use halo2_base::gates::GateInstructions;
use halo2_base::utils::ScalarField;
use halo2_base::AssignedValue;
#[allow(unused_imports)]
use halo2_base::{
    Context,
    QuantumCell::{Constant, Existing, Witness},
};
use halo2_scaffold::gadget::{
    distance::{DistanceChip, DistanceInstructions},
    fixed_point::{FixedPointChip, FixedPointInstructions},
    vectordb::{VectorDBChip, VectorDBInstructions},
};
use halo2_scaffold::scaffold::cmd::Cli;
use halo2_scaffold::scaffold::run;
use serde::{Deserialize, Serialize};
use std::env::var;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircuitInput {
    pub vectors: Vec<Vec<f64>>,
}

fn kmeans<F: ScalarField>(
    ctx: &mut Context<F>,
    input: CircuitInput,
    make_public: &mut Vec<AssignedValue<F>>,
) {
    assert!(input.vectors.iter().all(|vec| vec.len() == input.vectors[0].len()));

    let lookup_bits =
        var("LOOKUP_BITS").unwrap_or_else(|_| panic!("LOOKUP_BITS not set")).parse().unwrap();
    const PRECISION_BITS: u32 = 32;
    let fixed_point_chip = FixedPointChip::<F, PRECISION_BITS>::default(lookup_bits);
    let distance_chip = DistanceChip::default(fixed_point_chip.clone());
    let vectordb_chip = VectorDBChip::default(fixed_point_chip);

    let vectors: Vec<Vec<AssignedValue<F>>> = input
        .vectors
        .iter()
        .map(|v| ctx.assign_witnesses(distance_chip.quantize_vector(&v)))
        .collect();

    let centroids = vectordb_chip
        .kmeans::<2, 10>(ctx, &vectors, &|ctx, a, b| distance_chip.euclidean_distance(ctx, a, b));

    // output centroids as public variables
    centroids.iter().for_each(|c| {
        c.iter().for_each(|c_i| {
            make_public.push(*c_i);
        })
    });
}

fn main() {
    env_logger::init();

    let args = Cli::parse();
    run(kmeans, args);
}
