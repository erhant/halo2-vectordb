use halo2_base::{utils::ScalarField, AssignedValue, Context};

use super::fixed_point::FixedPointChip;

pub trait FixedPointVectorInstructions<F: ScalarField, const PRECISION_BITS: u32> {
    /// Calls `quantize` on a vector of elements.
    fn quantize_vector(&self, v: &Vec<f64>) -> Vec<F>;

    /// Calls `dequantize` on a vector of elements.
    fn dequantize_vector(&self, v: &Vec<F>) -> Vec<f64>;

    /// Calls `quantize` on a vector of elements, and assigns them to context with `assign_witnesses`.
    fn quantize_and_assign_vector(
        &self,
        ctx: &mut Context<F>,
        v: &Vec<f64>,
    ) -> Vec<AssignedValue<F>>;
}

impl<F: ScalarField, const PRECISION_BITS: u32> FixedPointVectorInstructions<F, PRECISION_BITS>
    for FixedPointChip<F, PRECISION_BITS>
{
    fn quantize_vector(&self, v: &Vec<f64>) -> Vec<F> {
        v.iter().map(|v_i| self.quantization(*v_i)).collect()
    }

    fn dequantize_vector(&self, v: &Vec<F>) -> Vec<f64> {
        v.iter().map(|v_i| self.dequantization(*v_i)).collect()
    }

    fn quantize_and_assign_vector(
        &self,
        ctx: &mut Context<F>,
        v: &Vec<f64>,
    ) -> Vec<AssignedValue<F>> {
        ctx.assign_witnesses(self.quantize_vector(v))
    }
}
