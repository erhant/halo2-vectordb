# Notes

Some notes, references & helpers.

## Approach

We can do the following:

-   Work with floating point values alone, and use the fixed point arithemtic chip.
-   Implement a scalar quantization method other than the fixed point quantization, and apply it to all vectors. Then, commit to the database with the quantized vectors, in a single proof that both quantizes the vectors and commits to them. Set these vector quantization values as fixed columns for the circuit, and work your algorithm over those.

## Halo2

-   [Halo2 Scaffold](https://github.com/axiom-crypto/halo2-scaffold) is the template that this repository was created with, prepared by [Axiom](https://www.axiom.xyz/).
-   [Halo2 lib docs](https://docs.axiom.xyz/zero-knowledge-proofs/getting-started-with-halo2)
-   [Halo2 cheatsheet](https://hackmd.io/@axiom/HyoXzD7Zh)
-   [ZKFixedPointChip](https://github.com/DCMMC/ZKFixedPointChip) has a chip that provides fixed-point arithmetic & math.

## Vector Similarity

-   [Post by Labelbox](https://labelbox.com/blog/how-vector-similarity-search-works/)
-   [Post by Pinecone](https://www.pinecone.io/learn/vector-similarity/)
-   [LSH by Pinecone](https://www.pinecone.io/learn/series/faiss/locality-sensitive-hashing/)

## Space Partioning (IVF)

To reduce the number of vectors to compare, we may do space partitioning:

-   Compute a set of `C` centroids (clusters).
-   Compare query `q` to to each centroid, choose to most similar `c_i`.
-   Then, find the most similar vector `v` from the cluster `i`, return the most similar vector along with a Merkle root.

The query phase and indexing phase will be split.

-   During the query phase, verifier will compute the similar vector exhaustively and the circuit will return the proof: "the computed vector `v` is the most similar to `q` among the vectors with Merkle root `m`".

-   During the indexing phase, the db must provide several proofs regarding the merkle roots. In particular, given a set of vectors `V` it will compute a set of centroid vectors `c` and output the following:

    -   merkle root of `V`
    -   `k` for the k-means
    -   hash of centroid `c_0`
    -   merkle root of vectors belonging to cluster of `c_0`
    -   hash of centroid `c_1`
    -   merkle root of vectors belonging to cluster of `c_1`
    -   ...
    -   hash of centroid `c_k`
    -   merkle root of vectors belonging to cluster of `c_k`
