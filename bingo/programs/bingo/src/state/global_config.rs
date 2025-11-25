//! # Global Configuration State
//!
//! Platform-wide settings and economic parameters for the Fundraisely smart contract.
//!
//! ## Overview
//!
//! GlobalConfig is a singleton Program Derived Address (PDA) that stores immutable platform
//! settings and economic constraints. This account is created once during program initialization
//! and defines the rules that all rooms must follow.
//!
//! ## Architecture Role
//!
//! GlobalConfig serves as the source of truth for:
//!
//! 1. **Administrative Control**: Defines who can update platform settings
//! 2. **Wallet Routing**: Specifies where platform fees and charity donations are sent
//! 3. **Economic Boundaries**: Sets maximum/minimum values for fee allocations
//! 4. **Emergency Controls**: Provides circuit breaker for security incidents
//!
//! ## PDA Derivation
//!
//! ```text
//! Seeds: ["global-config"]
//! Bump: Stored in GlobalConfig.bump
//! ```
//!
//! This deterministic address allows any client to derive the GlobalConfig account without
//! additional lookups or storage.
//!
//! ## Economic Model Enforcement
//!
//! GlobalConfig enforces the platform's economic constraints through validation at room creation:
//!
//! ### Fixed Platform Fee
//! - **platform_fee_bps**: 2000 (20% of the total pool)
//! - Non-negotiable infrastructure and development cost
//! - Applied to the combined total of entry fees + extras
//!
//! ### Configurable Limits
//! - **max_host_fee_bps**: 500 (5% maximum)
//!   - Incentivizes room creation while preventing excessive host compensation
//!   - Host chooses fee between 0-5% at room creation
//!
//! - **max_prize_pool_bps**: Not used for validation anymore
//!   - Prize pool is validated by: prize_pool_bps > 0 AND host_fee + prize_pool ≤ 40%
//!   - Host decides how to split the 40%: 0-5% for themselves, remainder for prizes
//!   - This allows up to 40% for prizes if host takes 0%
//!
//! - **min_charity_bps**: 4000 (40% minimum)
//!   - Ensures substantial portion of entry fees benefit charitable causes
//!   - Calculated as remainder: charity = 100% - platform - host - prizes
//!   - Validation enforced in init_pool_room instruction
//!
//! ### Critical Constraint
//! ```text
//! platform_fee_bps + host_fee_bps + prize_pool_bps + charity_bps = 10000 (100%)
//!
//! Where:
//!   platform_fee_bps = 2000 (fixed)
//!   host_fee_bps = 0-500 (host choice)
//!   prize_pool_bps > 0 (host choice, must be positive)
//!   charity_bps >= 4000 (calculated remainder)
//!   host_fee_bps + prize_pool_bps <= 4000 (40% combined max)
//! ```
//!
//! ## Frontend Integration
//!
//! The `useFundraiselyContract.ts` hook fetches GlobalConfig to:
//!
//! 1. **Display Fee Structure**: Show users where their entry fees go
//! 2. **Validate Room Creation**: Ensure host inputs meet platform constraints
//! 3. **Calculate Charity Amount**: Preview charity allocation before room creation
//! 4. **Check Emergency Status**: Disable operations if emergency_pause is true
//!
//! Example frontend usage:
//! ```typescript
//! const [globalConfigPDA] = PublicKey.findProgramAddressSync(
//!   [Buffer.from("global-config")],
//!   program.programId
//! );
//!
//! const globalConfig = await program.account.globalConfig.fetch(globalConfigPDA);
//!
//! // Validate host's desired fee structure
//! if (hostFeeBps > globalConfig.maxHostFeeBps) {
//!   alert(`Host fee cannot exceed ${globalConfig.maxHostFeeBps / 100}%`);
//! }
//!
//! // Calculate charity allocation
//! const charityBps = 10000 - globalConfig.platformFeeBps - hostFeeBps - prizePoolBps;
//! if (charityBps < globalConfig.minCharityBps) {
//!   alert(`Charity must receive at least ${globalConfig.minCharityBps / 100}%`);
//! }
//! ```
//!
//! ## Wallet Configuration
//!
//! - **platform_wallet**: Receives the platform percentage of every collected token
//! - **charity_wallet**: Receives the remainder after platform/host/prize allocations
//!
//! Both wallets must have associated token accounts for the fee_token_mint used in each room.
//!
//! ## Emergency Controls
//!
//! - **emergency_pause**: Boolean flag to halt all contract operations
//! - When true, all instructions (except administrative updates) fail immediately
//! - Allows admin to respond to critical vulnerabilities or exploits
//! - Frontend checks this flag before submitting transactions
//!
//! ## Security Considerations
//!
//! - **Admin Authority**: Only admin can modify GlobalConfig values
//! - **Immutable Economic Rules**: After initialization, economic constraints are fixed
//! - **Single Source of Truth**: All rooms reference this singleton for validation
//! - **PDA Security**: Only the program can sign transactions using this account

use anchor_lang::prelude::*;

/// Platform-wide configuration and economic parameters
///
/// This singleton PDA defines the economic constraints and wallet routing
/// for all fundraising rooms. Created once during program initialization.
#[account]
#[derive(Debug)]
pub struct GlobalConfig {
    /// Admin public key (can update config)
    pub admin: Pubkey,

    /// Platform wallet (receives platform fees)
    pub platform_wallet: Pubkey,

    /// Charity wallet (receives charity donations)
    pub charity_wallet: Pubkey,

    /// Platform fee in basis points (2000 = 20%)
    pub platform_fee_bps: u16,

    /// Maximum host fee in basis points (500 = 5%)
    pub max_host_fee_bps: u16,

    /// Maximum prize pool in basis points (3500 = 35%)
    pub max_prize_pool_bps: u16,

    /// Minimum charity allocation in basis points (4000 = 40%)
    pub min_charity_bps: u16,

    /// Emergency pause flag
    pub emergency_pause: bool,

    /// PDA bump seed
    pub bump: u8,
}

impl GlobalConfig {
    pub const LEN: usize = 8 + // discriminator
        32 + // admin
        32 + // platform_wallet
        32 + // charity_wallet
        2 + // platform_fee_bps
        2 + // max_host_fee_bps
        2 + // max_prize_pool_bps
        2 + // min_charity_bps
        1 + // emergency_pause
        1; // bump
}
