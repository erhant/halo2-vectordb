mod distances;

#[macro_use]
extern crate assert_float_eq;
use assert_float_eq::afe_is_relative_eq;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_euclidean_distance() {
        let a = vec![0.123, 0.456, 1.789];
        let b = vec![1.123, 0.456, 0.789];

        let dist_native = distances::euclidean_distance(&a, &b);
        let dist_chip = distances::chip_euclidean(&a, &b);
        assert_float_relative_eq!(dist_native, dist_chip);
    }

    #[test]
    fn test_manhattan_distance() {
        let a = vec![0.123, 0.456, 1.789];
        let b = vec![1.123, 0.456, 0.789];

        let dist_native = distances::manhattan_distance(&a, &b);
        let dist_chip = distances::chip_manhattan(&a, &b);
        assert_float_relative_eq!(dist_native, dist_chip);
    }

    #[test]
    fn test_cosine_distance() {
        let a = vec![0.123, 0.456, 1.789];
        let b = vec![1.123, 0.456, 0.789];

        let dist_native = distances::cosine_distance(&a, &b);
        let dist_chip = distances::chip_cosine(&a, &b);
        assert_float_relative_eq!(dist_native, dist_chip);
    }

    #[test]
    fn test_hamming_distance() {
        let a = vec![0.123, 0.456, 1.789];
        let b = vec![1.123, 0.456, 0.789];

        let dist_native = distances::hamming_distance(&a, &b);
        let dist_chip = distances::chip_hamming(&a, &b);
        assert_float_relative_eq!(dist_native, dist_chip);
    }
}
