//! Field type definitions using Plonky3's KoalaBear field
//!
//! KoalaBear is a prime field with modulus p = 2^31 - 2^24 + 1

extern crate alloc;

use p3_field::PrimeCharacteristicRing;
use p3_koala_bear::KoalaBear;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The prime field element type used throughout leanSig
/// KoalaBear: p = 2^31 - 2^24 + 1 = 2130706433
pub type F = KoalaBear;

/// Array of field elements - used for parameters, hashes, etc.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldArray<const N: usize>(pub [F; N]);

impl<const N: usize> FieldArray<N> {
    /// Create a new FieldArray from an array
    pub const fn new(arr: [F; N]) -> Self {
        Self(arr)
    }

    /// Get the inner array
    pub const fn inner(&self) -> &[F; N] {
        &self.0
    }

    /// Get iterator over field elements
    pub fn iter(&self) -> impl Iterator<Item = &F> {
        self.0.iter()
    }

    /// Get mutable iterator over field elements
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut F> {
        self.0.iter_mut()
    }
}

impl<const N: usize> Default for FieldArray<N> {
    fn default() -> Self {
        Self([F::ZERO; N])
    }
}

impl<const N: usize> AsRef<[F]> for FieldArray<N> {
    fn as_ref(&self) -> &[F] {
        &self.0
    }
}

impl<const N: usize> From<[F; N]> for FieldArray<N> {
    fn from(arr: [F; N]) -> Self {
        Self(arr)
    }
}

// Serialize FieldArray as array of u32 values (canonical representation)
impl<const N: usize> Serialize for FieldArray<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(N))?;
        for elem in &self.0 {
            // Convert field element to u32 canonical form
            seq.serialize_element(&elem.as_canonical_u32())?;
        }
        seq.end()
    }
}

// Deserialize FieldArray from array of u32 values
impl<'de, const N: usize> Deserialize<'de> for FieldArray<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use alloc::vec::Vec;
        use serde::de::Error;

        let values: Vec<u32> = Vec::deserialize(deserializer)?;
        if values.len() != N {
            return Err(D::Error::custom(alloc::format!(
                "expected {} elements, got {}",
                N,
                values.len()
            )));
        }

        let mut arr = [F::ZERO; N];
        for (i, &val) in values.iter().enumerate() {
            // Use MontyField31::new() for canonical representation
            arr[i] = F::new(val);
        }
        Ok(FieldArray(arr))
    }
}

// Constants used in leanSig
pub const TWEAK_SEPARATOR_FOR_MESSAGE_HASH: u8 = 0x02;
pub const TWEAK_SEPARATOR_FOR_TREE_HASH: u8 = 0x01;
pub const TWEAK_SEPARATOR_FOR_CHAIN_HASH: u8 = 0x00;

/// Message length in bytes for signatures
pub const MESSAGE_LENGTH: usize = 32;

// Re-export field trait for use
pub use p3_field::{Field, PrimeField32, PrimeField64};
