#[macro_use]
extern crate assert_float_eq;
use assert_float_eq::afe_is_relative_eq;

mod common;
mod distances;

#[cfg(test)]
mod test {
    use super::*;

    const DIM: usize = 10;

    #[test]
    fn test_euclidean_distance() {
        let a = common::random_vector(DIM);
        let b = common::random_vector(DIM);

        let dist_native = distances::euclidean_distance(&a, &b);
        let dist_chip = distances::chip_euclidean(&a, &b);
        assert_float_relative_eq!(dist_native, dist_chip);
    }

    #[test]
    fn test_manhattan_distance() {
        let a = common::random_vector(DIM);
        let b = common::random_vector(DIM);

        let dist_native = distances::manhattan_distance(&a, &b);
        let dist_chip = distances::chip_manhattan(&a, &b);
        assert_float_relative_eq!(dist_native, dist_chip);
    }

    #[test]
    fn test_cosine_distance() {
        let a = common::random_vector(DIM);
        let b = common::random_vector(DIM);

        let dist_native = distances::cosine_distance(&a, &b);
        let dist_chip = distances::chip_cosine(&a, &b);
        assert_float_relative_eq!(dist_native, dist_chip);
    }

    #[test]
    fn test_hamming_distance() {
        let a = common::random_vector(DIM);
        let b = common::random_vector(DIM);

        let dist_native = distances::hamming_distance(&a, &b);
        let dist_chip = distances::chip_hamming(&a, &b);
        assert_float_relative_eq!(dist_native, dist_chip);
    }
}
