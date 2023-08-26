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
    pub a: Vec<f32>,
    pub b: Vec<f32>,
}

fn euclidean_distance<F: ScalarField>(
    ctx: &mut Context<F>,
    input: CircuitInput,
    make_public: &mut Vec<AssignedValue<F>>,
) {
    assert_eq!(input.a.len(), input.b.len());

    // USE FIXED POINT CHIP

    println!("out: {:?}", input.a[0]);
}

fn main() {
    env_logger::init();

    let args = Cli::parse();
    run(euclidean_distance, args);
}
