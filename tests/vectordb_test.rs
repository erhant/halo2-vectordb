#[macro_use]
extern crate assert_float_eq;

mod common;
mod distances;
mod vectordb;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_kmeans_small() {
        // the following page does the same init strategy,
        // i.e. takes first k vectors as the initial centroids.
        // we can therefore compare our results to there
        // https://people.revoledu.com/kardi/tutorial/kMean/Online-K-Means-Clustering.html

        const K: usize = 2;
        const I: usize = 4;
        const DIM: usize = 5;
        let vectors = common::random_vectors(DIM, 30);

        let (centroids_native, clusterids_native) =
            vectordb::kmeans::<K, I>(&vectors, &distances::euclidean_distance);
        let (centroids_chip, clusterids_chip) = vectordb::chip_kmeans::<K, I>(&vectors);
        common::assert_multiple_vectors_eq(&centroids_native.to_vec(), &centroids_chip.to_vec());
        assert_eq!(clusterids_native, clusterids_chip);
    }

    #[test]
    fn test_nearest_vector() {
        const DIM: usize = 4;
        let query = common::random_vector(DIM);
        let vectors = common::random_vectors(DIM, 4);

        let (idx_native, result_native) =
            vectordb::nearest_vector(&query, &vectors, &distances::euclidean_distance);
        let (idx_chip, result_chip, _) = vectordb::chip_nearest_vector(&query, &vectors);
        common::assert_vectors_eq(&result_native, &result_chip);
        assert_eq!(idx_native, idx_chip);
    }
}
