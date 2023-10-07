#![allow(dead_code)]

use byteorder::{LittleEndian, ReadBytesExt};
use halo2_base::utils::ScalarField;
use std::cmp::Ordering;
use std::fs::read;
use std::io::Cursor;

use assert_float_eq::assert_float_relative_eq;

/// Compare `f64` elements of two vectors with relative error.
///
/// Will fail the test if at least one non-matching element is found.
pub fn assert_vectors_eq(a: &Vec<f64>, b: &Vec<f64>) {
    assert_eq!(a.len(), b.len());
    a.iter().zip(b).for_each(|(a, b)| assert_float_relative_eq!(*a, *b))
}

/// Compare `f64` elements of two sets of vectors with relative error.
///
/// Will fail the test if at least one non-matching element is found within a vector.
pub fn assert_multiple_vectors_eq(a: &Vec<Vec<f64>>, b: &Vec<Vec<f64>>) {
    assert_eq!(a.len(), b.len());
    a.iter().zip(b).for_each(|(a, b)| assert_vectors_eq(a, b))
}

/// Compare two field elements, returns true if they are equal.
pub fn compare_fields<F: ScalarField>(p: &F, q: &F) -> bool {
    p.cmp(q) == Ordering::Equal
}

/// Generate a random vector with `dim` elements.
pub fn random_vector(dim: usize) -> Vec<f64> {
    let mut vector: Vec<f64> = Vec::with_capacity(dim);
    for _ in 0..dim {
        vector.push(rand::random::<f64>())
    }
    vector
}

/// Generate a random vector with integers in range `[0, max)`.
pub fn random_indices(dim: usize, max: usize) -> Vec<usize> {
    let mut vector: Vec<usize> = Vec::with_capacity(dim);
    for _ in 0..dim {
        vector.push(rand::random::<usize>() % max)
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

/// From a set of vectors and their cluster ids, select
/// the vectors within that cluster id. For example:
///
/// ```rs
/// vectors     = [a, b, c, d, e]
/// cluster_ids = [0, 1, 0, 2, 1]
/// cluster_id  = 1
/// // returns:
/// [c, e]
/// ```
pub fn select_cluster(
    vectors: &Vec<Vec<f64>>,
    cluster_ids: &Vec<usize>,
    cluster_id: usize,
) -> Vec<Vec<f64>> {
    assert_eq!(vectors.len(), cluster_ids.len(), "vectors & cluster ids do not  match lengths");

    (0..cluster_ids.len())
        .filter(|i| cluster_ids[*i] == cluster_id)
        .map(|i| vectors[i].clone())
        .collect()
}

/// Read vectors from disk. We mainly use this for the dataset downloaded
/// from [this link](http://corpus-texmex.irisa.fr/).
///
/// For example:
/// ```rs
/// let vecs = read_vectors_from_disk("./res/siftsmall_query.fvecs", 128);
/// println!("{:?}", vecs.len());
/// ```
pub fn read_vectors_from_disk(path: &str, dims: usize) -> Vec<f32> {
    let data_r = read(path).expect("expected file at path");
    let data_r_slice = data_r.as_slice();

    let num_vectors = data_r.len() / (4 + dims * 4);
    let mut data_w: Vec<f32> = Vec::with_capacity(num_vectors * dims);

    let mut reader = Cursor::new(data_r_slice);
    for _i in 0..num_vectors {
        // read dimension of the vector
        let dim = reader.read_u32::<LittleEndian>().expect("could not read dimension");
        assert_eq!(dim, dims as u32, "unexpected vector dimension");

        // read vector data
        for _j in 0..dim {
            data_w.push(reader.read_f32::<LittleEndian>().expect("could not read vector data"));
        }
    }

    data_w
}

/// Given a single vector that has all vectors in it (i.e. the result of
/// `read_vectors_from_disk`) selects few vectors and returns them.
///
/// The `select` array is an array of indices that correspond to vector indices.
pub fn select_from_vectors(vectors: &Vec<f32>, dims: usize, select: &Vec<usize>) -> Vec<Vec<f64>> {
    // ensure that `vectors` has N vectors, each of `dims` dimension
    assert_eq!(vectors.len() % dims, 0, "dimension is wrong");
    let num_vecs = vectors.len() / dims;

    assert!(select.iter().all(|s| *s < num_vecs), "one of the indices is out-of-bounds");

    let result = select
        .iter()
        .map(|s| {
            let mut selected: Vec<f64> = Vec::with_capacity(dims);
            for i in *s..(*s + dims) {
                selected.push(vectors[i] as f64);
            }
            selected
        })
        .collect();

    result
}
