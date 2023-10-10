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
    fn test_random_small() {
        const DIM: usize = 4;
        const NUM_VECS: usize = 4;
        const K: usize = 2;
        const I: usize = 4;

        let vectors = common::random_vectors(DIM, NUM_VECS);
        let query = common::random_vector(DIM);

        // native results
        let db_native = demo::DemoDB::<K, I>::new(vectors.clone());
        let result_native = db_native.ann(query.clone());

        // chip results
        let db_chip = demo::DemoZKDB::<K, I>::new(vectors);
        let result_chip = db_chip.ann(query.clone());

        // compare
        common::assert_vectors_eq(&result_chip, &result_native);
    }

    // this one might take some time
    #[test]
    fn test_random_large() {
        const DIM: usize = 128;
        const NUM_VECS: usize = 100;
        const K: usize = 10;
        const I: usize = 10;

        let vectors = common::random_vectors(DIM, NUM_VECS);
        let query = common::random_vector(DIM);

        // native results
        let db_native = demo::DemoDB::<K, I>::new(vectors.clone());
        let result_native = db_native.ann(query.clone());

        // chip results
        let db_chip = demo::DemoZKDB::<K, I>::new(vectors);
        let result_chip = db_chip.ann(query.clone());

        // compare
        common::assert_vectors_eq(&result_chip, &result_native);
    }

    // this one might take some time
    #[test]
    fn test_siftsmall() {
        // out of memory for K = 25 & I = 50 & N = 1000
        const K: usize = 2; // 100 as per the dataset description
        const I: usize = 1;
        const NUM_VECS: usize = 10;
        const DIM: usize = 128; // 128 as per the dataset description
        assert_eq!(DIM, 128, "siftsmall vectors have dimension 128");

        // read vectors from disk
        let query_vecs = common::read_vectors_from_disk("./res/siftsmall_query.fvecs", DIM);
        assert_eq!(query_vecs.len(), DIM * 100); // 100 query vectors
        let base_vecs = common::read_vectors_from_disk("./res/siftsmall_base.fvecs", DIM);
        assert_eq!(base_vecs.len(), DIM * 10000); // 10K base vectors

        // split into separate vectors
        let vectors = common::select_from_vectors(&base_vecs, DIM, &(0..NUM_VECS).collect());
        let query = common::select_from_vectors(&query_vecs, DIM, &vec![1])[0].clone();

        // native results
        let db_native = demo::DemoDB::<K, I>::new(vectors.clone());
        let result_native = db_native.ann(query.clone());

        // chip results
        let db_chip = demo::DemoZKDB::<K, I>::new(vectors);
        let result_chip = db_chip.ann(query.clone());

        // compare
        common::assert_vectors_eq(&result_chip, &result_native);
    }
}
