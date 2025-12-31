//! TargetSum W=1 encoding for XMSS signatures.
//!
//! This module implements the message encoding scheme that converts
//! a 32-byte message into 155 binary codeword values (0 or 1).

use alloc::vec;
use alloc::vec::Vec;
use p3_field::{PrimeCharacteristicRing, PrimeField32, PrimeField64};

use crate::field::F;
use crate::poseidon::{poseidon_compress, poseidon2_24, MERGE_COMPRESSION_WIDTH};
use crate::tweak_hash::TWEAK_LEN;
use crate::types::{
    EncodingRandomness, Parameter,
    PARAMETER_LEN, MSG_HASH_LEN, MSG_LEN_FE, NUM_CHAINS, BASE,
    RANDOMNESS_LEN,
};

/// Tweak separator for message hash
const TWEAK_SEPARATOR_FOR_MESSAGE_HASH: u8 = 0x02;

/// Compute the codeword from message, parameters, epoch, and randomness.
///
/// Returns NUM_CHAINS (155) binary values (0 or 1).
pub fn compute_codeword(
    parameter: &Parameter,
    epoch: u32,
    rho: &EncodingRandomness,
    message: &[u8; 32],
) -> Vec<u8> {
    // Step 1: Compute message hash using Poseidon2
    let msg_hash = compute_message_hash(parameter, epoch, rho, message);

    // Step 2: Convert hash to big integer and encode to binary
    encode_to_binary(&msg_hash)
}

/// Compute message hash using Poseidon2 compression.
///
/// Input: parameter || tweak || rho || message_as_field_elements
/// Output: MSG_HASH_LEN field elements
fn compute_message_hash(
    parameter: &Parameter,
    epoch: u32,
    rho: &EncodingRandomness,
    message: &[u8; 32],
) -> [F; MSG_HASH_LEN] {
    // Build tweak for message hash
    let tweak = build_message_tweak(epoch);

    // Convert message bytes to field elements
    let msg_fe = bytes_to_field_elements(message);

    // Build input: parameter || tweak || rho || msg_fe
    let mut input = Vec::with_capacity(PARAMETER_LEN + TWEAK_LEN + RANDOMNESS_LEN + MSG_LEN_FE);
    input.extend_from_slice(parameter.inner());
    input.extend_from_slice(&tweak);
    input.extend_from_slice(rho.inner());
    input.extend_from_slice(&msg_fe);

    // Apply Poseidon2 compression
    let perm = poseidon2_24();
    poseidon_compress::<F, _, MERGE_COMPRESSION_WIDTH, MSG_HASH_LEN>(&perm, &input)
}

/// Build tweak for message hash operation.
fn build_message_tweak(epoch: u32) -> [F; TWEAK_LEN] {
    // Message tweak: epoch in upper bits, separator in lower bits
    let acc: u128 = ((epoch as u128) << 8) | (TWEAK_SEPARATOR_FOR_MESSAGE_HASH as u128);

    // Base-p decomposition
    let p = F::ORDER_U64 as u128;
    let mut val = acc;
    let mut out = [F::ZERO; TWEAK_LEN];
    for digit in &mut out {
        let d = (val % p) as u64;
        val /= p;
        *digit = F::from_u64(d);
    }
    out
}

/// Convert 32 message bytes to field elements.
fn bytes_to_field_elements(message: &[u8; 32]) -> [F; MSG_LEN_FE] {
    let p = F::ORDER_U64 as u128;

    // Interpret message as big-endian integer
    let mut acc: u128 = 0;
    for &byte in message.iter().take(16) {
        acc = (acc << 8) | (byte as u128);
    }

    // First half -> field elements
    let mut result = [F::ZERO; MSG_LEN_FE];
    let half = MSG_LEN_FE / 2 + 1; // 5 elements for first 128 bits
    for i in 0..half.min(MSG_LEN_FE) {
        let d = (acc % p) as u64;
        acc /= p;
        result[i] = F::from_u64(d);
    }

    // Second half of message
    acc = 0;
    for &byte in message.iter().skip(16) {
        acc = (acc << 8) | (byte as u128);
    }

    // Remaining elements
    for i in half..MSG_LEN_FE {
        let d = (acc % p) as u64;
        acc /= p;
        result[i] = F::from_u64(d);
    }

    result
}

/// Encode message hash to binary codeword (TargetSum W=1).
///
/// Uses the simple arbitrary-precision integer arithmetic to convert
/// MSG_HASH_LEN field elements into NUM_CHAINS binary values.
fn encode_to_binary(hash: &[F; MSG_HASH_LEN]) -> Vec<u8> {
    let p = F::ORDER_U64;

    // Convert field elements to a big integer representation
    // Using SmallBigUint for arbitrary precision
    let mut big_int = SmallBigUint::zero();

    // Reconstruct big integer from hash (little-endian in field element array)
    for i in (0..MSG_HASH_LEN).rev() {
        big_int.mul_scalar(p);
        big_int.add_scalar(hash[i].as_canonical_u32() as u64);
    }

    // Extract NUM_CHAINS binary digits
    let mut codeword = Vec::with_capacity(NUM_CHAINS);
    for _ in 0..NUM_CHAINS {
        let digit = big_int.mod_scalar(BASE as u64) as u8;
        codeword.push(digit);
        big_int.div_scalar(BASE as u64);
    }

    codeword
}

/// Simple arbitrary-precision unsigned integer for encoding/decoding.
///
/// Uses a vector of u64 limbs in little-endian order.
struct SmallBigUint {
    limbs: Vec<u64>,
}

impl SmallBigUint {
    fn zero() -> Self {
        Self { limbs: vec![0] }
    }

    fn add_scalar(&mut self, val: u64) {
        let mut carry = val as u128;
        for limb in &mut self.limbs {
            carry += *limb as u128;
            *limb = carry as u64;
            carry >>= 64;
        }
        if carry > 0 {
            self.limbs.push(carry as u64);
        }
    }

    fn mul_scalar(&mut self, val: u64) {
        let mut carry: u128 = 0;
        for limb in &mut self.limbs {
            let prod = (*limb as u128) * (val as u128) + carry;
            *limb = prod as u64;
            carry = prod >> 64;
        }
        if carry > 0 {
            self.limbs.push(carry as u64);
        }
    }

    fn mod_scalar(&self, divisor: u64) -> u64 {
        let mut remainder: u128 = 0;
        for &limb in self.limbs.iter().rev() {
            remainder = (remainder << 64) | (limb as u128);
            remainder %= divisor as u128;
        }
        remainder as u64
    }

    fn div_scalar(&mut self, divisor: u64) {
        let mut remainder: u128 = 0;
        for limb in self.limbs.iter_mut().rev() {
            let dividend = (remainder << 64) | (*limb as u128);
            *limb = (dividend / (divisor as u128)) as u64;
            remainder = dividend % (divisor as u128);
        }
        // Trim leading zeros
        while self.limbs.len() > 1 && self.limbs.last() == Some(&0) {
            self.limbs.pop();
        }
    }
}

