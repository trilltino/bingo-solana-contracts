use anchor_lang::prelude::*;

use crate::constants::{MAX_SAFE_AMOUNT, MIN_DUST_THRESHOLD};
use crate::errors::BingoError;

/// Amount validation utilities shared across instructions.
pub struct AmountValidator;

impl AmountValidator {
    /// Validate an amount is within safe bounds.
    pub fn validate_amount(amount: u64, allow_zero: bool) -> Result<()> {
        if !allow_zero {
            require!(amount > 0, BingoError::InvalidEntryFee);
        }

        require!(amount <= MAX_SAFE_AMOUNT, BingoError::ArithmeticOverflow);

        if !allow_zero && amount < MIN_DUST_THRESHOLD {
            msg!(
                "Warning: Amount {} is below dust threshold {}",
                amount,
                MIN_DUST_THRESHOLD
            );
        }

        Ok(())
    }

    /// Validate entry fee is reasonable.
    pub fn validate_entry_fee(entry_fee: u64) -> Result<()> {
        Self::validate_amount(entry_fee, false)
    }

    /// Validate extras amount (can be zero).
    pub fn validate_extras(extras: u64) -> Result<()> {
        Self::validate_amount(extras, true)
    }

    /// Validate total payment amount.
    pub fn validate_total_payment(entry_fee: u64, extras: u64) -> Result<u64> {
        Self::validate_entry_fee(entry_fee)?;
        Self::validate_extras(extras)?;

        let total = entry_fee
            .checked_add(extras)
            .ok_or(BingoError::ArithmeticOverflow)?;

        Self::validate_amount(total, false)?;

        Ok(total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amount_validator() {
        assert!(AmountValidator::validate_entry_fee(1_000_000).is_ok());
        assert!(AmountValidator::validate_extras(0).is_ok());
        assert!(AmountValidator::validate_extras(500_000).is_ok());

        assert!(AmountValidator::validate_entry_fee(0).is_err());
        assert!(AmountValidator::validate_amount(MAX_SAFE_AMOUNT + 1, false).is_err());
    }
}
