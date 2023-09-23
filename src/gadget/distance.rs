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

        let one = ctx.load_constant(self.fixed_point_gate().quantization(1.0));
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

        let one = ctx.load_constant(self.fixed_point_gate().quantization(1.0));
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
}
