use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use std::fs::read;
use std::io::Cursor;
use std::num::TryFromIntError;
use std::path::{Path, PathBuf};

// TODO handle errors
fn fetch_vectors(filename: &str, dims: usize) -> Result<Vec<f32>, TryFromIntError> {
    // let path = "./".to_owned() + filename;
    let data_r = read("./file.fvec")?;
    let data_r_slice = data_r.as_slice();

    let num_vectors = data_r.len() / (4 + dims * 4);
    let mut data_w: Vec<f32> = Vec::with_capacity(num_vectors * dims);
    let mut reader = Cursor::new(data_r_slice);
    for _i in 0..num_vectors {
        // read dimension of the vector
        let dim = reader.read_u32::<LittleEndian>().unwrap();
        if dim != dims.try_into()? {
            panic!("dim mismatch while reading the source data");
        }

        // read vector data
        for _j in 0..dim {
            data_w.push(reader.read_f32::<LittleEndian>().unwrap());
        }
    }
    Ok(data_w)
}

fn main() {
    unimplemented!();
}
