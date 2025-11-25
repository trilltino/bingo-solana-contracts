use anchor_lang::prelude::*;

use crate::errors::BingoError;

/// Checked math helpers that return Anchor `Result`.
pub struct ArithmeticGuard;

impl ArithmeticGuard {
    /// Safe addition with overflow check.
    pub fn checked_add(a: u64, b: u64) -> Result<u64> {
        a.checked_add(b)
            .ok_or(BingoError::ArithmeticOverflow.into())
    }

    /// Safe subtraction with underflow check.
    pub fn checked_sub(a: u64, b: u64) -> Result<u64> {
        a.checked_sub(b)
            .ok_or(BingoError::ArithmeticUnderflow.into())
    }

    /// Safe multiplication with overflow check.
    pub fn checked_mul(a: u64, b: u64) -> Result<u64> {
        a.checked_mul(b)
            .ok_or(BingoError::ArithmeticOverflow.into())
    }

    /// Safe division with zero check.
    pub fn checked_div(a: u64, b: u64) -> Result<u64> {
        require!(b > 0, BingoError::ArithmeticUnderflow);
        Ok(a.checked_div(b).unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arithmetic_guard() {
        assert_eq!(ArithmeticGuard::checked_add(100, 200).unwrap(), 300);
        assert_eq!(ArithmeticGuard::checked_sub(200, 100).unwrap(), 100);
        assert_eq!(ArithmeticGuard::checked_mul(100, 2).unwrap(), 200);
        assert_eq!(ArithmeticGuard::checked_div(200, 2).unwrap(), 100);

        assert!(ArithmeticGuard::checked_add(u64::MAX, 1).is_err());
        assert!(ArithmeticGuard::checked_sub(100, 200).is_err());
        assert!(ArithmeticGuard::checked_div(100, 0).is_err());
    }
}
