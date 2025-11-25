//! # Add Approved Token Instruction
//!
//! This instruction allows the registry admin to add a new SPL token mint to the global allowlist.
//! Rooms can only be created with tokens that exist in this registry. This provides centralized
//! control over which tokens are acceptable for entry fees and prizes, preventing spam tokens
//! or malicious mints from being used in the platform.

use crate::events::TokenApproved;
use crate::{errors::BingoError, TokenRegistry};
use anchor_lang::prelude::*;

/// Add a token to the approved list
pub fn handler(ctx: Context<AddApprovedToken>, token_mint: Pubkey) -> Result<()> {
    let registry = &mut ctx.accounts.token_registry;

    // Check admin
    require!(
        ctx.accounts.admin.key() == registry.admin,
        BingoError::Unauthorized
    );

    // Check if already approved
    require!(
        !registry.is_token_approved(&token_mint),
        BingoError::TokenAlreadyApproved
    );

    // Check capacity
    require!(
        registry.approved_tokens.len() < TokenRegistry::MAX_TOKENS,
        BingoError::TokenRegistryFull
    );

    // Add token
    registry.approved_tokens.push(token_mint);

    msg!("Token approved: {}", token_mint);
    msg!("Total approved tokens: {}", registry.approved_tokens.len());

    // Emit event
    emit!(TokenApproved {
        token_mint,
        admin: ctx.accounts.admin.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

/// Context for adding approved token
#[derive(Accounts)]
pub struct AddApprovedToken<'info> {
    /// Token registry PDA account
    #[account(
        mut,
        seeds = [b"token-registry-v4"],
        bump = token_registry.bump
    )]
    pub token_registry: Account<'info, TokenRegistry>,

    /// Admin account adding the token
    #[account(mut)]
    pub admin: Signer<'info>,
}
