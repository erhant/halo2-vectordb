use clap::Parser;
use halo2_base::gates::{GateChip, GateInstructions, RangeChip};
use halo2_base::utils::ScalarField;
use halo2_base::AssignedValue;
#[allow(unused_imports)]
use halo2_base::{
    Context,
    QuantumCell::{Constant, Existing, Witness},
};
use halo2_scaffold::gadget::{
    fixed_point::FixedPointInstructions,
    similarity::{SimilarityChip, SimilarityInstructions},
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
    _: &mut Vec<AssignedValue<F>>,
) {
    assert!(input.database.iter().all(|vec| vec.len() == input.query.len()));

    let lookup_bits =
        var("LOOKUP_BITS").unwrap_or_else(|_| panic!("LOOKUP_BITS not set")).parse().unwrap();
    const PRECISION_BITS: u32 = 32;
    let similarity_chip = SimilarityChip::<F, PRECISION_BITS>::default(lookup_bits);

    // quantize everything
    let query: Vec<AssignedValue<F>> =
        ctx.assign_witnesses(similarity_chip.quantize_vector(input.query));
    let database: Vec<Vec<AssignedValue<F>>> = input
        .database
        .iter()
        .map(|v| ctx.assign_witnesses(similarity_chip.quantize_vector(v.to_vec())))
        .collect();

    // compute distance to each vector
    let distances: Vec<AssignedValue<F>> = database
        .iter()
        .map(|v| similarity_chip.euclidean(ctx, v.to_vec(), query.clone()))
        .collect();

    // find the minimum
    let min: AssignedValue<F> = distances
        .clone() // TODO: can we use `iter` with reduce? maybe yes with fold
        .into_iter()
        .reduce(|acc, d| similarity_chip.fixed_point_gate().qmin(ctx, acc, d))
        .expect("unexpected error");
    let min_indicator: Vec<AssignedValue<F>> = distances
        .into_iter()
        .map(|d| similarity_chip.fixed_point_gate().range_gate().gate.is_equal(ctx, min, d))
        .collect();

    // return the vector by selecting each index with indicator
    let ans = similarity_chip.fixed_point_gate().range_gate().gate.select_by_indicator(
        ctx,
        min_indicator.clone(),
        min_indicator,
    );
}

fn main() {
    env_logger::init();

    let args = Cli::parse();
    run(exhaustive, args);
}
