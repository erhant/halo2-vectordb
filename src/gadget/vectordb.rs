use halo2_base::{
    gates::{GateInstructions, RangeInstructions},
    utils::ScalarField,
    AssignedValue, Context, QuantumCell,
};
use poseidon::PoseidonChip;
use std::fmt::Debug;

use super::fixed_point::{FixedPointChip, FixedPointInstructions};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VectorDBStrategy {
    Vertical,
}

#[derive(Clone, Debug)]
pub struct VectorDBChip<F: ScalarField, const PRECISION_BITS: u32> {
    strategy: VectorDBStrategy,
    pub fixed_point_gate: FixedPointChip<F, PRECISION_BITS>,
}

impl<F: ScalarField, const PRECISION_BITS: u32> VectorDBChip<F, PRECISION_BITS> {
    pub fn new(
        strategy: VectorDBStrategy,
        fixed_point_gate: FixedPointChip<F, PRECISION_BITS>,
    ) -> Self {
        Self { strategy, fixed_point_gate }
    }

    pub fn default(fixed_point_gate: FixedPointChip<F, PRECISION_BITS>) -> Self {
        Self::new(VectorDBStrategy::Vertical, fixed_point_gate)
    }

    /// Wrapper for `quantization` of the fixed-point chip.
    pub fn quantize(&self, x: f64) -> F {
        self.fixed_point_gate.quantization(x)
    }

    /// Wrapper for `dequantization` of the fixed-point chip.
    pub fn dequantize(&self, x: F) -> f64 {
        self.fixed_point_gate.dequantization(x)
    }

    /// Calls `quantize` on a vector of elements.
    pub fn quantize_vector(&self, a: &Vec<f64>) -> Vec<F> {
        a.iter().map(|a_i| self.fixed_point_gate.quantization(*a_i)).collect()
    }
}

pub trait VectorDBInstructions<F: ScalarField, const PRECISION_BITS: u32> {
    type FixedPointGate: FixedPointInstructions<F, PRECISION_BITS>;

    fn fixed_point_gate(&self) -> &Self::FixedPointGate;

    fn strategy(&self) -> VectorDBStrategy;

    /// Given a query vector, returns the most similar vector
    /// by doing an exhaustive search over all the given vectors
    /// and with respect to provided distance function
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
    ) -> Vec<AssignedValue<F>>
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
    ) -> (Vec<Vec<AssignedValue<F>>>, Vec<Vec<AssignedValue<F>>>)
    where
        F: ScalarField;
}

impl<F: ScalarField, const PRECISION_BITS: u32> VectorDBInstructions<F, PRECISION_BITS>
    for VectorDBChip<F, PRECISION_BITS>
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
        ) -> AssignedValue<F>, // TODO: ask for distance metric function here
    ) -> Vec<AssignedValue<F>>
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
            .reduce(|acc, d| self.fixed_point_gate().qmin(ctx, acc, d))
            .unwrap();
        let min_indicator: Vec<AssignedValue<F>> = distances
            .into_iter()
            .map(|d| self.fixed_point_gate().gate().is_equal(ctx, min, d))
            .collect();

        // get the most similar vector by selecting each index with indicator
        let result: Vec<AssignedValue<F>> = (0..vectors[0].len())
            .map(|i| {
                self.fixed_point_gate().gate().select_by_indicator(
                    ctx,
                    vectors.iter().map(|d| d[i]),
                    min_indicator.iter().copied(),
                )
            })
            .collect();

        result
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
                poseidon.squeeze(ctx, self.fixed_point_gate().gate()).unwrap()
            })
            .collect();

        // construct merklee tree from the hashes
        // TODO: make this extend with zeros for powers of two
        let mut leaves: Vec<AssignedValue<F>> = hashes;

        while leaves.len() > 1 {
            // assert that the number of leaves is a power of two
            assert!((leaves.len() & (leaves.len() - 1)) == 0);

            let mut next_leaves = Vec::with_capacity(leaves.len() / 2);
            for i in (0..leaves.len()).step_by(2) {
                poseidon.clear();
                poseidon.update(&[leaves[i], leaves[i + 1]]);
                next_leaves.push(poseidon.squeeze(ctx, self.fixed_point_gate().gate()).unwrap());
            }
            leaves = next_leaves;
        }
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
    ) -> (Vec<Vec<AssignedValue<F>>>, Vec<Vec<AssignedValue<F>>>)
    where
        F: ScalarField,
    {
        // choose initial centroids as the first `k` vectors
        let mut centroids: Vec<Vec<AssignedValue<F>>> =
            (0..K).map(|i| vectors[i].clone()).collect();

        // ones and zeros needed for indicators
        let one: AssignedValue<F> = ctx.load_constant(self.quantize(1.0));
        let zero: AssignedValue<F> = ctx.load_zero(); // quantized zero is equal to native zero

        let mut cluster_indicators: Vec<Vec<AssignedValue<F>>> = vec![];

        for _iter in 0..I {
            // assign each vector to closest centroid
            //
            // instead of assigning a cluster id to each vector,
            // we will store an indicator (one-hot encoding) for that cluster
            // suppose K = 4 and vectors A and B belong to 1, 3 respectively
            // we would have [1, 0, 0, 0] and [0, 0, 1, 0] as the indicators
            cluster_indicators = vectors
                .clone()
                .iter()
                .map(|v| {
                    // compute distance to centroids
                    let distances: Vec<AssignedValue<F>> =
                        centroids.iter().map(|c| distance(ctx, c, v)).collect();

                    // find the minimum
                    let min: AssignedValue<F> = distances
                        .clone()
                        .into_iter()
                        .reduce(|min, d| self.fixed_point_gate().qmin(ctx, min, d))
                        .unwrap();

                    // return indicator
                    let indicators: Vec<AssignedValue<F>> = distances
                        .into_iter()
                        .map(|d| {
                            // check if distance is the minimum
                            let eq = self.fixed_point_gate().gate().is_equal(ctx, min, d);

                            // return 1 if so, 0 otherwise
                            self.fixed_point_gate().gate().select(ctx, one, zero, eq)
                        })
                        .collect();

                    indicators
                })
                .collect();

            // index-wise summation of indicators will give the cluster sizes
            // this will be used to take the mean value after computing sum of
            // vectors within the cluster
            let cluster_sizes: Vec<AssignedValue<F>> = cluster_indicators
                .clone()
                .into_iter()
                .reduce(|sizes, indicators| {
                    // element-wise addition
                    sizes
                        .into_iter()
                        .zip(indicators)
                        .map(|(a, c)| self.fixed_point_gate().qadd(ctx, a, c))
                        .collect()
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
                        let is_zero = self.fixed_point_gate().gate().is_zero(ctx, sel);
                        vector
                            .into_iter()
                            .map(|v| self.fixed_point_gate().gate().select(ctx, zero, v, is_zero))
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
                            .map(|(s, v)| self.fixed_point_gate().qadd(ctx, s, v))
                            .collect()
                    })
                    // divide by cluster size
                    .map(|sum| {
                        sum.into_iter()
                            .map(|s| {
                                self.fixed_point_gate().qdiv(ctx, s, cluster_sizes[cluster_id])
                            })
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