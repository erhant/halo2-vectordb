use clap::Parser;
use halo2_base::{
    gates::{GateChip, GateInstructions},
    utils::ScalarField,
    AssignedValue, Context,
};
use halo2_scaffold::{
    gadget::{
        fixed_point::FixedPointInstructions,
        similarity::{SimilarityChip, SimilarityInstructions},
    },
    scaffold::{cmd::Cli, run},
};
use poseidon::PoseidonChip;
use serde::{Deserialize, Serialize};
use std::env::var;

const T: usize = 3;
const RATE: usize = 2;
const R_F: usize = 8;
const R_P: usize = 57;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircuitInput {
    pub database: Vec<Vec<f64>>,
}

fn merkle_poseidon<F: ScalarField>(
    ctx: &mut Context<F>,
    input: CircuitInput,
    make_public: &mut Vec<AssignedValue<F>>,
) {
    assert!(input.database.iter().all(|vec| vec.len() == input.database[0].len()));

    let lookup_bits =
        var("LOOKUP_BITS").unwrap_or_else(|_| panic!("LOOKUP_BITS not set")).parse().unwrap();
    const PRECISION_BITS: u32 = 32;
    let similarity_chip = SimilarityChip::<F, PRECISION_BITS>::default(lookup_bits);
    let mut poseidon_chip = PoseidonChip::<F, T, RATE>::new(ctx, R_F, R_P).unwrap();

    let database: Vec<Vec<AssignedValue<F>>> = input
        .database
        .iter()
        .map(|v| ctx.assign_witnesses(similarity_chip.quantize_vector(&v)))
        .collect();

    let root = similarity_chip.merkle_commitment(ctx, &mut poseidon_chip, &database);

    make_public.push(root);
    println!("merkle root: {:?}", root.value());
}

fn main() {
    env_logger::init();

    let args = Cli::parse();
    run(merkle_poseidon, args);
}
