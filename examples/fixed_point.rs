// MIT License

// Copyright (c) 2023 Wentao Xiao

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use clap::Parser;
use halo2_base::utils::{BigPrimeField, ScalarField};
use halo2_base::AssignedValue;
use halo2_base::Context;
use halo2_scaffold::gadget::fixed_point::{FixedPointChip, FixedPointInstructions};
use halo2_scaffold::scaffold::cmd::Cli;
use halo2_scaffold::scaffold::run;
use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
// use halo2_scaffold::scaffold::{mock, prove};
use std::env::{set_var, var};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircuitInput {
    pub x: f64,
}

fn fixed_point<F: ScalarField>(
    ctx: &mut Context<F>,
    input: CircuitInput,
    make_public: &mut Vec<AssignedValue<F>>,
) where
    F: BigPrimeField,
{
    // `Context` can roughly be thought of as a single-threaded execution trace of a program we want to ZK prove. We do some post-processing on `Context` to optimally divide the execution trace into multiple columns in a PLONKish arithmetization
    // More advanced usage with multi-threaded witness generation is possible, but we do not explain it here

    // lookup bits must agree with the size of the lookup table, which is specified by an environmental variable
    let lookup_bits =
        var("LOOKUP_BITS").unwrap_or_else(|_| panic!("LOOKUP_BITS not set")).parse().unwrap();
    const PRECISION_BITS: u32 = 32;
    let fixed_point_chip = FixedPointChip::<F, PRECISION_BITS>::default(lookup_bits);

    let x_decimal = input.x;
    let x = fixed_point_chip.quantization(input.x);
    println!("x: {:?}", x);

    // first we load a number `x` into as system, as a "witness"
    let x = ctx.load_witness(x);
    // by default, all numbers in the system are private
    // we can make it public like so:
    make_public.push(x);

    let exp_1 = fixed_point_chip.qexp2(ctx, x);
    let y_decimal = fixed_point_chip.dequantization(*exp_1.value());
    let y_native = x_decimal.exp2();
    println!(
        "###### zk-exp2({:.6}) = {}, native-exp({:.6}) = {:.6}, error = {:.6} ({:.6}%)",
        x_decimal,
        y_decimal,
        x_decimal,
        y_native,
        (y_decimal - y_native).abs(),
        (y_decimal - y_native).abs() / y_native.abs() * 100.0
    );
    make_public.push(exp_1);

    if x_decimal > 0f64 {
        let log_2 = fixed_point_chip.qlog2(ctx, x);
        let y_decimal_2 = fixed_point_chip.dequantization(*log_2.value());
        let y_native_2 = x_decimal.log2();
        println!(
            "###### zk-log2({:.6}) = {}, native-log2({:.6}) = {:.6}, error = {:.6} ({:.6}%)",
            x_decimal,
            y_decimal_2,
            x_decimal,
            y_native_2,
            (y_decimal_2 - y_native_2).abs(),
            (y_decimal_2 - y_native_2).abs() / y_native_2.abs() * 100.0
        );
        make_public.push(log_2);
    }

    let sin_x = fixed_point_chip.qsin(ctx, x);
    println!("sin_x_q: {:?}", *sin_x.value());
    let y_decimal_3 = fixed_point_chip.dequantization(*sin_x.value());
    let y_native_3 = x_decimal.sin();
    println!(
        "###### zk-sin({:.6}) = {}, native-sin({:.6}) = {:.6}, error = {:.6} ({:.6}%)",
        x_decimal,
        y_decimal_3,
        x_decimal,
        y_native_3,
        (y_decimal_3 - y_native_3).abs(),
        (y_decimal_3 - y_native_3).abs() / y_native_3.abs() * 100.0
    );
    make_public.push(sin_x);
}

fn main() {
    env_logger::init();

    // // genrally lookup_bits is degree - 1
    // set_var("LOOKUP_BITS", 12.to_string());
    // set_var("DEGREE", 13.to_string());

    // run mock prover
    // mock(some_algorithm_in_zk, -12.0);
    // mock(some_algorithm_in_zk, -1.88724767676867);
    // mock(some_algorithm_in_zk, 0.0);
    // mock(some_algorithm_in_zk, 1.0);
    // mock(some_algorithm_in_zk, 1.128);
    // mock(some_algorithm_in_zk, 2.0);
    // mock(some_algorithm_in_zk, 4.0);
    // mock(some_algorithm_in_zk, 0.25 * std::f64::consts::PI);

    // uncomment below to run actual prover:
    // the 3rd parameter is a dummy input to provide for the proving key generation
    // prove(some_algorithm_in_zk, 0.25 * std::f64::consts::PI, 0.5 * std::f64::consts::PI);

    let args = Cli::parse();

    // run different zk commands based on the command line arguments
    run(fixed_point, args);
}
