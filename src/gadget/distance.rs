use halo2_base::{
    gates::{GateInstructions, RangeInstructions},
    utils::ScalarField,
    AssignedValue, Context,
};
use std::fmt::Debug;

use super::fixed_point::{FixedPointChip, FixedPointInstructions};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DistanceStrategy {
    Vertical,
}

#[derive(Clone, Debug)]
pub struct DistanceChip<F: ScalarField, const PRECISION_BITS: u32> {
    strategy: DistanceStrategy,
    pub fixed_point_gate: FixedPointChip<F, PRECISION_BITS>,
}

impl<F: ScalarField, const PRECISION_BITS: u32> DistanceChip<F, PRECISION_BITS> {
    pub fn new(
        strategy: DistanceStrategy,
        fixed_point_gate: FixedPointChip<F, PRECISION_BITS>,
    ) -> Self {
        Self { strategy, fixed_point_gate }
    }

    pub fn default(fixed_point_gate: FixedPointChip<F, PRECISION_BITS>) -> Self {
        Self::new(DistanceStrategy::Vertical, fixed_point_gate)
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

pub trait DistanceInstructions<F: ScalarField, const PRECISION_BITS: u32> {
    type FixedPointGate: FixedPointInstructions<F, PRECISION_BITS>;

    fn fixed_point_gate(&self) -> &Self::FixedPointGate;

    fn strategy(&self) -> DistanceStrategy;

    /// Computes the hamming distance of two quantized vectors.
    /// This is equal to (1 - HammingDistance).
    fn hamming_distance(
        &self,
        ctx: &mut Context<F>,
        a: &Vec<AssignedValue<F>>,
        b: &Vec<AssignedValue<F>>,
    ) -> AssignedValue<F>
    where
        F: ScalarField;

    /// Computes the Manhattan distance (L1) of two quantized vectors.
    fn manhattan_distance(
        &self,
        ctx: &mut Context<F>,
        a: &Vec<AssignedValue<F>>,
        b: &Vec<AssignedValue<F>>,
    ) -> AssignedValue<F>
    where
        F: ScalarField;

    /// Computes the Euclidean distance (L2) of two quantized vectors.
    fn euclidean_distance(
        &self,
        ctx: &mut Context<F>,
        a: &Vec<AssignedValue<F>>,
        b: &Vec<AssignedValue<F>>,
    ) -> AssignedValue<F>
    where
        F: ScalarField;

    /// Computes the Cosine distance of two quantized vectors.
    /// This is equal to (1 - CosineDistance).
    fn cosine_distance(
        &self,
        ctx: &mut Context<F>,
        a: &Vec<AssignedValue<F>>,
        b: &Vec<AssignedValue<F>>,
    ) -> AssignedValue<F>
    where
        F: ScalarField;

    // Given a query vector, returns the most similar vector

    // fn nearest_vector(
    //     &self,
    //     ctx: &mut Context<F>,
    //     query: &Vec<AssignedValue<F>>,
    //     vectors: &Vec<Vec<AssignedValue<F>>>,
    // ) -> Vec<AssignedValue<F>>
    // where
    //     F: ScalarField;

    // /// Commits to an array of vectors.
    // /// TODO: allow non-power-of-two lengths
    // fn merkle_commitment<const T: usize, const RATE: usize>(
    //     &self,
    //     ctx: &mut Context<F>,
    //     poseidon: &mut PoseidonChip<F, T, RATE>,
    //     vectors: &Vec<Vec<AssignedValue<F>>>,
    // ) -> AssignedValue<F>
    // where
    //     F: ScalarField;
}

impl<F: ScalarField, const PRECISION_BITS: u32> DistanceInstructions<F, PRECISION_BITS>
    for DistanceChip<F, PRECISION_BITS>
{
    type FixedPointGate = FixedPointChip<F, PRECISION_BITS>;

    fn fixed_point_gate(&self) -> &Self::FixedPointGate {
        &self.fixed_point_gate
    }

    fn strategy(&self) -> DistanceStrategy {
        self.strategy
    }

    fn euclidean_distance(
        &self,
        ctx: &mut Context<F>,
        a: &Vec<AssignedValue<F>>,
        b: &Vec<AssignedValue<F>>,
    ) -> AssignedValue<F>
    where
        F: ScalarField,
    {
        assert_eq!(a.len(), b.len());

        let ab: Vec<AssignedValue<F>> =
            a.iter().zip(b).map(|(a_i, b_i)| self.fixed_point_gate.qsub(ctx, *a_i, *b_i)).collect();

        // compute sum of squares of differences via self-inner product
        let dist_square = self.fixed_point_gate.inner_product(ctx, ab.clone(), ab);

        // take the square root
        self.fixed_point_gate.qsqrt(ctx, dist_square)
    }

    fn cosine_distance(
        &self,
        ctx: &mut Context<F>,
        a: &Vec<AssignedValue<F>>,
        b: &Vec<AssignedValue<F>>,
    ) -> AssignedValue<F>
    where
        F: ScalarField,
    {
        assert_eq!(a.len(), b.len());

        let ab: AssignedValue<F> = self.fixed_point_gate.inner_product(ctx, a.clone(), b.clone()); // sum (a.b)
        let aa = self.fixed_point_gate().inner_product(ctx, a.clone(), a.clone()); // sum (a^2)
        let bb = self.fixed_point_gate.inner_product(ctx, b.clone(), b.clone()); // sum (b^2)

        let aa_sqrt = self.fixed_point_gate.qsqrt(ctx, aa);
        let bb_sqrt = self.fixed_point_gate.qsqrt(ctx, bb);

        let denom = self.fixed_point_gate.qmul(ctx, aa_sqrt, bb_sqrt);
        let sim = self.fixed_point_gate.qdiv(ctx, ab, denom);

        let one = ctx.load_constant(self.quantize(1.0));
        self.fixed_point_gate.qsub(ctx, one, sim)
    }

    fn hamming_distance(
        &self,
        ctx: &mut Context<F>,
        a: &Vec<AssignedValue<F>>,
        b: &Vec<AssignedValue<F>>,
    ) -> AssignedValue<F>
    where
        F: ScalarField,
    {
        assert_eq!(a.len(), b.len());

        let ab: Vec<AssignedValue<F>> = a
            .iter()
            .zip(b)
            .map(|(a_i, b_i)| self.fixed_point_gate.gate().is_equal(ctx, *a_i, *b_i))
            .collect();

        let ab_sum: AssignedValue<F> = self.fixed_point_gate.range_gate().gate().sum(ctx, ab);

        let len: F = self.fixed_point_gate.quantization(a.len() as f64);
        let len: AssignedValue<F> = ctx.load_witness(len);

        let ab_sum_q: F = self.fixed_point_gate.quantization(ab_sum.value().get_lower_128() as f64);
        let ab_sum_q: AssignedValue<F> = ctx.load_witness(ab_sum_q);

        let sim = self.fixed_point_gate.qdiv(ctx, ab_sum_q, len);

        let one = ctx.load_constant(self.quantize(1.0));
        self.fixed_point_gate.qsub(ctx, one, sim)
    }

    fn manhattan_distance(
        &self,
        ctx: &mut Context<F>,
        a: &Vec<AssignedValue<F>>,
        b: &Vec<AssignedValue<F>>,
    ) -> AssignedValue<F>
    where
        F: ScalarField,
    {
        assert_eq!(a.len(), b.len());

        let ab_diff: Vec<AssignedValue<F>> =
            a.iter().zip(b).map(|(a_i, b_i)| self.fixed_point_gate.qsub(ctx, *a_i, *b_i)).collect();

        let ab_diff_abs: Vec<AssignedValue<F>> =
            ab_diff.iter().map(|d| self.fixed_point_gate.qabs(ctx, *d)).collect();

        self.fixed_point_gate.range_gate().gate().sum(ctx, ab_diff_abs)
    }

    // fn nearest_vector(
    //     &self,
    //     ctx: &mut Context<F>,
    //     query: &Vec<AssignedValue<F>>,
    //     vectors: &Vec<Vec<AssignedValue<F>>>,
    //     // TODO: ask for distance metric function here
    // ) -> Vec<AssignedValue<F>>
    // where
    //     F: ScalarField,
    // {
    //     // compute distance to each vector
    //     let distances: Vec<AssignedValue<F>> =
    //         vectors.iter().map(|v| self.euclidean_distance(ctx, v, query)).collect();

    //     // find the minimum
    //     let min: AssignedValue<F> = distances
    //         .clone()
    //         .into_iter()
    //         .reduce(|acc, d| self.fixed_point_gate().qmin(ctx, acc, d))
    //         .unwrap();
    //     let min_indicator: Vec<AssignedValue<F>> = distances
    //         .into_iter()
    //         .map(|d| self.fixed_point_gate().range_gate().gate.is_equal(ctx, min, d))
    //         .collect();

    //     // get the most similar vector by selecting each index with indicator
    //     let result: Vec<AssignedValue<F>> = (0..vectors[0].len())
    //         .map(|i| {
    //             self.fixed_point_gate().range_gate().gate.select_by_indicator(
    //                 ctx,
    //                 vectors.iter().map(|d| d[i]),
    //                 min_indicator.iter().copied(),
    //             )
    //         })
    //         .collect();

    //     result
    // }

    // fn merkle_commitment<const T: usize, const RATE: usize>(
    //     &self,
    //     ctx: &mut Context<F>,
    //     poseidon: &mut PoseidonChip<F, T, RATE>,
    //     vectors: &Vec<Vec<AssignedValue<F>>>,
    // ) -> AssignedValue<F>
    // where
    //     F: ScalarField,
    // {
    //     // hash vectors in db
    //     let hashes: Vec<AssignedValue<F>> = vectors
    //         .iter()
    //         .map(|v| {
    //             poseidon.clear();
    //             poseidon.update(&v.as_slice());
    //             poseidon.squeeze(ctx, self.fixed_point_gate().range_gate().gate()).unwrap()
    //         })
    //         .collect();

    //     // construct merklee tree from the hashes
    //     let mut leaves: Vec<AssignedValue<F>> = hashes; // TODO: make this extend with zeros for powers of two
    //     while leaves.len() > 1 {
    //         // assert that the number of leaves is a power of two
    //         assert!((leaves.len() & (leaves.len() - 1)) == 0);
    //         let mut next_leaves = Vec::with_capacity(leaves.len() / 2);
    //         for i in (0..leaves.len()).step_by(2) {
    //             poseidon.clear();
    //             poseidon.update(&[leaves[i], leaves[i + 1]]);
    //             next_leaves.push(
    //                 poseidon.squeeze(ctx, self.fixed_point_gate().range_gate().gate()).unwrap(),
    //             );
    //         }
    //         leaves = next_leaves;
    //     }
    //     assert!(leaves.len() == 1);
    //     leaves[0]
    // }
}
