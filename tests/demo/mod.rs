use crate::{common, distances, vectordb};
use halo2_base::halo2_proofs::halo2curves::bn256::Fr as F;
pub struct DemoDB<const K: usize, const I: usize> {
    database: Vec<Vec<f64>>,
    cluster_ids: Vec<usize>,
    centroids: Vec<Vec<f64>>,
}

impl<const K: usize, const I: usize> DemoDB<K, I> {
    /// Create a new d
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
        let cluster = common::select_cluster(&self.database, &self.cluster_ids, cluster_id);

        // choose nearest vector within the cluster
        let (_, result) =
            vectordb::nearest_vector(&vector, &cluster, &distances::euclidean_distance);

        result
    }
}

pub struct DemoZKDB<const K: usize, const I: usize> {
    database: Vec<Vec<f64>>,
    cluster_ids: Vec<usize>,
    centroids: Vec<Vec<f64>>,
    // chip: VectorDBChip<'a, F, PRECISION_BITS>,
    database_root: F,      // merkle root over the database
    centroids_root: F,     // merkle root over the centroids
    cluster_roots: Vec<F>, // one merkle root for each cluster
}

impl<'a, const K: usize, const I: usize> DemoZKDB<K, I> {
    pub fn new(database: Vec<Vec<f64>>) -> Self {
        let (centroids, cluster_ids) = vectordb::chip_kmeans::<K, I>(&database);
        let centroids = centroids.to_vec();

        let database_root: F = vectordb::chip_merkle(&database);
        let centroids_root: F = vectordb::chip_merkle(&centroids);
        let cluster_roots: Vec<F> = (0..centroids.len())
            .map(|cluster_id| {
                let cluster = common::select_cluster(&database, &cluster_ids, cluster_id);
                vectordb::chip_merkle(&cluster)
            })
            .collect();

        assert_eq!(cluster_ids.len(), database.len());
        assert_eq!(cluster_roots.len(), centroids.len());

        Self { database, cluster_ids, centroids, database_root, centroids_root, cluster_roots }
    }

    /// Querying (i.e. inference) takes in a query vector, and returns the most similar vector within the database.
    pub fn ann(&self, vector: Vec<f64>) -> Vec<f64> {
        // find nearest centroid & its cluster id
        let (cluster_id, _, centroids_root) =
            vectordb::chip_nearest_vector(&vector, &self.centroids);
        assert!(
            common::compare_fields(&self.centroids_root, &centroids_root),
            "centriod root do not match"
        );

        // get vectors within the cluster
        let cluster = common::select_cluster(&self.database, &self.cluster_ids, cluster_id);

        // choose nearest vector
        let (_, result, cluster_root) = vectordb::chip_nearest_vector(&vector, &cluster);
        assert!(
            common::compare_fields(&self.cluster_roots[cluster_id], &cluster_root),
            "cluster root do not match"
        );

        result
    }
}
