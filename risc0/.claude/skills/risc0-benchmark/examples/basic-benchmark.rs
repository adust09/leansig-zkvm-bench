// Basic RISC Zero Benchmark Example
// Demonstrates cycle counting and proving time measurement

use risc0_zkvm::{default_prover, ExecutorEnv};
use std::time::Instant;

// Import your guest ELF and ID
use methods::{GUEST_ELF, GUEST_ID};

/// Benchmark result structure
#[derive(Debug)]
struct BenchmarkResult {
    total_cycles: u64,
    segment_count: usize,
    execution_time: std::time::Duration,
    proving_time: std::time::Duration,
    receipt_size: usize,
}

/// Run a complete benchmark for the given input
fn benchmark_proof(input: &[u8]) -> anyhow::Result<BenchmarkResult> {
    // Build execution environment
    let env = ExecutorEnv::builder()
        .write_slice(input)
        .build()?;

    let prover = default_prover();

    // Measure execution (without proving)
    let exec_start = Instant::now();
    let session_info = prover.execute(env.clone(), GUEST_ELF)?;
    let execution_time = exec_start.elapsed();

    // Calculate cycle metrics
    let total_cycles = session_info.cycles();
    let segment_count = session_info.segments.len();

    // Measure proving
    let prove_start = Instant::now();
    let receipt = prover.prove(env, GUEST_ELF)?;
    let proving_time = prove_start.elapsed();

    // Verify locally
    receipt.verify(GUEST_ID)?;

    // Calculate receipt size
    let receipt_size = bincode::serialize(&receipt)?.len();

    Ok(BenchmarkResult {
        total_cycles,
        segment_count,
        execution_time,
        proving_time,
        receipt_size,
    })
}

/// Run multiple iterations and report statistics
fn run_benchmark_suite(input: &[u8], iterations: usize) -> anyhow::Result<()> {
    println!("=== RISC Zero Benchmark Suite ===");
    println!("Input size: {} bytes", input.len());
    println!("Iterations: {}", iterations);
    println!();

    let mut results = Vec::with_capacity(iterations);

    for i in 0..iterations {
        println!("Running iteration {}...", i + 1);
        let result = benchmark_proof(input)?;
        println!("  Cycles: {}", result.total_cycles);
        println!("  Proving time: {:?}", result.proving_time);
        results.push(result);
    }

    // Calculate statistics
    let avg_cycles: u64 = results.iter().map(|r| r.total_cycles).sum::<u64>() / iterations as u64;
    let avg_proving_ms: u128 = results.iter().map(|r| r.proving_time.as_millis()).sum::<u128>() / iterations as u128;
    let avg_receipt_size: usize = results.iter().map(|r| r.receipt_size).sum::<usize>() / iterations;

    println!();
    println!("=== Summary ===");
    println!("Average cycles: {}", avg_cycles);
    println!("Average proving time: {}ms", avg_proving_ms);
    println!("Average receipt size: {} bytes", avg_receipt_size);
    println!("Segments: {}", results[0].segment_count);

    Ok(())
}

fn main() -> anyhow::Result<()> {
    // Example: benchmark with sample input
    let input = b"Hello, RISC Zero!";

    // Single benchmark
    let result = benchmark_proof(input)?;
    println!("{:#?}", result);

    // Or run suite
    // run_benchmark_suite(input, 5)?;

    Ok(())
}
