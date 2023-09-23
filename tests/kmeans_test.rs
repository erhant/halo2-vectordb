use halo2_scaffold::gadget::{
    fixed_point::FixedPointChip,
    fixed_point_vec::FixedPointVectorInstructions,
    vectordb::{VectorDBChip, VectorDBInstructions},
};

mod distances;
mod vectordb;

#[macro_use]
extern crate assert_float_eq;
use assert_float_eq::afe_is_relative_eq;

use halo2_base::{gates::builder::GateThreadBuilder, utils::ScalarField, AssignedValue};
use halo2_proofs::halo2curves::bn256::Fr;
use halo2_scaffold::gadget::distance::{DistanceChip, DistanceInstructions};

const LOOKUP_BITS: usize = 13;
const PRECISION_BITS: u32 = 48;

fn chip_kmeans<F: ScalarField, const K: usize, const I: usize>(
    vectors: &Vec<Vec<f64>>,
) -> ([Vec<f64>; K], Vec<usize>) {
    let mut builder = GateThreadBuilder::mock();
    let ctx = builder.main(0);
    let fixed_point_chip = FixedPointChip::<F, PRECISION_BITS>::default(LOOKUP_BITS);
    let distance_chip = DistanceChip::default(fixed_point_chip.clone());
    let vectordb_chip = VectorDBChip::default(fixed_point_chip.clone());

    // quantize
    let qvectors: Vec<Vec<AssignedValue<F>>> = vectors
        .iter()
        .map(|v| ctx.assign_witnesses(fixed_point_chip.quantize_vector(&v)))
        .collect();

    let (centroids, cluster_indicators) =
        vectordb_chip.kmeans::<K, I>(ctx, &qvectors, &|ctx, a, b| {
            distance_chip.euclidean_distance(ctx, a, b)
        });

    // dequantize centroid values
    let centroids_native: [Vec<f64>; K] = centroids.map(|centroid| {
        centroid.into_iter().map(|c| fixed_point_chip.dequantization(*c.value())).collect()
    });

    // a vector of 1.0s and 0.0s for each vector
    let cluster_indicators_native: Vec<[f64; K]> = cluster_indicators
        .into_iter()
        .map(|centroid| centroid.map(|c| fixed_point_chip.dequantization(*c.value())))
        .collect();

    let cluster_ids: Vec<usize> = cluster_indicators_native
        .into_iter()
        .map(|cluster_indicator| {
            // the first index that has 1 is the cluster id
            for (i, ind) in cluster_indicator.iter().enumerate() {
                if *ind == 1.0 {
                    return i;
                }
            }
            unreachable!("expected 1 to appear in indicator");
        })
        .collect();

    (centroids_native, cluster_ids)
}

#[test]
fn test_kmeans_small() {
    const K: usize = 2;
    const I: usize = 10;
    let vectors = vec![vec![1.0, 1.0], vec![2.0, 1.0], vec![4.0, 3.0], vec![5.0, 4.0]];

    // fixed iterations since we have to do it that way in our circuit
    let results = vectordb::kmeans::<K, I>(&vectors, &distances::euclidean_distance);

    // the following page does the same init strategy,
    // i.e. takes first k vectors as the initial centroids.
    // we can therefore compare our results to there
    // https://people.revoledu.com/kardi/tutorial/kMean/Online-K-Means-Clustering.html
    assert_eq!(results.0, [vec![1.5, 1.0], vec![4.5, 3.5]]); // centroids
    assert_eq!(results.1, [0, 0, 1, 1]); // cluster ids

    let chip_results = chip_kmeans::<Fr, K, I>(&vectors);
    println!("{:?}", chip_results.0);
    println!("{:?}", chip_results.1);
    assert_eq!(results.0, chip_results.0); // centroids
    assert_eq!(results.1, chip_results.1); // cluster ids
}
