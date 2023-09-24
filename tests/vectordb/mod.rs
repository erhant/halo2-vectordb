const LOOKUP_BITS: usize = 13;
const PRECISION_BITS: u32 = 48;

use halo2_base::halo2_proofs::halo2curves::bn256::Fr as F;
use halo2_base::{gates::builder::GateThreadBuilder, AssignedValue};
use halo2_scaffold::gadget::distance::{DistanceChip, DistanceInstructions};
use halo2_scaffold::gadget::{
    fixed_point::FixedPointChip,
    fixed_point_vec::FixedPointVectorInstructions,
    vectordb::{VectorDBChip, VectorDBInstructions},
};

/// A straightforward k-means algorithm.
///
/// Given a vectors, it will try to produce `k` clusters, returning the list of centroids
/// and the cluster ids of each vector in the given order.
pub fn kmeans<const K: usize, const I: usize>(
    vectors: &Vec<Vec<f64>>,
    distance: &dyn Fn(&Vec<f64>, &Vec<f64>) -> f64,
) -> ([Vec<f64>; K], Vec<usize>) {
    // dimensions of each vector
    let n = vectors[0].len();

    // take first K vectors as the initial centroids
    let mut centroids: [Vec<f64>; K] =
        vectors.iter().take(K).cloned().collect::<Vec<Vec<f64>>>().try_into().unwrap();

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
            let min: f64 = distances.iter().fold(f64::INFINITY, |a, &b| a.min(b));

            // return the corresponding index as the cluster id
            let id: usize = distances.into_iter().enumerate().find(|(_, d)| *d == min).unwrap().0;

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

pub fn chip_kmeans<const K: usize, const I: usize>(
    vectors: &Vec<Vec<f64>>,
) -> ([Vec<f64>; K], Vec<usize>) {
    let mut builder = GateThreadBuilder::mock();
    let ctx = builder.main(0);
    let fixed_point_chip = FixedPointChip::<F, PRECISION_BITS>::default(LOOKUP_BITS);
    let distance_chip = DistanceChip::default(&fixed_point_chip);
    let vectordb_chip = VectorDBChip::default(&fixed_point_chip);

    let qvectors: Vec<Vec<AssignedValue<F>>> = vectors
        .iter()
        .map(|v| ctx.assign_witnesses(fixed_point_chip.quantize_vector(&v)))
        .collect();

    let (centroids, cluster_indicators) =
        vectordb_chip.kmeans::<K, I>(ctx, &qvectors, &|ctx, a, b| {
            distance_chip.euclidean_distance(ctx, a, b)
        });

    let centroids_native: [Vec<f64>; K] =
        centroids.map(|centroid| fixed_point_chip.dequantize_vector(&centroid));

    // a vector of 1.0s and 0.0s for each vector
    let cluster_indicators_native: Vec<[f64; K]> = cluster_indicators
        .into_iter()
        .map(|centroid| fixed_point_chip.dequantize_array(&centroid))
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

/// An exhaustive search to find the most similar vector among a database to a given query vector.
///
/// The respective distance function is given as a parameter.
pub fn nearest_vector(
    query: &Vec<f64>,
    vectors: &Vec<Vec<f64>>,
    distance: &dyn Fn(&Vec<f64>, &Vec<f64>) -> f64,
) -> Vec<f64> {
    let distances: Vec<f64> = vectors.iter().map(|v| distance(v, query)).collect();

    let min = distances.iter().fold(f64::INFINITY, |a, &b| a.min(b));

    vectors
        .iter()
        .enumerate()
        .find(|(i, _)| min == distances[*i])
        .and_then(|(_, v)| Some(v))
        .expect("should have found a minimum")
        .to_owned()
}

pub fn chip_nearest_vector(query: &Vec<f64>, vectors: &Vec<Vec<f64>>) -> Vec<f64> {
    let mut builder = GateThreadBuilder::mock();
    let ctx = builder.main(0);
    let fixed_point_chip = FixedPointChip::<F, PRECISION_BITS>::default(LOOKUP_BITS);
    let distance_chip = DistanceChip::default(&fixed_point_chip);
    let vectordb_chip = VectorDBChip::default(&fixed_point_chip);

    let qquery: Vec<AssignedValue<F>> = fixed_point_chip.quantize_and_assign_vector(ctx, query);
    let qvectors: Vec<Vec<AssignedValue<F>>> = vectors
        .iter()
        .map(|v| ctx.assign_witnesses(fixed_point_chip.quantize_vector(&v)))
        .collect();

    let result = vectordb_chip.nearest_vector(ctx, &qquery, &qvectors, &|ctx, a, b| {
        distance_chip.euclidean_distance(ctx, a, b)
    });

    fixed_point_chip.dequantize_vector(&result)
}
