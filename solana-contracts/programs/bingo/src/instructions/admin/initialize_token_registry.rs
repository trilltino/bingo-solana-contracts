//! # Initialize Token Registry Instruction
//!
//! One-time setup of the token registry PDA.
//! Creates the registry and sets the admin who can modify it.

use anchor_lang::prelude::*;

/// Initialize the token registry (one-time setup)
pub fn handler(ctx: Context<crate::InitializeTokenRegistry>) -> Result<()> {
    let registry = &mut ctx.accounts.token_registry;
    registry.admin = ctx.accounts.admin.key();
    registry.approved_tokens = Vec::new();
    registry.bump = ctx.bumps.token_registry;

    msg!("Token registry initialized");
    msg!("   Admin: {}", registry.admin);

    Ok(())
}

// Note: Account struct is in lib.rs for Anchor compatibility
