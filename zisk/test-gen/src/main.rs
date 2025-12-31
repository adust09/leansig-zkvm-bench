//! Test vector generator for Zisk verification.
//!
//! This program generates test vectors using the shared test-gen-core library.

use test_gen_core::{generate_test_input, generate_and_write, print_summary, verify_roundtrip};
use leansig_core::{NUM_CHAINS, TREE_HEIGHT};

fn main() {
    println!("=== leanSig Test Vector Generator (Zisk) ===");
    println!("Parameters: TargetSum W=1, {} chains, tree height {}", NUM_CHAINS, TREE_HEIGHT);
    println!("Generating synthetic test input for benchmarking...\n");

    let input = generate_test_input();

    // Write to Zisk data directory
    let output_path = "../data/input.bin";
    println!("Serializing with postcard...");
    let serialized = generate_and_write(output_path).expect("Failed to write test vector");

    print_summary(&input, serialized.len());
    println!("\nTest vector written to data/input.bin");

    // Verify roundtrip
    println!("\nVerifying roundtrip deserialization...");
    if verify_roundtrip(&input, &serialized) {
        println!("Roundtrip verification passed!");
    } else {
        eprintln!("Roundtrip verification failed!");
        std::process::exit(1);
    }

    println!("\n=== Done ===");
    println!("Note: This is synthetic test data for benchmarking.");
    println!("      The signature will not verify correctly,");
    println!("      but it exercises the full verification code path.");
}
