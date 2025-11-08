//! # Remove Approved Token Instruction
//!
//! Removes a token mint from the allowlist.
//! Only the registry admin can call this.

use anchor_lang::prelude::*;
use crate::errors::BingoError;
use crate::events::TokenRemoved;

/// Remove a token from the approved list
pub fn handler(ctx: Context<crate::RemoveApprovedToken>, token_mint: Pubkey) -> Result<()> {
    let registry = &mut ctx.accounts.token_registry;

    // Check admin
    require!(
        ctx.accounts.admin.key() == registry.admin,
        BingoError::Unauthorized
    );

    // Find and remove token
    if let Some(index) = registry.approved_tokens.iter().position(|&t| t == token_mint) {
        registry.approved_tokens.remove(index);
        msg!("Token removed: {}", token_mint);
        msg!("   Remaining approved tokens: {}", registry.approved_tokens.len());

        // Emit event
        emit!(TokenRemoved {
            token_mint,
            admin: ctx.accounts.admin.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });
    } else {
        return Err(BingoError::TokenNotApproved.into());
    }

    Ok(())
}

// Note: Account struct is in lib.rs
