<p align="center">
  <h1 align="center">
    Halo2 VectorDB
  </h1>
  <p align="center">
    <i>Verifiable vector similarity queries over a committed vector database.</i>
  </p>
</p>

This projects aims to obtain a proof-of-concept for a verifiable vector database using zero-knowledge proofs. We make heavy use of the awesome [ZKFixedPointChip](https://github.com/DCMMC/ZKFixedPointChip) which enables fixed-point arithmetic with [halo2-lib](https://github.com/axiom-crypto/halo2-lib).

## Installation

You need Rust installed:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then, you can clone this repository and use the chips inside it:

```sh
git clone https://github.com/erhant/halo2-vectordb.git
cd halo2-vectordb
```

## Usage

We implement two chips, one for distance metrics in halo2, and the other for basic vector database operations.

### [`DistanceChip`](./src/gadget/distance.rs)

`DistanceChip` provides distance metrics that operate on two vectors of equal length. The vector elements are expected to be quantized with the `FixedPointChip`. The following distance metrics are implemented:

- `euclidean_distance` computes the Euclidean distance between two vectors.
- `manhattan_distance` computes the Manhattan distance between two vectors.
- `hamming_distance` computes one minus Hamming similarity between two vectors.
- `cosine_distance` computes one minus Cosine similarity between two vectors.

### [`VectorDBChip`](./src/gadget/vectordb.rs)

`VectorDBChip` implements basic vector database functionality over a set of vectors. Similar to `DistanceChip`, it requires a `FixedPointChip` to operate over quantized values. It exposes the following functions:

- `nearest_vector` takes a set of vectors and a query vector, and finds the vector that is most similar to the query w.r.t. a given distance function. It also returns an indicator (i.e. one-hot encoded vector that indicates the index of the result vector) which may be used at later steps.
- `merkle_commitment` takes a set of vectors, and commits to them using a Merkle tree with Poseidon hashes. If the given set does not include power-of-two many elements, it will pad zeros to the remaining leaves. In our scenario, we only need the entire vector or none at all, and for that reason we do not care about committing to elements within the vector. As such, we first hash the entire vector, and then treat that hash as the leaf node.
- `kmeans` takes a set of vectors, a `K` constant to determine the number of centroids and an `I` constant to determine the number of iterations. K-means usually is an iterative algorithm that terminates when the centroids are no more updated; however, such a control-flow is not possible in a zk-circuit. Therefore, the `I` parameter determines a fixed number of iterations.

We also have a trait `FixedPointVectorInstructions` and its implementation for the `FixedPointChip`, which are simple utility functions to quantize and dequantize vectors.

### Demonstration

A demonstrative test suite can be found at [`demo_test`](./tests/demo_test.rs):

- First, we train a database without any ZK circuits involved, resulting in `K` centroids & clusters.
- Then, we train another database with Halo2 circuits, resulting in `K` centroids & clusters along with `K+2` merkle roots.
- A random query vector is given to both databases, and results are compared element-wise with relative error. Note that fixed-point precision
  in not as accurate as the `f64` Rust implementation.
- The merkle roots are compared between indexing and query phases, and are expected to match.

## Examples

Run the examples via one of the following:

```sh
# demonstrate distance computations
LOOKUP_BITS=12 cargo run --example distances -- \
  --name distances -k 13 mock

# example merkle commitment to vectors
LOOKUP_BITS=12 cargo run --example merkle -- \
  --name merkle -k 13 mock

# exhaustively find the similar vector & commit to the database
LOOKUP_BITS=12 cargo run --example query -- \
  --name query -k 13 mock

# compute centroids
LOOKUP_BITS=15 cargo run --example kmeans -- \
  --name kmeans -k 16 mock
```

<!-- LOOKUP_BITS=12 cargo run --example euclid -- --name euclid -k 13 mock -->

You can provide a specific input via the `--input <input-name>` option.

## Testing

To run tests:

```sh
cargo test
```

Some of the tests make use of the `ANN_SIFT_10K` dataset by [Jégou et al.](https://inria.hal.science/inria-00514462/en) which can be downloaded at [Corpus-Texmex](http://corpus-texmex.irisa.fr/). This dataset 128-dimensional vectors. Within our tests folder, the `common` module exposes two functions to read these vectors:

- `read_vectors_from_disk` takes a path and dimension (128 in this case) and reads the `.fvec` files from the dataset, returning a single vector of `f32` values. This single vector is composed of all vector elements, composed together.
- `select_from_vectors` takes that single vector and a list of indices, and returns the selected vectors only.

## Acknowledgements

- This project is developed as part of [Axiom Open Source V2](https://www.axiom.xyz/open-source-v2) program.
- This project would not be possible without the [ZKFixedPointChip](https://github.com/DCMMC/ZKFixedPointChip).
