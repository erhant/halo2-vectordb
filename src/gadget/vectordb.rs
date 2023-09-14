use halo2_base::{
    gates::{GateInstructions, RangeInstructions},
    utils::ScalarField,
    AssignedValue, Context,
};
use poseidon::PoseidonChip;
use std::fmt::Debug;

use super::fixed_point::{FixedPointChip, FixedPointInstructions};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VectorDBStrategy {
    Vertical, // vanilla implementation with vertical basic gate(s)
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
    /// TODO: allow non-power-of-two lengths
    fn merkle_commitment<const T: usize, const RATE: usize>(
        &self,
        ctx: &mut Context<F>,
        poseidon: &mut PoseidonChip<F, T, RATE>,
        vectors: &Vec<Vec<AssignedValue<F>>>,
    ) -> AssignedValue<F>
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
            .map(|d| self.fixed_point_gate().range_gate().gate.is_equal(ctx, min, d))
            .collect();

        // get the most similar vector by selecting each index with indicator
        let result: Vec<AssignedValue<F>> = (0..vectors[0].len())
            .map(|i| {
                self.fixed_point_gate().range_gate().gate.select_by_indicator(
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
        // hash vectors in db
        let hashes: Vec<AssignedValue<F>> = vectors
            .iter()
            .map(|v| {
                poseidon.clear();
                poseidon.update(&v.as_slice());
                poseidon.squeeze(ctx, self.fixed_point_gate().range_gate().gate()).unwrap()
            })
            .collect();

        // construct merklee tree from the hashes
        let mut leaves: Vec<AssignedValue<F>> = hashes; // TODO: make this extend with zeros for powers of two
        while leaves.len() > 1 {
            // assert that the number of leaves is a power of two
            assert!((leaves.len() & (leaves.len() - 1)) == 0);
            let mut next_leaves = Vec::with_capacity(leaves.len() / 2);
            for i in (0..leaves.len()).step_by(2) {
                poseidon.clear();
                poseidon.update(&[leaves[i], leaves[i + 1]]);
                next_leaves.push(
                    poseidon.squeeze(ctx, self.fixed_point_gate().range_gate().gate()).unwrap(),
                );
            }
            leaves = next_leaves;
        }
        assert!(leaves.len() == 1);
        leaves[0]
    }
}
