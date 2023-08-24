use clap::Parser;
use halo2_base::gates::{GateChip, GateInstructions};
use halo2_base::utils::ScalarField;
use halo2_base::AssignedValue;
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
    pub u: Vec<u64>,
    pub v: Vec<u64>,
}

fn dot_product<F: ScalarField>(
    ctx: &mut Context<F>,
    input: CircuitInput,
    make_public: &mut Vec<AssignedValue<F>>,
) {
    // map vectors to field elements
    let mut u = ctx.assign_witnesses(input.u.into_iter().map(F::from));
    let mut v = ctx.assign_witnesses(input.v.into_iter().map(F::from));
    // assert_eq!(u.len(), 5);
    // assert_eq!(v.len(), 5);

    // compute dot product via the Gate Chip
    let gate = GateChip::<F>::default();

    // TODO
    let out = gate.inner_product(ctx, u, v);

    make_public.push(out);
    println!("out: {:?}", out.value());
}

fn main() {
    env_logger::init();

    let args = Cli::parse();
    run(dot_product, args);
}
