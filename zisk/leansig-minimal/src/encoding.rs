//! Incomparable encoding implementation (target-sum based).

use alloc::vec;
use alloc::vec::Vec;
use sha3::{Shake128, digest::{Update, ExtendableOutput, XofReader}};

use crate::MESSAGE_LENGTH;

/// Encoding dimension (number of chunks)
pub const DIMENSION: usize = 256;
/// Base for chunk values (max value + 1)
pub const BASE: u32 = 256;
/// Target sum that valid encodings must satisfy
pub const TARGET_SUM: u32 = (DIMENSION as u32 * (BASE - 1)) / 2;

/// Encode a message using target-sum encoding.
///
/// Returns the encoded chunks if the sum equals TARGET_SUM,
/// otherwise returns an error.
pub fn encode(
    epoch: u32,
    randomness: &[u8],
    message: &[u8; MESSAGE_LENGTH],
) -> Result<Vec<u8>, EncodingError> {
    // Generate chunks using SHAKE128
    let chunks = generate_chunks(epoch, randomness, message);

    // Verify target sum
    let sum: u32 = chunks.iter().map(|&c| c as u32).sum();

    if sum == TARGET_SUM {
        Ok(chunks)
    } else {
        Err(EncodingError::SumMismatch { expected: TARGET_SUM, actual: sum })
    }
}

/// Generate chunks using SHAKE128-based message hash.
fn generate_chunks(
    epoch: u32,
    randomness: &[u8],
    message: &[u8; MESSAGE_LENGTH],
) -> Vec<u8> {
    let mut hasher = Shake128::default();

    // Domain separation
    hasher.update(&[0x00]); // Message hash tweak
    hasher.update(&epoch.to_le_bytes());
    hasher.update(randomness);
    hasher.update(message);

    // Extract DIMENSION bytes
    let mut chunks = vec![0u8; DIMENSION];
    let mut reader = hasher.finalize_xof();
    reader.read(&mut chunks);

    chunks
}

/// Error during encoding.
#[derive(Debug, Clone)]
pub enum EncodingError {
    SumMismatch { expected: u32, actual: u32 },
}
