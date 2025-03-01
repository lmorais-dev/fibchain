use std::io::Read;
use alloy_sol_types::SolValue;
use risc0_zkvm::guest::env;

fn main() {
    let mut input_bytes = Vec::<u8>::new();
    env::stdin().read_to_end(&mut input_bytes).unwrap();

    let iterations = <u16>::abi_decode(&input_bytes, true).unwrap();

    let mut a: u128 = 0;
    let mut b: u128 = 1;

    for _ in 0..iterations {
        let temp = a;
        a = b;
        b += temp;
    }

    env::commit_slice(b.abi_encode().as_slice());
}