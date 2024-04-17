#![no_std]
#![no_main]
use zkwasm_rust_sdk::wasm_input;
use zkwasm_rust_sdk::require;
use wasm_bindgen::prelude::wasm_bindgen;

fn fib(n: u64) -> u64 {
    const MOD: u64 = 1_000_000_007;
    let mut a: u64 = 0;
    let mut b: u64 = 1;
    let mut sum: u64;
    for _ in 1..n {
        sum = (a + b) % MOD;
        a = b;
        b = sum;
    }

    b
}

#[wasm_bindgen]
pub fn zkmain() {
    // 1 is public, 0 is private
    let n = unsafe { wasm_input(1) };
    let result = fib(n);
    unsafe {require(result > 0)};
}