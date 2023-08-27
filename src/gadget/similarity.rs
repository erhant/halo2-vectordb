use halo2_base::QuantumCell::{Constant, Existing, Witness};
use halo2_base::{
    gates::{range::RangeStrategy, GateChip, GateInstructions, RangeChip, RangeInstructions},
    utils::{biguint_to_fe, fe_to_biguint, BigPrimeField, ScalarField},
    AssignedValue, Context, QuantumCell,
};
use num_bigint::BigUint;
use num_integer::Integer;
use std::{fmt::Debug, ops::Sub};

use super::fixed_point::{FixedPointChip, FixedPointInstructions, FixedPointStrategy};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SimilarityStrategy {
    Vertical, // vanilla implementation with vertical basic gate(s)
}

/// `PRECISION_BITS` indicates the precision of integer and fractional parts.
/// For example, `PRECISION_BITS = 32` indicates this chip implements 32.32 fixed point decimal arithmetics.
/// The valid range of the fixed point decimal is -max_value < x < max_value.
#[derive(Clone, Debug)]
pub struct SimilarityChip<F: BigPrimeField, const PRECISION_BITS: u32> {
    strategy: SimilarityStrategy,
    pub fixed_point_gate: FixedPointChip<F, PRECISION_BITS>,
}

impl<F: BigPrimeField, const PRECISION_BITS: u32> SimilarityChip<F, PRECISION_BITS> {
    pub fn new(strategy: SimilarityStrategy, lookup_bits: usize) -> Self {
        let fixed_point_gate = FixedPointChip::<F, PRECISION_BITS>::new(
            match strategy {
                SimilarityStrategy::Vertical => FixedPointStrategy::Vertical,
            },
            lookup_bits,
        );

        Self { strategy, fixed_point_gate }
    }

    pub fn default(lookup_bits: usize) -> Self {
        Self::new(SimilarityStrategy::Vertical, lookup_bits)
    }

    pub fn quantize(&self, x: f64) -> F {
        self.fixed_point_gate.quantization(x)
    }

    pub fn dequantize(&self, x: F) -> f64 {
        self.fixed_point_gate.dequantization(x)
    }

    pub fn quantize_vector(&self, a: Vec<f64>) -> Vec<F> {
        a.iter().map(|a_i| self.fixed_point_gate.quantization(*a_i)).collect()
    }
}

pub trait SimilarityInstructions<F: ScalarField, const PRECISION_BITS: u32> {
    type FixedPointGate: FixedPointInstructions<F, PRECISION_BITS>;

    fn fixed_point_gate(&self) -> &Self::FixedPointGate;

    fn strategy(&self) -> SimilarityStrategy;

    fn dot_product<QA>(
        &self,
        ctx: &mut Context<F>,
        a: impl IntoIterator<Item = QA>,
        b: impl IntoIterator<Item = QA>,
    ) -> AssignedValue<F>
    where
        F: BigPrimeField,
        QA: Into<QuantumCell<F>> + Copy;

    fn hamming<QA>(
        &self,
        ctx: &mut Context<F>,
        a: impl IntoIterator<Item = QA>,
        b: impl IntoIterator<Item = QA>,
    ) -> AssignedValue<F>
    where
        F: BigPrimeField,
        QA: Into<QuantumCell<F>> + Copy;

    fn euclidean<QA>(
        &self,
        ctx: &mut Context<F>,
        a: impl IntoIterator<Item = QA>,
        b: impl IntoIterator<Item = QA>,
    ) -> AssignedValue<F>
    where
        F: BigPrimeField,
        QA: Into<QuantumCell<F>> + Copy;

    fn cosine<QA>(
        &self,
        ctx: &mut Context<F>,
        a: impl IntoIterator<Item = QA>,
        b: impl IntoIterator<Item = QA>,
    ) -> AssignedValue<F>
    where
        F: BigPrimeField,
        QA: Into<QuantumCell<F>> + Copy;
}

impl<F: BigPrimeField, const PRECISION_BITS: u32> SimilarityInstructions<F, PRECISION_BITS>
    for SimilarityChip<F, PRECISION_BITS>
{
    type FixedPointGate = FixedPointChip<F, PRECISION_BITS>;

    fn fixed_point_gate(&self) -> &Self::FixedPointGate {
        &self.fixed_point_gate
    }

    fn strategy(&self) -> SimilarityStrategy {
        self.strategy
    }

    fn dot_product<QA>(
        &self,
        ctx: &mut Context<F>,
        a: impl IntoIterator<Item = QA>,
        b: impl IntoIterator<Item = QA>,
    ) -> AssignedValue<F>
    where
        F: BigPrimeField,
        QA: Into<QuantumCell<F>> + Copy,
    {
        self.fixed_point_gate.inner_product(ctx, a, b)
    }

    fn euclidean<QA>(
        &self,
        ctx: &mut Context<F>,
        a: impl IntoIterator<Item = QA>,
        b: impl IntoIterator<Item = QA>,
    ) -> AssignedValue<F>
    where
        F: BigPrimeField,
        QA: Into<QuantumCell<F>> + Copy,
    {
        let a: Vec<QA> = a.into_iter().collect();
        let b: Vec<QA> = b.into_iter().collect();
        assert_eq!(a.len(), b.len());

        let ab: Vec<AssignedValue<F>> = a
            .iter()
            .zip(&b)
            .map(|(a_i, b_i)| self.fixed_point_gate.qsub(ctx, *a_i, *b_i))
            .collect();

        // compute sum of squares of differences via self-inner product
        let dist_square = self.fixed_point_gate.inner_product(ctx, ab.clone(), ab);

        // take the square root
        self.fixed_point_gate.qsqrt(ctx, dist_square)
    }

    fn cosine<QA>(
        &self,
        ctx: &mut Context<F>,
        a: impl IntoIterator<Item = QA>,
        b: impl IntoIterator<Item = QA>,
    ) -> AssignedValue<F>
    where
        F: BigPrimeField,
        QA: Into<QuantumCell<F>> + Copy,
    {
        let a: Vec<QA> = a.into_iter().collect();
        let b: Vec<QA> = b.into_iter().collect();
        assert_eq!(a.len(), b.len());

        let ab: AssignedValue<F> = self.fixed_point_gate.inner_product(ctx, a.clone(), b.clone()); // sum (a.b)
        let aa = self.fixed_point_gate.inner_product(ctx, a.clone(), a); // sum (a^2)
        let bb = self.fixed_point_gate.inner_product(ctx, b.clone(), b); // sum (b^2)

        let aa_sqrt = self.fixed_point_gate.qsqrt(ctx, aa);
        let bb_sqrt = self.fixed_point_gate.qsqrt(ctx, bb);

        let denom = self.fixed_point_gate.qmul(ctx, aa_sqrt, bb_sqrt);
        self.fixed_point_gate.qdiv(ctx, ab, denom)
    }

    fn hamming<QA>(
        &self,
        ctx: &mut Context<F>,
        a: impl IntoIterator<Item = QA>,
        b: impl IntoIterator<Item = QA>,
    ) -> AssignedValue<F>
    where
        F: BigPrimeField,
        QA: Into<QuantumCell<F>> + Copy,
    {
        let a: Vec<QA> = a.into_iter().collect();
        let b: Vec<QA> = b.into_iter().collect();
        assert_eq!(a.len(), b.len());

        let ab: Vec<AssignedValue<F>> = a
            .iter()
            .zip(&b)
            .map(|(a_i, b_i)| self.fixed_point_gate.gate().is_equal(ctx, *a_i, *b_i))
            .collect();

        // TODO weird type error?
        let ab_sum: AssignedValue<F> = self.fixed_point_gate.range_gate().gate().sum(ctx, ab);

        let len: F = self.fixed_point_gate.quantization(a.len() as f64);
        let len: AssignedValue<F> = ctx.load_witness(len);

        let ab_sum_q: F = self.fixed_point_gate.quantization(ab_sum.value().get_lower_128() as f64);
        let ab_sum_q: AssignedValue<F> = ctx.load_witness(ab_sum_q);

        self.fixed_point_gate.qdiv(ctx, ab_sum_q, len)
    }
}
