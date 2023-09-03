use clap::Parser;
use halo2_base::gates::{GateChip, GateInstructions};
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
use rand::seq::SliceRandom;
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
    let similarity_chip = SimilarityChip::<F, PRECISION_BITS>::default(lookup_bits);

    // load k
    const K: usize = 4;

    let vectors: Vec<Vec<AssignedValue<F>>> = input
        .vectors
        .iter()
        .map(|v| ctx.assign_witnesses(similarity_chip.quantize_vector(&v)))
        .collect();

    // choose initial centroids (just first few vectors)
    let centroids: [Vec<AssignedValue<F>>; K] = vectors.as_slice().get(0..K).unwrap();
    let centroid_labels: [AssignedValue<F>; K] = ctx.assign_witnesses(
        centroids.iter().enumerate().map(|(i, _)| F::from(i as u64)).collect::<Vec<F>>(),
    );

    // labels, initially zero
    // TODO: is this line needed?
    let vector_labels: Vec<AssignedValue<F>> =
        ctx.assign_witnesses(vec![F::from(0); vectors.len()]);

    // k-means with fixed number of iteraitons
    const NUM_ITERS: usize = 10;
    for i in 0..NUM_ITERS {
        // compute distances between each data point and the set of centroids
        // and assign each data point to the closest centroid

        // select all data points that belong to cluster i and compute
        // the mean of these data points (each feature individually)
    }
}

fn main() {
    env_logger::init();

    let args = Cli::parse();
    run(kmeans, args);
}
