pub mod distances;
pub mod kmeans;

// #[macro_use]
// extern crate assert_float_eq;
// use assert_float_eq::afe_is_relative_eq;

// use halo2_base::{gates::builder::GateThreadBuilder, utils::ScalarField, AssignedValue};
// use halo2_proofs::halo2curves::bn256::Fr;
// use halo2_scaffold::gadget::distance::{DistanceChip, DistanceInstructions};

// use crate::kmeans::kmeans;

// const LOOKUP_BITS: usize = 13;
// const PRECISION_BITS: u32 = 48;

// fn chip_euclidean<F: ScalarField>(a: &Vec<f64>, b: &Vec<f64>) -> f64 {
//     let mut builder = GateThreadBuilder::mock();
//     let ctx = builder.main(0);
//     let distance_chip = DistanceChip::<F, PRECISION_BITS>::default(LOOKUP_BITS);

//     let qa: Vec<AssignedValue<F>> = ctx.assign_witnesses(distance_chip.quantize_vector(a));
//     let qb: Vec<AssignedValue<F>> = ctx.assign_witnesses(distance_chip.quantize_vector(b));
//     let dist: AssignedValue<F> = distance_chip.euclidean_distance(ctx, &qa, &qb);
//     distance_chip.dequantize(*dist.value())
// }

#[test]
fn test_kmeans_small() {
    let vectors = vec![vec![1.0, 1.0], vec![2.0, 1.0], vec![4.0, 3.0], vec![5.0, 4.0]];

    // fixed iterations since we have to do it that way in our circuit
    let results = kmeans::kmeans::<2, 10>(&vectors, &distances::euclidean_distance);

    // the following page does the same init strategy, i.e. takes first k vectors
    // as the initial centroids.
    // we can therefore compare our results to there
    // https://people.revoledu.com/kardi/tutorial/kMean/Online-K-Means-Clustering.html
    assert_eq!(results.0, vec![vec![1.5, 1.0], vec![4.5, 3.5]]); // centroids
    assert_eq!(results.1, vec![0, 0, 1, 1]); // cluster ids
}
