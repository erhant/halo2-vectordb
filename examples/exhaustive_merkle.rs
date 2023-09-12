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
use poseidon::PoseidonChip;
use serde::{Deserialize, Serialize};
use std::env::var;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircuitInput {
    pub query: Vec<f64>,
    pub database: Vec<Vec<f64>>,
}

const T: usize = 3;
const RATE: usize = 2;
const R_F: usize = 8;
const R_P: usize = 57;

fn exhaustive_merkle<F: ScalarField>(
    ctx: &mut Context<F>,
    input: CircuitInput,
    make_public: &mut Vec<AssignedValue<F>>,
) {
    assert!(input.database.iter().all(|vec| vec.len() == input.query.len()));

    let lookup_bits =
        var("LOOKUP_BITS").unwrap_or_else(|_| panic!("LOOKUP_BITS not set")).parse().unwrap();
    const PRECISION_BITS: u32 = 32;
    let distance_chip = DistanceChip::<F, PRECISION_BITS>::default(lookup_bits);
    let mut poseidon_chip = PoseidonChip::<F, T, RATE>::new(ctx, R_F, R_P).unwrap();

    let query: Vec<AssignedValue<F>> =
        ctx.assign_witnesses(distance_chip.quantize_vector(&input.query));
    let database: Vec<Vec<AssignedValue<F>>> = input
        .database
        .iter()
        .map(|v| ctx.assign_witnesses(distance_chip.quantize_vector(&v)))
        .collect();

    let result = distance_chip.nearest_vector(ctx, &query, &database);
    make_public.extend(result.iter());

    println!("Result:");
    for e in result {
        print!("{:?} ", distance_chip.dequantize(*e.value()));
    }
    println!("");

    // compute commitment to the database
    let root = distance_chip.merkle_commitment(ctx, &mut poseidon_chip, &database);
    make_public.push(root);
    println!("Merkle Root: {:?}", root.value());
}

fn main() {
    env_logger::init();

    let args = Cli::parse();
    run(exhaustive_merkle, args);
}