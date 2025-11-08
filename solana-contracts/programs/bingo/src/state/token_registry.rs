//! # Token Registry State
//!
//! On-chain allowlist of approved SPL tokens for entry fees.
//!
//! ## Purpose
//!
//! Prevents spam/scam tokens from being used in fundraising rooms.
//! Only tokens explicitly approved by platform admin can be used as fee_token_mint.
//!
//! ## PDA Derivation
//!
//! Seeds: ["token-registry"]
//! Bump: Stored in TokenRegistry.bump
//!
//! ## Admin Operations
//!
//! - initialize_token_registry: One-time setup (creates PDA)
//! - add_approved_token: Add token to allowlist
//! - remove_approved_token: Remove token from allowlist
//!
//! ## Validation
//!
//! init_pool_room checks that fee_token_mint exists in approved_tokens Vec

use anchor_lang::prelude::*;
use crate::constants::MAX_APPROVED_TOKENS;

/// Token registry containing allowlist of approved SPL tokens
#[account]
#[derive(Debug)]
pub struct TokenRegistry {
    /// Admin who can modify the registry
    pub admin: Pubkey,

    /// List of approved token mints
    pub approved_tokens: Vec<Pubkey>,

    /// PDA bump seed
    pub bump: u8,
}

impl TokenRegistry {
    /// Maximum number of approved tokens (prevents unbounded growth)
    pub const MAX_TOKENS: usize = MAX_APPROVED_TOKENS;

    /// Account size calculation
    pub const LEN: usize = 8 + // discriminator
        32 + // admin
        (4 + 32 * Self::MAX_TOKENS) + // approved_tokens Vec
        1; // bump

    /// Check if a token is approved
    pub fn is_token_approved(&self, token_mint: &Pubkey) -> bool {
        self.approved_tokens.contains(token_mint)
    }
}
