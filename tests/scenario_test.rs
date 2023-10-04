mod common;
mod distances;
mod vectordb;

// user-scenario test will be here (or maybe in examples)
//
// 1. db is trained with a set of vectors, resulting in K centroids along with merkle roots
// 2. a query vector is given, it is first compared to centroids and then to vectors within that cluster

struct ExampleDatabase<const K: usize, const I: usize> {
    database: Vec<Vec<f64>>,
    cluster_ids: Vec<usize>,
    centroids: Vec<Vec<f64>>,
}

impl<const K: usize, const I: usize> ExampleDatabase<K, I> {
    /// Indexing (i.e. training) takes in a set of vectors, and produces centroids from them.
    fn indexing(&mut self, vectors: Vec<Vec<f64>>) {
        self.database = vectors;

        // find centroids
        let (centroids, cluster_ids) =
            vectordb::kmeans::<K, I>(&vectors, &distances::euclidean_distance);
        self.centroids = centroids.to_vec();
        self.cluster_ids = cluster_ids;
    }

    /// Querying (i.e. inference) takes in a query vector, and returns the most similar vector within the database.
    fn querying(&self, vector: Vec<f64>) {
        let (centroid_no, closest_centroid) =
            vectordb::nearest_vector(&vector, &self.centroids, &distances::euclidean_distance);
    }
}
