mod common;
mod distances;
mod vectordb;

#[macro_use]
extern crate assert_float_eq;

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
        let vectors = vec![vec![1.0, 1.0], vec![2.0, 1.0], vec![4.0, 3.0], vec![5.0, 4.0]];

        // fixed iterations since we have to do it that way in our circuit
        let results = vectordb::kmeans::<K, I>(&vectors, &distances::euclidean_distance);

        assert_eq!(results.0, [vec![1.5, 1.0], vec![4.5, 3.5]]); // centroids
        assert_eq!(results.1, [0, 0, 1, 1]); // cluster ids

        let chip_results = vectordb::chip_kmeans::<K, I>(&vectors);
        println!("{:?}", chip_results.0);
        println!("{:?}", chip_results.1);
        assert_eq!(results.0, chip_results.0); // centroids
        assert_eq!(results.1, chip_results.1); // cluster ids
    }

    #[test]
    fn test_nearest_vector() {
        let query = vec![0.123, 0.456, 1.789];
        let vectors = vec![
            vec![1.123, 0.456, 0.789],
            vec![1.111, 0.111, 0.111],
            vec![0.111, 0.444, 1.777],
            vec![8.890, 4.456, 2.234],
        ];

        let result_native =
            vectordb::nearest_vector(&query, &vectors, &distances::euclidean_distance);
        let result_chip = vectordb::chip_nearest_vector(&query, &vectors);
        common::compare_vectors(&result_native, &result_chip);
    }
}
