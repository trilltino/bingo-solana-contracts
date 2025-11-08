//! # Utility Functions
//!
//! Helper functions used across multiple instructions for common calculations.

use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use crate::errors::BingoError;

/// Validate that a token mint does not have freeze authority set
///
/// # Security
/// Tokens with freeze authority can be frozen at any time, preventing transfers.
/// For fundraising, this would brick the entire room's funds if the charity/prize
/// token accounts get frozen. We require tokens without freeze authority for safety.
///
/// # Arguments
/// * `mint` - The token mint account to validate
///
/// # Returns
/// Ok if freeze authority is None, Err otherwise
pub fn validate_no_freeze_authority(mint: &Account<Mint>) -> Result<()> {
    require!(
        mint.freeze_authority.is_none(),
        BingoError::TokenHasFreezeAuthority
    );
    Ok(())
}

/// Calculate basis points (percentage) of an amount
///
/// BPS (Basis Points) are 1/100th of a percent:
/// - 100 BPS = 1%
/// - 1000 BPS = 10%
/// - 10000 BPS = 100%
///
/// Used for fee calculations throughout the program.
///
/// # Arguments
/// * `amount` - The total amount to calculate percentage of
/// * `bps` - Basis points (0-10000)
///
/// # Returns
/// The calculated amount, or error if overflow occurs
///
/// # Example
/// ```
/// let platform_fee = calculate_bps(1000, 2000)?; // 20% of 1000 = 200
/// ```
pub fn calculate_bps(amount: u64, bps: u16) -> Result<u64> {
    const BPS_DENOMINATOR: u64 = 10000;
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
        // 20% of 1000 = 200
        assert_eq!(calculate_bps(1000, 2000).unwrap(), 200);

        // 5% of 1000 = 50
        assert_eq!(calculate_bps(1000, 500).unwrap(), 50);

        // 100% of 1000 = 1000
        assert_eq!(calculate_bps(1000, 10000).unwrap(), 1000);

        // 0% of 1000 = 0
        assert_eq!(calculate_bps(1000, 0).unwrap(), 0);
    }
}
