use anchor_lang::prelude::*;

use crate::constants::BPS_DENOMINATOR;
use crate::errors::BingoError;

/// Calculate basis points (percentage) of an amount.
pub fn calculate_bps(amount: u64, bps: u16) -> Result<u64> {
    amount
        .checked_mul(bps as u64)
        .and_then(|v| v.checked_div(BPS_DENOMINATOR))
        .ok_or(BingoError::ArithmeticOverflow.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_bps() {
        assert_eq!(calculate_bps(1000, 2000).unwrap(), 200);
        assert_eq!(calculate_bps(1000, 500).unwrap(), 50);
        assert_eq!(calculate_bps(1000, 10000).unwrap(), 1000);
        assert_eq!(calculate_bps(1000, 0).unwrap(), 0);
    }
}
