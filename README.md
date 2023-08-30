<p align="center">
  <h1 align="center">
    Halo2 VectorDB
  </h1>
  <p align="center">
    <i>Verifiable vector similarity queries over a committed vector database.</i>
  </p>
</p>

We are given a set of embedding vectors. These vectors are typically composed of floating point values, which is an issue for our arithmetic circuits that operate over integers modulo some large prime. Thankfully, an awesome work by Wentao Xiao enables fixed-point arithmetic in Halo2: [ZKFixedPointChip](https://github.com/DCMMC/ZKFixedPointChip). We will make heavy use of this chip.

After this, we have several vectors in the database along with a query vector. We would like the circuit to find a vector in the database that is similar to our query vector. This task has two aspects:

-   The vector similarity algorithms should be verifiable, i.e. we need to implement chips for them.
-   The database should be committed to, ensuring that the verifiable similarity algorithm has been used on all vectors.

### Distance Metrics

We provide a `SimilarityChip` that operate on two vectors $a, b$ of length $n$, and exposes the following similarity functions:

-   **Cosine Similarity**: $(a \cdot b) / \left(||a|| \cdot ||b||\right)$
-   **Manhattan Similarity**: $||a-b||_{1}$
-   **Euclidean Similarity**: $||a-b||_{2}$
-   **Dot-Product Similarity**: $a \cdot b$
-   **Hamming Similarity**: $\frac{1}{n}\sum_{i = 1}^{n} [a_i = b_i]$

### Committing to a Database

TODO: merkle the entire thing? treat vectors as polys and commit to them (e.g. KZG?)

## Usage

Run the examples via one of the following:

```sh
LOOKUP_BITS=12 cargo run --example similarities -- --name similarities --input vec4 -k 13 mock
```

You can provide a specific input via the `--input <input-name>` option.

## Testing

We plan on testing our implementations over vectors from `ANN_SIFT_10K` by [Jégou et al.](https://inria.hal.science/inria-00514462/en) from [Corpus-Texmex](http://corpus-texmex.irisa.fr/), which is composed of 10K 128-dimensional vectors. We have downloaded the dataset and store it under `res` folder.

## Acknowledgements

The project is developed as part of [Axiom Open Source V2](https://www.axiom.xyz/open-source-v2) program.
