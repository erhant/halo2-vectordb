use clap::Parser;
use halo2_base::gates::{GateChip, GateInstructions};
use halo2_base::utils::ScalarField;
use halo2_base::{AssignedValue, QuantumCell};
#[allow(unused_imports)]
use halo2_base::{
    Context,
    QuantumCell::{Constant, Existing, Witness},
};
use halo2_scaffold::scaffold::cmd::Cli;
use halo2_scaffold::scaffold::run;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircuitInput {
    pub a: Vec<u8>,
    pub b: Vec<u8>,
}

fn int_dot_product<F: ScalarField>(
    ctx: &mut Context<F>,
    input: CircuitInput,
    make_public: &mut Vec<AssignedValue<F>>,
) {
    assert_eq!(input.a.len(), input.b.len());
    let gate = GateChip::<F>::default();

    // map vectors to field elements
    let a: Vec<AssignedValue<F>> =
        ctx.assign_witnesses(input.a.iter().map(|a_i| F::from(*a_i as u64)));
    let b: Vec<AssignedValue<F>> =
        ctx.assign_witnesses(input.b.iter().map(|b_i| F::from(*b_i as u64)));

    // compute
    let out = gate.inner_product(ctx, a, b.into_iter().map(QuantumCell::Existing));
    make_public.push(out);
    println!("out: {:?}", out.value());
}

fn main() {
    env_logger::init();

    let args = Cli::parse();
    run(int_dot_product, args);
}
