#[macro_use]
extern crate assert_float_eq;
use assert_float_eq::afe_is_relative_eq;

mod common;
mod distances;
mod vectordb;

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
        let cluster: Vec<Vec<f64>> = (0..self.cluster_ids.len())
            .filter(|i| self.cluster_ids[*i] == cluster_id)
            .map(|i| self.database[i].clone())
            .collect();

        // choose nearest vector
        let (_, result) =
            vectordb::nearest_vector(&vector, &cluster, &distances::euclidean_distance);

        result
    }
}

struct SimpleVerifiableDatabase<const K: usize, const I: usize> {
    database: Vec<Vec<f64>>,
    cluster_ids: Vec<usize>,
    centroids: Vec<Vec<f64>>,
}

impl<const K: usize, const I: usize> SimpleVerifiableDatabase<K, I> {
    pub fn new(database: Vec<Vec<f64>>) -> Self {
        // TODO: return roots and stuff
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
        let cluster: Vec<Vec<f64>> = (0..self.cluster_ids.len())
            .filter(|i| self.cluster_ids[*i] == cluster_id)
            .map(|i| self.database[i].clone())
            .collect();

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
