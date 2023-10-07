const LOOKUP_BITS: usize = 13;
const PRECISION_BITS: u32 = 48;

#[macro_use]
extern crate assert_float_eq;

mod common;
mod demo;
mod distances;
mod vectordb;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_demo_small() {
        const DIM: usize = 4;
        const NUM_VECS: usize = 4;
        const K: usize = 2;
        const I: usize = 4;

        let vectors = common::random_vectors(DIM, NUM_VECS);
        // let query = common::random_vector(DIM);
        let query = vectors[0].clone();

        // native results
        let db_native = demo::DemoDB::<K, I>::new(vectors.clone());
        let result_native = db_native.ann(query.clone());
        common::assert_vectors_eq(&query, &result_native);

        // chip results
        let db_chip = demo::DemoZKDB::<K, I>::new(vectors);
        let result_chip = db_chip.ann(query.clone());
        common::assert_vectors_eq(&query, &result_chip);
    }

    #[test]
    fn test_demo_real() {
        let vectors = common::random_vectors(DIM, NUM_VECS);
        let query = if USE_RANDOM_VECTOR { common::random_vector(DIM) } else { vectors[0].clone() };
        const K: usize = 2;
        const I: usize = 4;

        // native results
        let db_native = demo::DemoDB::<K, I>::new(vectors.clone());
        let result_native = db_native.ann(query.clone());
        common::assert_vectors_eq(&query, &result_native);

        // chip results
        let db_chip = demo::DemoZKDB::<K, I>::new(vectors);
        let result_chip = db_chip.ann(query.clone());
        common::assert_vectors_eq(&query, &result_chip);
    }
}
