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
