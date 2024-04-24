use lazy_static::lazy_static;
use std::collections::HashSet;
use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use std::{
    time::{Duration, Instant},
    usize,
};
use std::fs;
use utils::benchmark;

const ZKWASM: &str = "ZKWASMCLI";
const OUTOUT_DIR: &str = "./output";
const PARAMS_DIR: &str = "./params";
const PROOF_FILE: &str = "./output/zkwasm.0.transcript.data";
const GUESTS: [&str; 6] = [
    "fibonacci-guest",
    "sha2-guest",
    "sha2-chain-guest",
    "sha3-chain-guest",
    "sha3-guest",
    "bigmem-guest",
];
const K: usize = 18;

lazy_static! {
    static ref SETUP_COMPLETED: Mutex<HashSet<&'static str>> = Mutex::new(HashSet::new());
}

macro_rules! zkwasm_cli {
    () => {
        env::var(ZKWASM).expect("CLI environment variable not set")
    };
}

fn format_image_dir(guest_name: &str) -> String {
    let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");

    // Create a PathBuf and push the relative path to the file
    let mut file_path = PathBuf::from(cargo_manifest_dir);
    file_path.push(guest_name);
    file_path.push("target");
    file_path.push("guest.wasm");

    // Convert the PathBuf to a string representation
    file_path.to_str().unwrap_or("Invalid path").to_string()
}

fn main() {
    // let ns = [100, 1000, 10000, 50000];
    let ns = [100];
    // benchmark(
    //     bench_fibonacci,
    //     &ns,
    //     "../benchmark_outputs/fiboancci_zkwasm.csv",
    //     "n",
    // );
    // let lengths = [32, 256, 512, 1024, 2048];
    let lengths = [32];
    benchmark(
        bench_sha2,
        &lengths,
        "../benchmark_outputs/sha2_zkwasm.csv",
        "byte length",
    );
}

fn bench_fibonacci(n: u64) -> (Duration, usize) {
    // guest name
    let guest = "fibonacci-guest";
    // Assert it is a valid guest
    assert!(GUESTS.contains(&guest));
    // Setup should not counted as part of the benchmark
    prepare(n, K, guest);
    // Start the timer
    let start = Instant::now();
    // Prove the statement
    prove(n, K, guest);
    // Stop the timer
    let end = Instant::now();
    // Calculate the size of proof
    let size = std::fs::metadata(PROOF_FILE).unwrap().len() as usize;
    // Verify the proof
    verify(n, K, guest);

    (end.duration_since(start), size)
}

fn bench_sha2_chain(iters: u32) -> (Duration, usize) {
    let guest = "sha2-chain-guest";
    assert!(GUESTS.contains(&guest));
    prepare(iters as u64, K, guest);
    let start = Instant::now();
    prove(iters as u64, K, guest);
    let end = Instant::now();
    let size = std::fs::metadata(PROOF_FILE).unwrap().len() as usize;
    verify(iters as u64, K, guest);

    (end.duration_since(start), size)
}

fn bench_sha3_chain(iters: u32) -> (Duration, usize) {
    let guest = "sha3-chain-guest";
    assert!(GUESTS.contains(&guest));
    prepare(iters as u64, K, guest);
    let start = Instant::now();
    prove(iters as u64, K, guest);
    let end = Instant::now();
    let size = std::fs::metadata(PROOF_FILE).unwrap().len() as usize;
    verify(iters as u64, K, guest);

    (end.duration_since(start), size)
}

fn bench_sha2(num_bytes: usize) -> (Duration, usize) {
    let k: usize = 23;
    let guest = "sha2-guest";
    assert!(GUESTS.contains(&guest));
    meta_build(guest, num_bytes);
    prepare(num_bytes as u64, k, guest);
    let start = Instant::now();
    prove(num_bytes as u64, k, guest);
    let end = Instant::now();
    let size = std::fs::metadata(PROOF_FILE).unwrap().len() as usize;
    verify(num_bytes as u64, k, guest);

    (end.duration_since(start), size)
}

fn bench_sha3(num_bytes: usize) -> (Duration, usize) {
    let guest = "sha3-guest";
    assert!(GUESTS.contains(&guest));
    prepare(num_bytes as u64, K, guest);
    let start = Instant::now();
    prove(num_bytes as u64, K, guest);
    let end = Instant::now();
    let size = std::fs::metadata(PROOF_FILE).unwrap().len() as usize;
    verify(num_bytes as u64, K, guest);

    (end.duration_since(start), size)
}

fn bench_bigmem(num_bytes: usize) -> (Duration, usize) {
    let guest = "bigmem-guest";
    assert!(GUESTS.contains(&guest));
    prepare(num_bytes as u64, K, guest);
    let start = Instant::now();
    prove(num_bytes as u64, K, guest);
    let end = Instant::now();
    let size = std::fs::metadata(PROOF_FILE).unwrap().len() as usize;
    verify(num_bytes as u64, K, guest);

    (end.duration_since(start), size)
}

fn meta_build(guest_name: &'static str, input_size: usize) {
    let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");

    // Create a PathBuf and push the relative path to the file
    let mut file_path = PathBuf::from(cargo_manifest_dir);
    file_path.push(guest_name);
    file_path.push("src");
    file_path.push("template");

    let template_path = file_path.to_str().unwrap_or("Invalid path");
    println!("Template path: {}", template_path);
    let template = fs::read_to_string(template_path).expect("Failed to read guest_template.rs");

    // Substitute the {{INPUT_SIZE}} placeholder with the actual input size
    let guest_code = template.replace("{{INPUT_SIZE}}", &input_size.to_string());

    // write as lib.rs
    file_path.pop();
    file_path.push("lib.rs");
    println!("Guest code path {}", file_path.display());
    fs::write(file_path.clone(), guest_code).expect("Failed to write guest lib.rs");

    // get into the guest directory
    file_path.pop();
    file_path.pop();

    println!("Guest directory: {}", file_path.display());
    // run make on the guest directory
    let status = Command::new("make")
        .current_dir(file_path)
        .status()
        .expect("Failed to run make command");

    if !status.success() {
        panic!("Make command failed");
    }

}

fn prepare(_: u64, k: usize, guest: &'static str) {
    let mut completed_guests = SETUP_COMPLETED.lock().unwrap();

    // Setup once and only once for each guest
    if !completed_guests.contains(guest) {
        // Remove existing params and output directories
        std::fs::remove_dir_all("params").ok();
        std::fs::remove_dir_all("output").ok();

        let cli = zkwasm_cli!();
        // Run command to prepare the zkWASM setup
        let status = Command::new(&cli)
            .arg("--host")
            .arg("standard")
            .arg("-k")
            .arg(k.to_string())
            .arg("--function")
            .arg("zkmain")
            .arg("--param")
            .arg(PARAMS_DIR)
            .arg("--output")
            .arg(OUTOUT_DIR)
            .arg("--wasm")
            .arg(format_image_dir(guest))
            .arg("setup")
            .status()
            .expect("Failed to run prepare command");

        if !status.success() {
            panic!("Prepare command failed");
        }

        completed_guests.insert(guest);
    }
}

fn prove(n: u64, k: usize, guest: &str) {
    let cli = zkwasm_cli!();

    let status = Command::new(&cli)
        .arg("--host")
        .arg("standard")
        .arg("-k")
        .arg(k.to_string())
        .arg("--function")
        .arg("zkmain")
        .arg("--output")
        .arg(OUTOUT_DIR)
        .arg("--param")
        .arg(PARAMS_DIR)
        .arg("--wasm")
        .arg(format_image_dir(guest))
        .arg("single-prove")
        .arg("--public")
        .arg(format!("{}:i64", n))
        .status()
        .expect("Failed to run prove command");

    if !status.success() {
        panic!("Prove command failed");
    }
}

fn verify(_: u64, k: usize, guest: &str) {
    let cli = zkwasm_cli!();

    let status = Command::new(&cli)
        .arg("--host")
        .arg("standard")
        .arg("-k")
        .arg(k.to_string())
        .arg("--function")
        .arg("zkmain")
        .arg("--output")
        .arg(OUTOUT_DIR)
        .arg("--param")
        .arg(PARAMS_DIR)
        .arg("--wasm")
        .arg(format_image_dir(guest))
        .arg("single-verify")
        .status()
        .expect("Failed to run verify command");

    if !status.success() {
        panic!("Verify command failed");
    }
}

// test meta_build
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meta_build() {
        let guest = "sha2-guest";
        let input_size = 512;
        meta_build(guest, input_size);
    }
}