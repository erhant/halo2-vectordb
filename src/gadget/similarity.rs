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
        let a: Vec<QA> = a.into_iter().collect();
        let b: Vec<QA> = b.into_iter().collect();
        assert!(a.len() == b.len());
        let mut res = self.qadd(ctx, Constant(F::zero()), Constant(F::zero()));
        for (ai, bi) in a.iter().zip(b.iter()).into_iter() {
            let ai_bi = self.qmul(ctx, *ai, *bi);
            res = self.qadd(ctx, res, ai_bi);
        }

        res
    }
}
