// Basic RISC Zero Proof Generation Example
// Demonstrates the complete proof generation workflow

use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use serde::{Deserialize, Serialize};

// Import your guest ELF and ID from methods crate
use methods::{GUEST_ELF, GUEST_ID};

/// Input data sent to guest program
#[derive(Debug, Serialize, Deserialize)]
struct GuestInput {
    pub value: u64,
    pub message: String,
}

/// Output data committed by guest program
#[derive(Debug, Serialize, Deserialize)]
struct GuestOutput {
    pub result: u64,
    pub hash: [u8; 32],
}

/// Generate a zero-knowledge proof for the given input
fn generate_proof(input: &GuestInput) -> anyhow::Result<Receipt> {
    // Step 1: Build execution environment with serialized input
    let env = ExecutorEnv::builder()
        .write(input)?  // Serialize input for guest
        .build()?;

    // Step 2: Get the default prover
    // This respects RISC0_DEV_MODE and RISC0_CUDA environment variables
    let prover = default_prover();

    // Step 3: Execute and prove
    println!("Generating proof...");
    let receipt = prover.prove(env, GUEST_ELF)?;
    println!("Proof generated successfully!");

    // Step 4: Verify the proof locally (recommended)
    println!("Verifying proof...");
    receipt.verify(GUEST_ID)?;
    println!("Proof verified!");

    Ok(receipt)
}

/// Extract and decode the output from a receipt
fn decode_output(receipt: &Receipt) -> anyhow::Result<GuestOutput> {
    let output: GuestOutput = receipt.journal.decode()?;
    Ok(output)
}

/// Compress receipt to Succinct format
fn compress_receipt(receipt: Receipt) -> anyhow::Result<Receipt> {
    println!("Compressing to Succinct...");
    let succinct = receipt.compress()?;
    println!("Compressed receipt size: {} bytes",
        bincode::serialize(&succinct)?.len());
    Ok(succinct)
}

fn main() -> anyhow::Result<()> {
    // Prepare input
    let input = GuestInput {
        value: 42,
        message: "Hello, RISC Zero!".to_string(),
    };
    println!("Input: {:?}", input);

    // Generate proof
    let receipt = generate_proof(&input)?;

    // Decode output from journal
    let output = decode_output(&receipt)?;
    println!("Output: {:?}", output);

    // Print receipt info
    let receipt_bytes = bincode::serialize(&receipt)?;
    println!("Receipt size: {} bytes", receipt_bytes.len());

    // Optionally compress
    // let succinct = compress_receipt(receipt)?;

    Ok(())
}

// Guest program example (would be in guest/src/main.rs)
/*
#![no_main]
#![no_std]

use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);

fn main() {
    // Read input from host
    let input: GuestInput = env::read();

    // Perform computation
    let result = input.value * 2;
    let hash = risc0_zkvm::sha::Impl::hash_bytes(input.message.as_bytes());

    // Commit output to journal (public)
    let output = GuestOutput { result, hash };
    env::commit(&output);
}
*/
