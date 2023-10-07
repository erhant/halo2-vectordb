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
    const USE_RANDOM_VECTOR: bool = false;
    const DIM: usize = 4;
    const NUM_VECS: usize = 4;

    #[test]
    fn test_scenario_native() {
        let vectors = common::random_vectors(DIM, NUM_VECS);
        let query = if USE_RANDOM_VECTOR { common::random_vector(DIM) } else { vectors[0].clone() };

        let db = demo::DemoDB::<2, 4>::new(vectors.clone());
        let result = db.ann(query.clone());

        // println!("DB:\n{:?}", vectors);
        // println!("QUERY:\n{:?}", query);
        // println!("RESULT:\n{:?}", result);
        common::assert_vectors_eq(&query, &result);
    }

    #[test]
    fn test_scenario_chip() {
        let vectors = common::random_vectors(DIM, NUM_VECS);
        let query = if USE_RANDOM_VECTOR { common::random_vector(DIM) } else { vectors[0].clone() };

        let db = demo::DemoZKDB::<2, 4>::new(vectors.clone());
        let result = db.ann(query.clone());

        common::assert_vectors_eq(&query, &result);
    }
}
