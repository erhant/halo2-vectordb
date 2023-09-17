use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::read;
use std::io::Cursor;

// TODO handle errors
pub fn fetch_vectors(path: &str, dims: usize) -> Vec<f32> {
    let data_r = read(path).unwrap();
    let data_r_slice = data_r.as_slice();

    let num_vectors = data_r.len() / (4 + dims * 4);
    let mut data_w: Vec<f32> = Vec::with_capacity(num_vectors * dims);
    let mut reader = Cursor::new(data_r_slice);
    for _i in 0..num_vectors {
        // read dimension of the vector
        let dim = reader.read_u32::<LittleEndian>().unwrap();
        if dim != dims.try_into().unwrap() {
            panic!("dim mismatch while reading the source data");
        }

        // read vector data
        for _j in 0..dim {
            data_w.push(reader.read_f32::<LittleEndian>().unwrap());
        }
    }

    data_w
}

// fn main() {
//     let vecs = fetch_vectors("./res/siftsmall_query.fvecs", 128);
//     println!("{:?}", vecs.len());
// }
