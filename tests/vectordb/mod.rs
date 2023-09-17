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
