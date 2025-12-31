//! Minimal KoalaBear field implementation for SP1.
//!
//! KoalaBear is a 31-bit prime field with p = 2^31 - 2^24 + 1 = 2130706433
//! This is a minimal implementation sufficient for leanSig verification.

use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use serde::{Deserialize, Serialize};

/// KoalaBear prime: p = 2^31 - 2^24 + 1 = 2130706433
pub const P: u32 = 2130706433;

/// KoalaBear field element.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KoalaBear(pub u32);

impl KoalaBear {
    /// Create a new field element.
    #[inline]
    pub const fn new(value: u32) -> Self {
        Self(value % P)
    }

    /// Get the canonical representative.
    #[inline]
    pub const fn value(self) -> u32 {
        self.0
    }

    /// Zero element.
    pub const ZERO: Self = Self(0);

    /// One element.
    pub const ONE: Self = Self(1);

    /// Compute modular inverse using Fermat's little theorem.
    /// a^(-1) = a^(p-2) mod p
    #[inline]
    pub fn inverse(self) -> Self {
        self.pow(P - 2)
    }

    /// Compute self^exp mod p using binary exponentiation.
    #[inline]
    pub fn pow(self, mut exp: u32) -> Self {
        let mut result = Self::ONE;
        let mut base = self;
        while exp > 0 {
            if exp & 1 == 1 {
                result = result * base;
            }
            base = base * base;
            exp >>= 1;
        }
        result
    }
}

impl Add for KoalaBear {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        let sum = self.0 as u64 + rhs.0 as u64;
        Self((sum % P as u64) as u32)
    }
}

impl AddAssign for KoalaBear {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for KoalaBear {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        let diff = if self.0 >= rhs.0 {
            self.0 - rhs.0
        } else {
            P - rhs.0 + self.0
        };
        Self(diff)
    }
}

impl SubAssign for KoalaBear {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Mul for KoalaBear {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        let prod = self.0 as u64 * rhs.0 as u64;
        Self((prod % P as u64) as u32)
    }
}

impl MulAssign for KoalaBear {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl Neg for KoalaBear {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        if self.0 == 0 {
            self
        } else {
            Self(P - self.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_arithmetic() {
        let a = KoalaBear::new(100);
        let b = KoalaBear::new(200);

        assert_eq!((a + b).value(), 300);
        assert_eq!((b - a).value(), 100);
        assert_eq!((a * b).value(), 20000);
    }

    #[test]
    fn test_modular_reduction() {
        let a = KoalaBear::new(P - 1);
        let b = KoalaBear::new(2);
        let sum = a + b;
        assert_eq!(sum.value(), 1);
    }

    #[test]
    fn test_inverse() {
        let a = KoalaBear::new(12345);
        let inv = a.inverse();
        let prod = a * inv;
        assert_eq!(prod.value(), 1);
    }
}
