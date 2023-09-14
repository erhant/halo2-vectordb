use halo2_scaffold::gadget::{
    fixed_point::FixedPointChip,
    vectordb::{VectorDBChip, VectorDBInstructions},
};

mod distances;

#[macro_use]
extern crate assert_float_eq;
use assert_float_eq::afe_is_relative_eq;

use halo2_base::{gates::builder::GateThreadBuilder, utils::ScalarField, AssignedValue};
use halo2_proofs::halo2curves::bn256::Fr;
use halo2_scaffold::gadget::distance::{DistanceChip, DistanceInstructions};

const LOOKUP_BITS: usize = 13;
const PRECISION_BITS: u32 = 48;

fn chip_kmeans<F: ScalarField>(vectors: &Vec<Vec<f64>>) -> (Vec<Vec<f64>>, Vec<usize>) {
    let mut builder = GateThreadBuilder::mock();
    let ctx = builder.main(0);
    let fixed_point_chip = FixedPointChip::<F, PRECISION_BITS>::default(LOOKUP_BITS);
    let distance_chip = DistanceChip::default(fixed_point_chip.clone());
    let vectordb_chip = VectorDBChip::default(fixed_point_chip);

    // quantize
    let qvectors: Vec<Vec<AssignedValue<F>>> =
        vectors.iter().map(|v| ctx.assign_witnesses(distance_chip.quantize_vector(&v))).collect();

    let (centroids, cluster_indicators) =
        vectordb_chip.kmeans::<2, 10>(ctx, &qvectors, &|ctx, a, b| {
            distance_chip.euclidean_distance(ctx, a, b)
        });

    // dequantize centroid values
    let centroids_native: Vec<Vec<f64>> = centroids
        .into_iter()
        .map(|centroid| {
            centroid.into_iter().map(|c| distance_chip.dequantize(*c.value())).collect()
        })
        .collect();

    let cluster_indicators_native: Vec<Vec<f64>> = cluster_indicators
        .into_iter()
        .map(|centroid| {
            centroid.into_iter().map(|c| distance_chip.dequantize(*c.value())).collect()
        })
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
    let vectors = vec![vec![1.0, 1.0], vec![2.0, 1.0], vec![4.0, 3.0], vec![5.0, 4.0]];

    // fixed iterations since we have to do it that way in our circuit
    let results = kmeans::<2, 10>(&vectors, &distances::euclidean_distance);

    // the following page does the same init strategy,
    // i.e. takes first k vectors as the initial centroids.
    // we can therefore compare our results to there
    // https://people.revoledu.com/kardi/tutorial/kMean/Online-K-Means-Clustering.html
    assert_eq!(results.0, vec![vec![1.5, 1.0], vec![4.5, 3.5]]); // centroids
    assert_eq!(results.1, vec![0, 0, 1, 1]); // cluster ids

    let chip_results = chip_kmeans::<Fr>(&vectors);
    println!("{:?}", chip_results.0);
    println!("{:?}", chip_results.1);
}

/// A straightforward k-means algorithm.
///
/// Given a vectors, it will try to produce `k` clusters, returning the list of centroids
/// and the cluster ids of each vector in the given order.
pub fn kmeans<const K: usize, const I: usize>(
    vectors: &Vec<Vec<f64>>,
    distance: &dyn Fn(&Vec<f64>, &Vec<f64>) -> f64,
) -> (Vec<Vec<f64>>, Vec<usize>) {
    // dimensions of each vector
    let n = vectors[0].len();

    // we take the first `k` vectors as the initial centroid
    let mut centroids: Vec<Vec<f64>> = (0..K).map(|i| vectors[i].clone()).collect();

    // number of vectors within each cluster
    let mut cluster_sizes: [usize; K] = [0; K];

    // cluster id of each vector
    let mut cluster_ids: Vec<usize> = (0..vectors.len()).map(|_| 0).collect();

    for _iter in 0..I {
        // assign each vector to closest centroid
        vectors.iter().enumerate().for_each(|(i, v)| {
            // compute distances to every centroid
            let distances: Vec<f64> = centroids.iter().map(|c| distance(v, c)).collect();

            // find the minimum (TODO: remove clone)
            let min = distances.clone().into_iter().reduce(f64::min).unwrap();

            // return the corresponding index as the cluster id
            let id = distances.into_iter().enumerate().find(|(_, d)| *d == min).unwrap().0;

            cluster_ids[i] = id;
            cluster_sizes[id] += 1;
        });

        // update centroids
        for id in 0..K {
            // mean of vectors in this cluster
            let mut mean: Vec<f64> = (0..n).map(|_| 0.0).collect();
            vectors.iter().enumerate().for_each(|(v_i, v)| {
                if cluster_ids[v_i] == id {
                    for i in 0..n {
                        mean[i] += v[i];
                    }
                }
            });
            for i in 0..n {
                mean[i] /= cluster_sizes[id] as f64;
            }

            // reset cluster size for next iteration
            cluster_sizes[id] = 0;

            // assign to centroid
            centroids[id] = mean;
        }

        // println!("{:?}:\t{:?}\n\t{:?}", _iter, centroids, cluster_ids);
    }

    (centroids, cluster_ids)
}
