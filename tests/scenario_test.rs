const LOOKUP_BITS: usize = 13;
const PRECISION_BITS: u32 = 48;

#[macro_use]
extern crate assert_float_eq;
use assert_float_eq::afe_is_relative_eq;
use halo2_base::halo2_proofs::halo2curves::bn256::Fr as F;
use halo2_base::utils::ScalarField;
use halo2_scaffold::gadget::{fixed_point::FixedPointChip, vectordb::VectorDBChip};
use vectordb::chip_kmeans;

mod common;
mod distances;
mod vectordb;

/// From a set of vectors and their cluster ids, select
/// the vectors within that cluster id.
fn select_cluster(
    vectors: &Vec<Vec<f64>>,
    cluster_ids: &Vec<usize>,
    cluster_id: usize,
) -> Vec<Vec<f64>> {
    (0..cluster_ids.len())
        .filter(|i| cluster_ids[*i] == cluster_id)
        .map(|i| vectors[i].clone())
        .collect()
}

// user-scenario test will be here (or maybe in examples)
//
// 1. db is trained with a set of vectors, resulting in K centroids along with merkle roots
// 2. a query vector is given, it is first compared to centroids and then to vectors within that cluster

struct SimpleDatabase<const K: usize, const I: usize> {
    database: Vec<Vec<f64>>,
    cluster_ids: Vec<usize>,
    centroids: Vec<Vec<f64>>,
}

impl<const K: usize, const I: usize> SimpleDatabase<K, I> {
    pub fn new(database: Vec<Vec<f64>>) -> Self {
        let (centroids, cluster_ids) =
            vectordb::kmeans::<K, I>(&database, &distances::euclidean_distance);

        Self { database, cluster_ids, centroids: centroids.to_vec() }
    }

    /// Querying (i.e. inference) takes in a query vector, and returns the most similar vector within the database.
    pub fn ann(&self, vector: Vec<f64>) -> Vec<f64> {
        // find nearest centroid & its cluster id
        let (cluster_id, _) =
            vectordb::nearest_vector(&vector, &self.centroids, &distances::euclidean_distance);

        // get vectors within the cluster
        let cluster = select_cluster(&self.database, &self.cluster_ids, cluster_id);

        // choose nearest vector within the cluster
        let (_, result) =
            vectordb::nearest_vector(&vector, &cluster, &distances::euclidean_distance);

        result
    }
}

struct SimpleVerifiableDatabase<const K: usize, const I: usize> {
    database: Vec<Vec<f64>>,
    cluster_ids: Vec<usize>,
    centroids: Vec<Vec<f64>>,
    // chip: VectorDBChip<'a, F, PRECISION_BITS>,
    database_root: F,      // merkle root over the database
    centroids_root: F,     // merkle root over the centroids
    cluster_roots: Vec<F>, // one merkle root for each cluster
}

impl<'a, const K: usize, const I: usize> SimpleVerifiableDatabase<K, I> {
    pub fn new(database: Vec<Vec<f64>>) -> Self {
        let fixed_point_chip = FixedPointChip::<F, PRECISION_BITS>::default(LOOKUP_BITS);
        let vectordb_chip = VectorDBChip::default(&fixed_point_chip);

        let (centroids, cluster_ids) = vectordb::chip_kmeans::<K, I>(&database);
        let centroids = centroids.to_vec();

        let database_root: F = vectordb::chip_merkle(&database);
        let centroids_root: F = vectordb::chip_merkle(&centroids);
        let cluster_roots: Vec<F> = (0..centroids.len())
            .map(|cluster_id| {
                let cluster = select_cluster(&database, &cluster_ids, cluster_id);
                vectordb::chip_merkle(&cluster)
            })
            .collect();

        Self { database, cluster_ids, centroids, database_root, centroids_root, cluster_roots }
    }

    /// Querying (i.e. inference) takes in a query vector, and returns the most similar vector within the database.
    pub fn ann(&self, vector: Vec<f64>) -> Vec<f64> {
        // find nearest centroid & its cluster id
        let (cluster_id, _) =
            vectordb::nearest_vector(&vector, &self.centroids, &distances::euclidean_distance);

        // get vectors within the cluster
        let cluster = select_cluster(&self.database, &self.cluster_ids, cluster_id);

        // choose nearest vector
        let (_, result) =
            vectordb::nearest_vector(&vector, &cluster, &distances::euclidean_distance);

        result
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_scenario_native() {
        const DIM: usize = 4;
        // let query = common::random_vector(DIM);
        let vectors = common::random_vectors(DIM, 4);
        let query = vectors[0].clone();

        let db = SimpleDatabase::<2, 4>::new(vectors.clone());
        let result = db.ann(query.clone());

        // println!("DB:\n{:?}", vectors);
        // println!("QUERY:\n{:?}", query);
        // println!("RESULT:\n{:?}", result);
        common::compare_vectors(&query, &result);
    }
}
