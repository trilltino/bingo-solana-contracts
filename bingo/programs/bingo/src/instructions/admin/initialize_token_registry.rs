//! # Initialize Token Registry Instruction
//!
//! One-time setup of the token registry PDA.
//! Creates the registry and sets the admin who can modify it.

use crate::TokenRegistry;
use anchor_lang::prelude::*;

/// Initialize the token registry (one-time setup)
pub fn handler(ctx: Context<InitializeTokenRegistry>) -> Result<()> {
    let registry = &mut ctx.accounts.token_registry;
    registry.admin = ctx.accounts.admin.key();
    registry.approved_tokens = Vec::new();
    registry.bump = ctx.bumps.token_registry;

    msg!("Token registry initialized");
    msg!("   Admin: {}", registry.admin);

    Ok(())
}

/// Context for initializing token registry
#[derive(Accounts)]
pub struct InitializeTokenRegistry<'info> {
    /// Token registry PDA account
    #[account(
        init,
        payer = admin,
        space = TokenRegistry::LEN,
        seeds = [b"token-registry-v4"],
        bump
    )]
    pub token_registry: Account<'info, TokenRegistry>,

    /// Admin account initializing the registry
    #[account(mut)]
    pub admin: Signer<'info>,

    /// System program for account creation
    pub system_program: Program<'info, System>,
}
