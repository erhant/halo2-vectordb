use clap::Parser;
use halo2_base::utils::ScalarField;
use halo2_base::AssignedValue;
#[allow(unused_imports)]
use halo2_base::{
    Context,
    QuantumCell::{Constant, Existing, Witness},
};
use halo2_scaffold::gadget::similarity::{SimilarityChip, SimilarityInstructions};
use halo2_scaffold::scaffold::cmd::Cli;
use halo2_scaffold::scaffold::run;
use serde::{Deserialize, Serialize};
use std::env::var;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircuitInput {
    pub a: Vec<f64>,
    pub b: Vec<f64>,
}

fn euclidean_distance<F: ScalarField>(
    ctx: &mut Context<F>,
    input: CircuitInput,
    make_public: &mut Vec<AssignedValue<F>>,
) {
    assert_eq!(input.a.len(), input.b.len());

    let lookup_bits =
        var("LOOKUP_BITS").unwrap_or_else(|_| panic!("LOOKUP_BITS not set")).parse().unwrap();
    const PRECISION_BITS: u32 = 32;
    let similarity_chip = SimilarityChip::<F, PRECISION_BITS>::default(lookup_bits);

    let a: Vec<AssignedValue<F>> = ctx.assign_witnesses(similarity_chip.quantize_vector(input.a));
    let b: Vec<AssignedValue<F>> = ctx.assign_witnesses(similarity_chip.quantize_vector(input.b));

    let dist: AssignedValue<F> = similarity_chip.euclidean(ctx, a.clone(), b.clone());
    let dist_native = similarity_chip.dequantize(*dist.value());
    println!("euclidean similarity: {:?}", dist_native);
    make_public.push(dist);

    let dist: AssignedValue<F> = similarity_chip.manhattan(ctx, a.clone(), b.clone());
    let dist_native = similarity_chip.dequantize(*dist.value());
    println!("manhattan similarity: {:?}", dist_native);
    make_public.push(dist);

    let dist: AssignedValue<F> = similarity_chip.cosine(ctx, a.clone(), b.clone());
    let dist_native = similarity_chip.dequantize(*dist.value());
    println!("cosine similarity: {:?}", dist_native);
    make_public.push(dist);

    let dist: AssignedValue<F> = similarity_chip.hamming(ctx, a.clone(), b.clone());
    let dist_native = similarity_chip.dequantize(*dist.value());
    println!("hamming similarity: {:?}", dist_native);
    make_public.push(dist);

    let dist: AssignedValue<F> = similarity_chip.dot_product(ctx, a, b);
    let dist_native = similarity_chip.dequantize(*dist.value());
    println!("dot product similarity: {:?}", dist_native);
    make_public.push(dist);
}

fn main() {
    env_logger::init();

    let args = Cli::parse();
    run(euclidean_distance, args);
}
