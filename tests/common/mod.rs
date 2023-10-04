use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::read;
use std::io::Cursor;

use assert_float_eq::assert_float_relative_eq;

/// Compare `f64` elements of two vectors with relative error.
///
/// Will fail the test if at least one non-matching element is found.
pub fn compare_vectors(a: &Vec<f64>, b: &Vec<f64>) {
    assert_eq!(a.len(), b.len());
    a.iter().zip(b).for_each(|(a, b)| assert_float_relative_eq!(*a, *b))
}

/// Compare `f64` elements of two sets of vectors with relative error.
///
/// Will fail the test if at least one non-matching element is found within a vector.
pub fn compare_set_of_vectors(a: &Vec<Vec<f64>>, b: &Vec<Vec<f64>>) {
    assert_eq!(a.len(), b.len());
    a.iter().zip(b).for_each(|(a, b)| compare_vectors(a, b))
}

/// Generate a random vector with `dim` elements.
pub fn random_vector(dim: usize) -> Vec<f64> {
    let mut vector: Vec<f64> = Vec::with_capacity(dim);
    for _ in 0..dim {
        vector.push(rand::random::<f64>())
    }
    vector
}

/// Generate `n` random vectors with `dim` elements.
pub fn random_vectors(dim: usize, n: usize) -> Vec<Vec<f64>> {
    let mut vectors: Vec<Vec<f64>> = Vec::with_capacity(n);
    for _ in 0..n {
        vectors.push(random_vector(dim))
    }
    vectors
}

// TODO handle errors
// fn main() {
//     let vecs = fetch_vectors("./res/siftsmall_query.fvecs", 128);
//     println!("{:?}", vecs.len());
// }
pub fn fetch_vectors(path: &str, dims: usize) -> Vec<f32> {
    let data_r = read(path).unwrap();
    let data_r_slice = data_r.as_slice();

    let num_vectors = data_r.len() / (4 + dims * 4);
    let mut data_w: Vec<f32> = Vec::with_capacity(num_vectors * dims);

    let mut reader = Cursor::new(data_r_slice);
    for _i in 0..num_vectors {
        // read dimension of the vector
        let dim = reader.read_u32::<LittleEndian>().unwrap();
        assert_eq!(dim, dims as u32, "unexpected vector dimension");

        // read vector data
        for _j in 0..dim {
            data_w.push(reader.read_f32::<LittleEndian>().unwrap());
        }
    }

    data_w
}
