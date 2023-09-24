use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::read;
use std::io::Cursor;

use assert_float_eq::afe_is_relative_eq;

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
        if dim != dims as u32 {
            panic!("dim mismatch while reading the source data");
        }

        // read vector data
        for _j in 0..dim {
            data_w.push(reader.read_f32::<LittleEndian>().unwrap());
        }
    }

    data_w
}

/// Compare `f64` elements of two vectors with relative error.
///
/// Will fail the test if at least one non-matching element is found.
pub fn compare_vectors(a: &Vec<f64>, b: &Vec<f64>) {
    a.iter().zip(b).for_each(|(a, b)| assert_float_relative_eq!(*a, *b))
}

/// Generate a random vector with `dim` elements.
pub fn random_vector(dim: usize) -> Vec<f64> {
    vec![]
}

/// Generate `n` random vectors with `dim` elements.
pub fn random_vectors(dim: usize, n: usize) -> Vec<Vec<f64>> {
    vec![]
}
