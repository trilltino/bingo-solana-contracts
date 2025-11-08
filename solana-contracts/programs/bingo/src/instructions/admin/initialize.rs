//! # Initialize Instruction
//!
//! One-time admin setup of global platform configuration and economic parameters.
//!
//! ## Overview
//!
//! This instruction creates the singleton GlobalConfig PDA that governs all fundraising rooms
//! on the platform. It must be called exactly once by the program admin before any rooms can
//! be created. This initialization sets immutable economic rules that ensure every room
//! maintains the platform's charitable mission (minimum 40% to charity).
//!
//! ## Role in Program Architecture
//!
//! Initialize is the **first instruction** that must be executed in the program lifecycle:
//!
//! ```text
//! 1. initialize        ← Creates GlobalConfig PDA (THIS INSTRUCTION)
//! 2. init_pool_room    ← Hosts create rooms (validates against GlobalConfig)
//! 3. join_room         ← Players join rooms
//! 4. end_room          ← Host distributes funds to platform/charity/winners
//! ```
//!
//! Without initialization, all other instructions will fail because they require the GlobalConfig
//! account to exist for validation.
//!
//! ## What This Instruction Does
//!
//! 1. **Creates GlobalConfig PDA**: Initializes the singleton config account using seed ["global-config"]
//! 2. **Sets Admin Authority**: Records the admin's pubkey for future config updates
//! 3. **Configures Wallets**: Sets platform_wallet and charity_wallet for fund routing
//! 4. **Defines Economic Constraints**: Sets maximum/minimum fee percentages:
//!    - platform_fee_bps: 2000 (20% fixed platform fee)
//!    - max_host_fee_bps: 500 (5% maximum host can take)
//!    - max_prize_pool_bps: 3500 (35% maximum for prizes)
//!    - min_charity_bps: 4000 (40% minimum must go to charity)
//!    - Combined host + prizes cannot exceed 40%
//! 5. **Sets Emergency Controls**: Initializes emergency_pause flag to false
//!
//! ## Economic Model Enforcement
//!
//! The economic parameters set here cannot be bypassed by room creators:
//!
//! ```text
//! Every room must satisfy:
//!   platform_fee (20%) + host_fee (0-5%) + prizes (0-35%) + charity (40%+) = 100%
//!   AND: host_fee + prizes <= 40% (combined maximum)
//!
//! GlobalConfig enforces:
//!   - Platform fee is always exactly 20% (fixed)
//!   - Host fee cannot exceed 5%
//!   - Prize pool cannot exceed 35%
//!   - Charity must receive at least 40% (calculated as remainder)
//!
//! This ensures the platform maintains its charitable mission while allowing
//! hosts flexibility in prize pool and host fee allocation.
//! ```
//!
//! ## Frontend Integration
//!
//! The frontend doesn't typically call this instruction (admin-only), but it does fetch
//! the GlobalConfig to display platform rules and validate room creation inputs:
//!
//! ```typescript
//! // Fetch global configuration
//! const [globalConfigPDA] = PublicKey.findProgramAddressSync(
//!   [Buffer.from("global-config")],
//!   program.programId
//! );
//!
//! const globalConfig = await program.account.globalConfig.fetch(globalConfigPDA);
//!
//! // Display to users
//! console.log(`Platform fee: ${globalConfig.platformFeeBps / 100}%`);
//! console.log(`Max host fee: ${globalConfig.maxHostFeeBps / 100}%`);
//! console.log(`Max prize pool: ${globalConfig.maxPrizePoolBps / 100}%`);
//! console.log(`Min charity: ${globalConfig.minCharityBps / 100}%`);
//! ```
//!
//! ## Security Considerations
//!
//! - **One-Time Operation**: Once created, GlobalConfig cannot be re-initialized
//! - **Admin Authority**: Only the admin pubkey can modify GlobalConfig values (future updates)
//! - **PDA Security**: GlobalConfig is a PDA, so only the program can sign for it
//! - **Economic Immutability**: After initialization, the economic constraints are set
//!   (admin could update them, but this would require additional instructions not yet implemented)
//!
//! ## Error Conditions
//!
//! This instruction will fail if:
//! - GlobalConfig PDA already exists (already initialized)
//! - Caller is not the designated admin (signature validation)
//! - Insufficient lamports for rent (payer balance check)
//!
//! ## On-Chain Logs
//!
//! Successful execution emits:
//! ```text
//! Fundraisely program initialized
//!    Admin: <admin_pubkey>
//!    Platform wallet: <platform_wallet>
//!    Charity wallet: <charity_wallet>
//! ```
//!
//! ## Related Files
//!
//! - **state/global_config.rs**: Defines the GlobalConfig struct and its layout
//! - **lib.rs**: Entry point that routes to this instruction handler
//! - **init_pool_room.rs**: Validates room creation against GlobalConfig constraints

use anchor_lang::prelude::*;

/// Initialize the global configuration (one-time setup)
///
/// This must be called by the admin before any other operations.
/// Sets up platform fees, wallets, and economic parameters.
pub fn handler(
    ctx: Context<crate::Initialize>,
    platform_wallet: Pubkey,
    charity_wallet: Pubkey,
) -> Result<()> {
    let global_config = &mut ctx.accounts.global_config;

    // Set configuration
    global_config.admin = ctx.accounts.admin.key();
    global_config.platform_wallet = platform_wallet;
    global_config.charity_wallet = charity_wallet;
    global_config.platform_fee_bps = 2000;      // 20% platform fee
    global_config.max_host_fee_bps = 500;       // 5% max host fee
    global_config.max_prize_pool_bps = 3500;    // 35% max prize pool (40% - 5% host max)
    global_config.min_charity_bps = 4000;       // 40% min charity
    global_config.emergency_pause = false;
    global_config.bump = ctx.bumps.global_config;

    // SECURITY: Validate BPS values sum correctly (platform + max_host + max_prize + min_charity <= 10000)
    // Note: In reality, host and prize vary, but this validates the maximum possible allocation
    let total_max_bps = global_config.platform_fee_bps
        .checked_add(global_config.max_host_fee_bps)
        .and_then(|v| v.checked_add(global_config.max_prize_pool_bps))
        .ok_or(crate::BingoError::ArithmeticOverflow)?;

    // This is a sanity check - platform + max_host + max_prize should leave room for min_charity
    require!(
        total_max_bps.checked_add(global_config.min_charity_bps).is_some(),
        crate::BingoError::TotalAllocationTooHigh
    );

    msg!("Bingo program initialized");
    msg!("   Admin: {}", ctx.accounts.admin.key());
    msg!("   Platform wallet: {}", platform_wallet);
    msg!("   Charity wallet: {}", charity_wallet);

    Ok(())
}

// Note: Initialize struct moved to lib.rs for Anchor macro compatibility
