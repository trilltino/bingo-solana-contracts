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

use anchor_lang::prelude::*;
use crate::events::GlobalConfigUpdated;

/// Update global configuration parameters
///
/// Allows admin to modify platform fees, wallet addresses, and economic constraints.
pub fn handler(
    ctx: Context<crate::UpdateGlobalConfig>,
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

    // Emit event for off-chain indexers
    emit!(GlobalConfigUpdated {
        admin: ctx.accounts.admin.key(),
        platform_wallet,
        charity_wallet,
        platform_fee_bps,
        max_host_fee_bps,
        max_prize_pool_bps,
        min_charity_bps,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

// Note: UpdateGlobalConfig struct moved to lib.rs for Anchor macro compatibility
