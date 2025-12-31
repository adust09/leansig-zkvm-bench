//! TargetSum W=1 encoding implementation using Poseidon2.
//!
//! This module implements the message encoding for TargetSum W=1:
//! - Hash message with Poseidon2 to get MSG_HASH_LEN field elements
//! - Decode hash output to NUM_CHAINS binary chunks (0 or 1)
//!
//! Unlike Winternitz which adds checksum chunks, TargetSum uses an
//! "incomparable" encoding where the sum of all positions is constrained.

use alloc::vec;
use alloc::vec::Vec;
use crate::{
    KoalaBear, F,
    NUM_CHAINS, BASE, MSG_HASH_LEN, MSG_LEN_FE,
    PARAMETER_LEN, RANDOMNESS_LEN, TWEAK_LEN,
    MESSAGE_LENGTH,
    poseidon::poseidon_compress_24,
};
use crate::koalabear::P;

/// Tweak separator for message hash
const TWEAK_SEPARATOR_FOR_MESSAGE_HASH: u8 = 0x02;

/// Input length for message hash: rho + parameter + tweak + message
const MESSAGE_HASH_INPUT_LEN: usize = RANDOMNESS_LEN + PARAMETER_LEN + TWEAK_LEN + MSG_LEN_FE;

/// Compute the TargetSum codeword (chain positions) for a message.
///
/// Returns NUM_CHAINS binary values (0 or 1) representing chain positions.
pub fn compute_codeword(
    parameter: &[F; PARAMETER_LEN],
    epoch: u32,
    randomness: &[F; RANDOMNESS_LEN],
    message: &[u8; MESSAGE_LENGTH],
) -> Vec<u8> {
    let hash = poseidon_message_hash(parameter, epoch, randomness, message);
    decode_to_chunks(&hash)
}

/// Compute Poseidon2-based message hash.
fn poseidon_message_hash(
    parameter: &[F; PARAMETER_LEN],
    epoch: u32,
    randomness: &[F; RANDOMNESS_LEN],
    message: &[u8; MESSAGE_LENGTH],
) -> [F; MSG_HASH_LEN] {
    let message_fe = encode_message(message);
    let epoch_fe = encode_epoch(epoch);

    let mut combined = [KoalaBear::ZERO; MESSAGE_HASH_INPUT_LEN];
    let mut idx = 0;

    // rho (randomness)
    for i in 0..RANDOMNESS_LEN {
        combined[idx] = randomness[i];
        idx += 1;
    }

    // parameter
    for i in 0..PARAMETER_LEN {
        combined[idx] = parameter[i];
        idx += 1;
    }

    // tweak (epoch with domain separator)
    for i in 0..TWEAK_LEN {
        combined[idx] = epoch_fe[i];
        idx += 1;
    }

    // message (as field elements)
    for i in 0..MSG_LEN_FE {
        combined[idx] = message_fe[i];
        idx += 1;
    }

    poseidon_compress_24::<MSG_HASH_LEN>(&combined)
}

/// Encode 32-byte message as MSG_LEN_FE field elements.
fn encode_message(message: &[u8; MESSAGE_LENGTH]) -> [F; MSG_LEN_FE] {
    let mut acc = SmallBigUint::from_le_bytes(message);
    let mut out = [KoalaBear::ZERO; MSG_LEN_FE];

    for digit in &mut out {
        let rem = acc.div_small(P);
        *digit = KoalaBear::new(rem);
    }

    out
}

/// Encode epoch with tweak separator as TWEAK_LEN field elements.
fn encode_epoch(epoch: u32) -> [F; TWEAK_LEN] {
    let value = ((epoch as u64) << 8) | (TWEAK_SEPARATOR_FOR_MESSAGE_HASH as u64);
    let mut acc = SmallBigUint::from_u64(value);
    let mut out = [KoalaBear::ZERO; TWEAK_LEN];

    for digit in &mut out {
        let rem = acc.div_small(P);
        *digit = KoalaBear::new(rem);
    }

    out
}

/// Decode MSG_HASH_LEN field elements to NUM_CHAINS binary chunks.
fn decode_to_chunks(fe: &[F; MSG_HASH_LEN]) -> Vec<u8> {
    // Reconstruct big integer from field elements (big-endian)
    let mut acc = SmallBigUint::zero();
    for element in fe {
        acc.mul_small(P);
        acc.add_small(element.value());
    }

    biguint_to_base(acc, BASE as u32, NUM_CHAINS)
}

/// Convert big integer to base-n representation with specified number of digits.
fn biguint_to_base(mut value: SmallBigUint, base: u32, digits: usize) -> Vec<u8> {
    let mut out = vec![0u8; digits];

    for slot in &mut out {
        if value.is_zero() {
            break;
        }
        *slot = value.div_small(base) as u8;
    }

    out
}

/// Simple arbitrary-precision unsigned integer for encoding/decoding.
#[derive(Clone, Debug)]
struct SmallBigUint {
    limbs: Vec<u32>,
}

impl SmallBigUint {
    fn zero() -> Self {
        Self { limbs: Vec::new() }
    }

    fn from_u64(value: u64) -> Self {
        let mut limbs = Vec::new();
        limbs.push(value as u32);
        let hi = (value >> 32) as u32;
        if hi != 0 {
            limbs.push(hi);
        }
        let mut out = Self { limbs };
        out.normalize();
        out
    }

    fn from_le_bytes(bytes: &[u8]) -> Self {
        let mut limbs = Vec::with_capacity((bytes.len() + 3) / 4);
        for chunk in bytes.chunks(4) {
            let mut buf = [0u8; 4];
            buf[..chunk.len()].copy_from_slice(chunk);
            limbs.push(u32::from_le_bytes(buf));
        }
        let mut out = Self { limbs };
        out.normalize();
        out
    }

    fn normalize(&mut self) {
        while matches!(self.limbs.last(), Some(0)) {
            self.limbs.pop();
        }
    }

    fn is_zero(&self) -> bool {
        self.limbs.is_empty()
    }

    fn mul_small(&mut self, mul: u32) {
        if mul == 0 || self.is_zero() {
            self.limbs.clear();
            return;
        }
        let mut carry: u64 = 0;
        for limb in &mut self.limbs {
            let prod = (*limb as u64) * (mul as u64) + carry;
            *limb = prod as u32;
            carry = prod >> 32;
        }
        if carry != 0 {
            self.limbs.push(carry as u32);
        }
    }

    fn add_small(&mut self, add: u32) {
        let mut carry = add as u64;
        for limb in &mut self.limbs {
            let sum = (*limb as u64) + carry;
            *limb = sum as u32;
            carry = sum >> 32;
            if carry == 0 {
                break;
            }
        }
        if carry != 0 {
            self.limbs.push(carry as u32);
        }
    }

    fn div_small(&mut self, divisor: u32) -> u32 {
        if divisor == 0 {
            return 0;
        }
        let mut rem: u64 = 0;
        for limb in self.limbs.iter_mut().rev() {
            let cur = (rem << 32) | (*limb as u64);
            let q = cur / divisor as u64;
            rem = cur % divisor as u64;
            *limb = q as u32;
        }
        self.normalize();
        rem as u32
    }
}
