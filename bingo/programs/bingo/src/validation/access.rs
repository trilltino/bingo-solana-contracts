use anchor_lang::prelude::*;

use crate::errors::BingoError;

/// Access control helpers shared across admin/host guarded instructions.
pub struct AccessControl;

impl AccessControl {
    /// Verify admin authority.
    pub fn verify_admin(admin: &Pubkey, expected_admin: &Pubkey) -> Result<()> {
        require!(admin == expected_admin, BingoError::Unauthorized);
        Ok(())
    }

    /// Verify host authority.
    pub fn verify_host(host: &Pubkey, expected_host: &Pubkey) -> Result<()> {
        require!(host == expected_host, BingoError::Unauthorized);
        Ok(())
    }

    /// Verify host is not included in a list (prevents self-dealing).
    pub fn verify_host_not_in_list(host: &Pubkey, list: &[Pubkey]) -> Result<()> {
        require!(!list.contains(host), BingoError::HostCannotBeWinner);
        Ok(())
    }
}
