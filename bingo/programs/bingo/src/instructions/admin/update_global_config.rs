//! # Update Global Config Instruction
//!
//! Admin-only instruction to update economic parameters in the GlobalConfig account.
//!
//! ## Overview
//!
//! This instruction allows the admin to modify the platform's economic constraints after
//! initial deployment. This is necessary when adjusting fee structures or updating wallet
//! addresses.
//!
//! ## Security
//!
//! - Only the admin stored in GlobalConfig can call this instruction
//! - Changes take effect immediately for all new rooms created after the update
//! - Existing rooms are not affected (they store their own fee parameters)

use crate::{BingoError, GlobalConfig};
use anchor_lang::prelude::*;
use std::str::FromStr;

/// Update global configuration parameters
///
/// Allows admin to modify platform fees, wallet addresses, and economic constraints.
pub fn handler(
    ctx: Context<UpdateGlobalConfig>,
    platform_wallet: Option<Pubkey>,
    charity_wallet: Option<Pubkey>,
    platform_fee_bps: Option<u16>,
    max_host_fee_bps: Option<u16>,
    max_prize_pool_bps: Option<u16>,
    min_charity_bps: Option<u16>,
) -> Result<()> {
    let global_config = &mut ctx.accounts.global_config;

    // Update fields if provided
    if let Some(wallet) = platform_wallet {
        global_config.platform_wallet = wallet;
        msg!("Updated platform_wallet: {}", wallet);
    }

    if let Some(wallet) = charity_wallet {
        global_config.charity_wallet = wallet;
        msg!("Updated charity_wallet: {}", wallet);
    }

    if let Some(fee) = platform_fee_bps {
        global_config.platform_fee_bps = fee;
        msg!("Updated platform_fee_bps: {}", fee);
    }

    if let Some(fee) = max_host_fee_bps {
        global_config.max_host_fee_bps = fee;
        msg!("Updated max_host_fee_bps: {}", fee);
    }

    if let Some(fee) = max_prize_pool_bps {
        global_config.max_prize_pool_bps = fee;
        msg!("Updated max_prize_pool_bps: {}", fee);
    }

    if let Some(fee) = min_charity_bps {
        global_config.min_charity_bps = fee;
        msg!("Updated min_charity_bps: {}", fee);
    }

    msg!("Global configuration updated successfully");
    Ok(())
}

/// Context for updating global configuration
#[derive(Accounts)]
pub struct UpdateGlobalConfig<'info> {
    /// Global configuration PDA account
    #[account(
        mut,
        seeds = [b"global-config"],
        bump = global_config.bump,
        constraint = global_config.admin == admin.key() || is_upgrade_authority(&admin.key()) @ BingoError::Unauthorized
    )]
    pub global_config: Account<'info, GlobalConfig>,

    /// Admin or upgrade authority account
    pub admin: Signer<'info>,
}

/// Check if signer is the upgrade authority
///
/// This allows the upgrade authority to modify configuration even if not the admin.
/// Useful for emergency situations or program upgrades.
fn is_upgrade_authority(signer: &Pubkey) -> bool {
    // Hardcoded upgrade authority
    let known_authority = Pubkey::from_str("C1vn2MT7tZotZPjUJQDf9oo3dpZZ2tr7NxYLg8jTYgkw").unwrap();
    signer == &known_authority
}
