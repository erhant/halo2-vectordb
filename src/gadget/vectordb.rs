use halo2_base::{gates::GateInstructions, utils::ScalarField, AssignedValue, Context};
use poseidon::PoseidonChip;
use std::fmt::Debug;

use super::fixed_point::{FixedPointChip, FixedPointInstructions};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VectorDBStrategy {
    Vertical,
}

#[derive(Clone, Debug)]
pub struct VectorDBChip<'a, F: ScalarField, const PRECISION_BITS: u32> {
    strategy: VectorDBStrategy,
    pub fixed_point_gate: &'a FixedPointChip<F, PRECISION_BITS>,
}

impl<'a, F: ScalarField, const PRECISION_BITS: u32> VectorDBChip<'a, F, PRECISION_BITS> {
    pub fn new(
        strategy: VectorDBStrategy,
        fixed_point_gate: &'a FixedPointChip<F, PRECISION_BITS>,
    ) -> Self {
        Self { strategy, fixed_point_gate }
    }

    pub fn default(fixed_point_gate: &'a FixedPointChip<F, PRECISION_BITS>) -> Self {
        Self::new(VectorDBStrategy::Vertical, fixed_point_gate)
    }
}

pub trait VectorDBInstructions<F: ScalarField, const PRECISION_BITS: u32> {
    type FixedPointGate: FixedPointInstructions<F, PRECISION_BITS>;

    fn fixed_point_gate(&self) -> &Self::FixedPointGate;

    fn strategy(&self) -> VectorDBStrategy;

    /// Given a query vector, returns the most similar vector
    /// by doing an exhaustive search over all the given vectors
    /// and with respect to provided distance function.
    ///
    /// Also returns the indicator, that is an array of 0s and 1s where
    /// only one index has 1, indicating the vector index within the vectors.
    fn nearest_vector(
        &self,
        ctx: &mut Context<F>,
        query: &Vec<AssignedValue<F>>,
        vectors: &Vec<Vec<AssignedValue<F>>>,
        distance: &dyn Fn(
            &mut Context<F>,
            &Vec<AssignedValue<F>>,
            &Vec<AssignedValue<F>>,
        ) -> AssignedValue<F>,
    ) -> (Vec<AssignedValue<F>>, Vec<AssignedValue<F>>)
    where
        F: ScalarField;

    /// Commits to an array of vectors.
    fn merkle_commitment<const T: usize, const RATE: usize>(
        &self,
        ctx: &mut Context<F>,
        poseidon: &mut PoseidonChip<F, T, RATE>,
        vectors: &Vec<Vec<AssignedValue<F>>>,
    ) -> AssignedValue<F>
    where
        F: ScalarField;

    /// K-means algorithm to compute `K` centroids from a given set of vectors.
    /// Since the algorithm can't stop execution based on convergence, we instead
    /// opt for a fixed-iteration approach.
    ///
    /// - K: number of centroids
    /// - I: number of iterations
    ///
    /// Returns the centroids and cluster indicators for each vector (one-hot encoded).
    fn kmeans<const K: usize, const I: usize>(
        &self,
        ctx: &mut Context<F>,
        vectors: &Vec<Vec<AssignedValue<F>>>,
        distance: &dyn Fn(
            &mut Context<F>,
            &Vec<AssignedValue<F>>,
            &Vec<AssignedValue<F>>,
        ) -> AssignedValue<F>,
    ) -> ([Vec<AssignedValue<F>>; K], Vec<[AssignedValue<F>; K]>)
    where
        F: ScalarField;
}

impl<'a, F: ScalarField, const PRECISION_BITS: u32> VectorDBInstructions<F, PRECISION_BITS>
    for VectorDBChip<'a, F, PRECISION_BITS>
{
    type FixedPointGate = FixedPointChip<F, PRECISION_BITS>;

    fn fixed_point_gate(&self) -> &Self::FixedPointGate {
        &self.fixed_point_gate
    }

    fn strategy(&self) -> VectorDBStrategy {
        self.strategy
    }

    fn nearest_vector(
        &self,
        ctx: &mut Context<F>,
        query: &Vec<AssignedValue<F>>,
        vectors: &Vec<Vec<AssignedValue<F>>>,
        distance: &dyn Fn(
            &mut Context<F>,
            &Vec<AssignedValue<F>>,
            &Vec<AssignedValue<F>>,
        ) -> AssignedValue<F>,
    ) -> (Vec<AssignedValue<F>>, Vec<AssignedValue<F>>)
    where
        F: ScalarField,
    {
        // compute distance to each vector
        let distances: Vec<AssignedValue<F>> =
            vectors.iter().map(|v| distance(ctx, v, query)).collect();

        // find the minimum
        let min: AssignedValue<F> = distances
            .clone()
            .into_iter()
            .reduce(|acc, d| self.fixed_point_gate.qmin(ctx, acc, d))
            .unwrap();
        let min_indicator: Vec<AssignedValue<F>> = distances
            .into_iter()
            .map(|d| self.fixed_point_gate.gate().is_equal(ctx, min, d))
            .collect();

        // get the most similar vector by selecting each index with indicator
        let result: Vec<AssignedValue<F>> = (0..vectors[0].len())
            .map(|i| {
                self.fixed_point_gate.gate().select_by_indicator(
                    ctx,
                    vectors.iter().map(|d| d[i]),
                    min_indicator.iter().copied(),
                )
            })
            .collect();

        (min_indicator, result)
    }

    fn merkle_commitment<const T: usize, const RATE: usize>(
        &self,
        ctx: &mut Context<F>,
        poseidon: &mut PoseidonChip<F, T, RATE>,
        vectors: &Vec<Vec<AssignedValue<F>>>,
    ) -> AssignedValue<F>
    where
        F: ScalarField,
    {
        // hash each vector to a field element
        // this is okay to do because we dont care about elements of vectors
        // we just want to commit to an entire vector, or none at all
        let hashes: Vec<AssignedValue<F>> = vectors
            .iter()
            .map(|v| {
                poseidon.clear();
                poseidon.update(&v.as_slice());
                poseidon.squeeze(ctx, self.fixed_point_gate.gate()).unwrap()
            })
            .collect();

        // extend leaves with zeros to ensure number of leaves is a power of two
        let num_hashes = hashes.len();
        let num_leaves: usize = if (num_hashes & (num_hashes - 1)) == 0 {
            num_hashes
        } else {
            let mut next_pow_of_two = 1 as usize;
            while next_pow_of_two < num_hashes {
                next_pow_of_two <<= 1;
            }
            next_pow_of_two
        };
        assert!(num_hashes <= num_leaves, "expected #hashes to be less than computed #leaves");
        let num_zeros = num_leaves - num_hashes;

        // construct merklee tree from the hashes & zeros
        let mut leaves: Vec<AssignedValue<F>> = hashes;
        if num_zeros > 0 {
            leaves.extend(vec![ctx.load_zero(); num_zeros])
        }
        assert_eq!(leaves.len(), num_leaves, "expected #leaves many leaves");

        while leaves.len() > 1 {
            // assert that the number of leaves is always a power of two
            assert!((leaves.len() & (leaves.len() - 1)) == 0);

            let mut next_leaves = Vec::with_capacity(leaves.len() / 2);
            for i in (0..leaves.len()).step_by(2) {
                poseidon.clear();
                poseidon.update(&[leaves[i], leaves[i + 1]]);
                next_leaves.push(poseidon.squeeze(ctx, self.fixed_point_gate.gate()).unwrap());
            }
            leaves = next_leaves;
        }
        // assert that we have reached root
        assert!(leaves.len() == 1);

        leaves[0]
    }

    fn kmeans<const K: usize, const I: usize>(
        &self,
        ctx: &mut Context<F>,
        vectors: &Vec<Vec<AssignedValue<F>>>,
        distance: &dyn Fn(
            &mut Context<F>,
            &Vec<AssignedValue<F>>,
            &Vec<AssignedValue<F>>,
        ) -> AssignedValue<F>,
    ) -> ([Vec<AssignedValue<F>>; K], Vec<[AssignedValue<F>; K]>)
    where
        F: ScalarField,
    {
        // ones and zeros needed for indicators
        let one: AssignedValue<F> = ctx.load_constant(self.fixed_point_gate.quantization(1.0));
        let zero: AssignedValue<F> = ctx.load_zero(); // quantized zero is equal to native zero

        // take first K vectors as the initial centroids
        let mut centroids: [Vec<AssignedValue<F>>; K] = vectors
            .iter()
            .take(K)
            .cloned()
            .collect::<Vec<Vec<AssignedValue<F>>>>()
            .try_into()
            .unwrap();

        let mut cluster_indicators: Vec<[AssignedValue<F>; K]> = vec![];

        for _iter in 0..I {
            // assign each vector to closest centroid
            //
            // instead of assigning a cluster id to each vector,
            // we will store an indicator (one-hot encoding) for that cluster
            // suppose K = 4 and vectors A and B belong to 1, 3 respectively
            // we would have [0, 1, 0, 0] and [0, 0, 0, 1] as the indicators.
            cluster_indicators = vectors
                .iter()
                .map(|v| {
                    // compute distance to centroids
                    let distances: [AssignedValue<F>; K] =
                        centroids.clone().map(|c| distance(ctx, &c, v));
                    // it works when i assign `[one; K];` instead

                    // find the minimum
                    let min: AssignedValue<F> = distances
                        .clone()
                        .into_iter()
                        .reduce(|min, d| self.fixed_point_gate.qmin(ctx, min, d))
                        .unwrap();

                    // return indicator
                    let indicators: [AssignedValue<F>; K] = distances.map(|d| {
                        // check if distance is the minimum
                        let eq = self.fixed_point_gate.gate().is_equal(ctx, min, d);

                        // return 1 if so, 0 otherwise
                        self.fixed_point_gate.gate().select(ctx, one, zero, eq)
                    });

                    indicators
                })
                .collect();

            // index-wise summation of indicators will give the cluster sizes
            // this will be used to take the mean value after computing sum of
            // vectors within the cluster
            let cluster_sizes: [AssignedValue<F>; K] = cluster_indicators
                .clone()
                .into_iter()
                .reduce(|sizes, indicators| {
                    // element-wise addition
                    sizes
                        .zip(indicators)
                        .map(|(size, ind)| self.fixed_point_gate.qadd(ctx, size, ind))
                })
                .unwrap();

            // update centroids by finding the mean vector in each cluster
            for cluster_id in 0..K {
                // the index of indicators for this cluster indicates whether a vector
                // belongs to that cluster or not
                let is_in_cluster: Vec<AssignedValue<F>> =
                    cluster_indicators.iter().map(|indicators| indicators[cluster_id]).collect();

                // multiply each element of all vectors with the indicator
                // that represent the current cluster
                let filtered_vectors: Vec<Vec<AssignedValue<F>>> = vectors
                    .clone()
                    .into_iter()
                    .zip(is_in_cluster)
                    .map(|(vector, sel)| {
                        // multiply each element of the vector by the current cluster indicator
                        // use select instead of multiplication here, because we would have to do
                        // a quantized multiplication instead, which is costly.
                        //
                        // note that select itself is either a zero or a quantized one, which is
                        // not boolean! so, we have to compare it to zero and then use that
                        // equality as the selector
                        //
                        // we will add these values later, and that is alright because `v` is
                        // already quantized, and a field 0 is equal to a quantized 0
                        // (note that a quantized 1 is not a field 1)

                        let is_zero = self.fixed_point_gate.gate().is_zero(ctx, sel);
                        vector
                            .into_iter()
                            .map(|v| self.fixed_point_gate.gate().select(ctx, zero, v, is_zero))
                            .collect()
                    })
                    .collect();

                // mean of vectors in this cluster
                let mean: Vec<AssignedValue<F>> = filtered_vectors
                    .into_iter()
                    // sum everything
                    .reduce(|sum, vector| {
                        vector
                            .into_iter()
                            .zip(sum)
                            .map(|(s, v)| self.fixed_point_gate.qadd(ctx, s, v))
                            .collect()
                    })
                    // divide by cluster size
                    .map(|sum| {
                        sum.into_iter()
                            .map(|s| self.fixed_point_gate.qdiv(ctx, s, cluster_sizes[cluster_id]))
                            .collect()
                    })
                    .unwrap();

                // update centroid
                centroids[cluster_id] = mean;
            }
        }

        (centroids, cluster_indicators)
    }
}
