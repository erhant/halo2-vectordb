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
    fixed_point::FixedPointInstructions,
    similarity::{SimilarityChip, SimilarityInstructions},
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
    let similarity_chip = SimilarityChip::<F, PRECISION_BITS>::default(lookup_bits);

    // load k
    const K: usize = 4;

    let vectors: Vec<Vec<AssignedValue<F>>> = input
        .vectors
        .iter()
        .map(|v| ctx.assign_witnesses(similarity_chip.quantize_vector(&v)))
        .collect();

    // choose initial centroids (just first few vectors)
    let mut centroids: Vec<Vec<AssignedValue<F>>> = vectors.clone().into_iter().take(K).collect();

    // k-means with fixed number of iteraitons
    const NUM_ITERS: usize = 10;
    for _ in 0..NUM_ITERS {
        // compute distances between each data point and the set of centroids
        // assign each data point to the closest centroid
        //
        // instead of assigning an id to each vector, we store an indicator (one-hot encoding)
        let vector_cluster_indicators: Vec<Vec<AssignedValue<F>>> = vectors
            .clone()
            .iter()
            .map(|v| {
                // compute distance to centroids
                let distances: Vec<AssignedValue<F>> = centroids
                    .iter()
                    .map(|c| similarity_chip.euclidean_distance(ctx, c, v))
                    .collect();

                // find the minimum
                let min: AssignedValue<F> = distances
                    .clone()
                    .into_iter()
                    .reduce(|acc, d| similarity_chip.fixed_point_gate().qmin(ctx, acc, d))
                    .expect("unexpected error");

                // return indicator
                distances
                    .into_iter()
                    .map(|d| {
                        let eq = similarity_chip.fixed_point_gate().gate().is_equal(ctx, min, d);

                        // quantized ones and zeros
                        let one = ctx.load_witness(similarity_chip.quantize(1.0));
                        let zero = ctx.load_witness(similarity_chip.quantize(0.0));

                        similarity_chip.fixed_point_gate().gate().select(ctx, one, zero, eq)
                    })
                    .collect()
            })
            .collect();

        // compute cluster sizes
        //
        // index-wise summation of indicators will give the cluster sizes
        // this will be used to take the mean value after computing sum of
        // vectors within the cluster
        let cluster_sizes: Vec<AssignedValue<F>> = vector_cluster_indicators
            .clone()
            .into_iter()
            .reduce(|acc, cluster_indicator| {
                acc.into_iter()
                    .zip(cluster_indicator)
                    .map(|(a, c)| similarity_chip.fixed_point_gate().qadd(ctx, a, c))
                    .collect()
            })
            .unwrap();

        // for each cluster compute the mean of vectors
        //
        // we can use indicator indices for each cluster, by multiplying the results
        // with the indicator which is known to be 1 or 0
        for ci in 0..K {
            // filtered vectors, obtained by multiplying each element of the vector with either 1 or 0
            // depending on the cluster indicator
            let filtered_vectors: Vec<Vec<AssignedValue<F>>> = vectors
                .clone()
                .into_iter()
                .zip(vector_cluster_indicators.clone())
                .map(|(v, indicator)| {
                    //
                    v.into_iter()
                        .map(|v_i| similarity_chip.fixed_point_gate().qmul(ctx, v_i, indicator[ci]))
                        .collect()
                })
                .collect();

            // sum of vectors in this cluster
            let sum: Vec<AssignedValue<F>> = filtered_vectors
                .into_iter()
                .reduce(|acc, v| {
                    v.into_iter()
                        .zip(acc)
                        .map(|(v_i, acc_i)| {
                            similarity_chip.fixed_point_gate().qadd(ctx, v_i, acc_i)
                        })
                        .collect()
                })
                .unwrap();

            // mean of the vectors in this cluster, assigned directly to the centroid
            centroids[ci] = sum
                .into_iter()
                .map(|v| similarity_chip.fixed_point_gate().qdiv(ctx, v, cluster_sizes[ci]))
                .collect();
        }
    }

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
