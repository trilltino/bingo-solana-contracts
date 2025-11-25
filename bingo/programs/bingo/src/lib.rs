//! # Fundraisely Smart Contract
//!
//! A trustless, on-chain fundraising platform built on Solana using the Anchor framework.
//!
//! ## Overview
//!
//! Fundraisely enables transparent, verifiable charitable fundraising through competitive game rooms.
//! All fund distribution logic executes on-chain, ensuring trustless operation with zero possibility
//! of fund misappropriation. Entry fees are automatically split between platform operations, hosts,
//! prize pools, and charitable causes according to immutable smart contract rules.
//!
//! ## Architecture
//!
//! This program implements five core instructions:
//!
//! 1. **initialize** - One-time setup of global configuration (platform wallets, fee structure)
//! 2. **init_pool_room** - Create a new fundraising game room with prize pool distribution
//! 3. **join_room** - Player entry with automatic fee collection and distribution
//! 4. **declare_winners** - Host declares 1-3 winners before fund distribution (transparency)
//! 5. **end_room** - Finalize game, distribute prizes, and transfer charity donations
//!
//! ## Economic Model (Trustless Distribution)
//!
//! Entry fees are automatically split via on-chain execution:
//! - **Platform Fee**: 20% (fixed) - Covers infrastructure and development
//! - **Host Fee**: 0-5% (configurable) - Incentivizes room creation
//! - **Prize Pool**: 0-35% (configurable) - Rewards to winners
//! - **Charity**: 40%+ minimum (calculated remainder) - Primary beneficiary
//!
//! **Critical Feature**: Optional "extras" payments (tips beyond entry fee) expand the
//! total pool and are split using the same immutable percentages, ensuring 100% of all
//! collected funds respect the configured allocation.
//!
//! ## Module Organization
//!
//! Following Anchor best practices, the codebase is organized into dedicated modules:
//!
//! - **state** - Account data structures (GlobalConfig, Room, PlayerEntry)
//! - **instructions** - Instruction handlers and contexts
//! - **errors** - Custom error definitions
//! - **events** - Event definitions for off-chain indexing
//!
//! ## Frontend Integration
//!
//! The frontend interacts with this program through the `useFundraiselyContract.ts` hook:
//!
//! - **Room Creation**: Host calls `createRoom()` which invokes `init_pool_room`
//! - **Joining Games**: Players call `joinRoom()` which invokes `join_room` with SPL token approval
//! - **Ending Games**: Host calls `endRoom()` which invokes `end_room` to distribute funds
//! - **Events**: Program emits RoomCreated, PlayerJoined, RoomEnded events for real-time UI updates
//!
//! ## Security Features
//!
//! - **PDA-based Accounts**: All accounts use Program Derived Addresses preventing signature forgery
//! - **Reentrancy Protection**: Checks-effects-interactions pattern in end_room
//! - **Token Validation**: Full mint and owner verification for prize distributions
//! - **Validation**: Strict checks on fee percentages, charity minimums, and winner eligibility
//! - **Emergency Pause**: Admin can halt operations if critical vulnerability discovered
//! - **Host Restrictions**: Hosts cannot be winners, preventing self-dealing
//! - **Arithmetic Safety**: All calculations use checked math to prevent overflow/underflow exploits

use anchor_lang::prelude::*;

// Module declarations
pub mod state;
pub mod errors;
pub mod events;
mod instructions;

// Re-export all types at crate root so Anchor macros can find them
pub use state::*;
pub use errors::*;
pub use events::*;

// Program ID - Updated with deployed program ID
declare_id!("AcUg72jFLTnKs478qaNpu2AWU3pBtqV1kmrkXaqjwjeK");

/// Fundraisely Program
///
/// All instruction handlers are implemented in the instructions module.
/// This keeps lib.rs clean and follows Anchor's recommended project structure.
#[program]
mod bingo {
    use super::*;

    ///Initialize the global configuration (one-time setup)
    pub fn initialize(
        ctx: Context<Initialize>,
        platform_wallet: Pubkey,
        charity_wallet: Pubkey,
    ) -> Result<()> {
        crate::instructions::admin::initialize::handler(ctx, platform_wallet, charity_wallet)
    }

    /// Create a pool-based room where prizes come from entry fee pool
    pub fn init_pool_room(
        ctx: Context<InitPoolRoom>,
        room_id: String,
        charity_wallet: Pubkey,
        entry_fee: u64,
        max_players: u32,
        host_fee_bps: u16,
        prize_pool_bps: u16,
        first_place_pct: u16,
        second_place_pct: Option<u16>,
        third_place_pct: Option<u16>,
        charity_memo: String,
        expiration_slots: Option<u64>,
    ) -> Result<()> {
        crate::instructions::room::init_pool_room::handler(
            ctx,
            room_id,
            charity_wallet,
            entry_fee,
            max_players,
            host_fee_bps,
            prize_pool_bps,
            first_place_pct,
            second_place_pct,
            third_place_pct,
            charity_memo,
            expiration_slots,
        )
    }

    /// Join a room by paying entry fee
    pub fn join_room(
        ctx: Context<JoinRoom>,
        room_id: String,
        extras_amount: u64,
    ) -> Result<()> {
        crate::instructions::player::join_room::handler(ctx, room_id, extras_amount)
    }

    /// Declare winners for a room (must be called before end_room)
    pub fn declare_winners<'info>(
        ctx: Context<'_, '_, '_, 'info, DeclareWinners<'info>>,
        room_id: String,
        winners: Vec<Pubkey>,
    ) -> Result<()> {
        crate::instructions::game::declare_winners::handler(ctx, room_id, winners)
    }

    /// End room and distribute prizes to winners
    pub fn end_room<'info>(
        ctx: Context<'_, '_, '_, 'info, EndRoom<'info>>,
        room_id: String,
        winners: Vec<Pubkey>,
    ) -> Result<()> {
        crate::instructions::game::end_room::handler(ctx, room_id, winners)
    }

    /// Initialize the token registry (one-time setup)
    pub fn initialize_token_registry(ctx: Context<InitializeTokenRegistry>) -> Result<()> {
        crate::instructions::admin::initialize_token_registry::handler(ctx)
    }

    /// Add a token to the approved list
    pub fn add_approved_token(ctx: Context<AddApprovedToken>, token_mint: Pubkey) -> Result<()> {
        crate::instructions::admin::add_approved_token::handler(ctx, token_mint)
    }

    /// Remove a token from the approved list
    pub fn remove_approved_token(ctx: Context<RemoveApprovedToken>, token_mint: Pubkey) -> Result<()> {
        crate::instructions::admin::remove_approved_token::handler(ctx, token_mint)
    }

    /// Initialize asset-based room
    pub fn init_asset_room(
        ctx: Context<InitAssetRoom>,
        room_id: String,
        charity_wallet: Pubkey,
        entry_fee: u64,
        max_players: u32,
        host_fee_bps: u16,
        charity_memo: String,
        expiration_slots: Option<u64>,
        prize_1_mint: Pubkey,
        prize_1_amount: u64,
        prize_2_mint: Option<Pubkey>,
        prize_2_amount: Option<u64>,
        prize_3_mint: Option<Pubkey>,
        prize_3_amount: Option<u64>,
    ) -> Result<()> {
        crate::instructions::asset::init_asset_room::handler(
            ctx,
            room_id,
            charity_wallet,
            entry_fee,
            max_players,
            host_fee_bps,
            charity_memo,
            expiration_slots,
            prize_1_mint,
            prize_1_amount,
            prize_2_mint,
            prize_2_amount,
            prize_3_mint,
            prize_3_amount,
        )
    }

    /// Add prize asset to asset-based room
    pub fn add_prize_asset(
        ctx: Context<AddPrizeAsset>,
        room_id: String,
        prize_index: u8,
    ) -> Result<()> {
        crate::instructions::asset::add_prize_asset::handler(ctx, room_id, prize_index)
    }

    /// Recover abandoned room (admin only)
    pub fn recover_room<'info>(
        ctx: Context<'_, '_, 'info, 'info, RecoverRoom<'info>>,
        room_id: String,
    ) -> Result<()> {
        crate::instructions::admin::recover_room::handler(ctx, room_id)
    }

    /// Set emergency pause (admin only)
    pub fn set_emergency_pause(
        ctx: Context<SetEmergencyPause>,
        paused: bool,
    ) -> Result<()> {
        crate::instructions::admin::set_emergency_pause::handler(ctx, paused)
    }

    /// Close joining for a room (host only)
    pub fn close_joining(
        ctx: Context<CloseJoining>,
        room_id: String,
    ) -> Result<()> {
        crate::instructions::room::close_joining::handler(ctx, room_id)
    }

    /// Cleanup room and reclaim rent (host or admin)
    pub fn cleanup_room(
        ctx: Context<CleanupRoom>,
        room_id: String,
    ) -> Result<()> {
        crate::instructions::room::cleanup_room::handler(ctx, room_id)
    }
}

// Account structures defined at crate root for Anchor macro compatibility
// The handlers are in separate modules, but Accounts structs must be here

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = admin,
        space = GlobalConfig::LEN,
        seeds = [b"global-config"],
        bump
    )]
    pub global_config: Account<'info, GlobalConfig>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(room_id: String)]
pub struct InitPoolRoom<'info> {
    #[account(
        init,
        payer = host,
        space = Room::LEN,
        seeds = [b"room", host.key().as_ref(), room_id.as_bytes()],
        bump
    )]
    pub room: Account<'info, Room>,

    /// CHECK: Room vault PDA - will be initialized as TokenAccount during room creation.
    /// SECURITY: Handler validates this is a proper TokenAccount with correct mint/authority.
    /// Using AccountInfo to avoid initialization order issues (room must exist before vault can use it as authority).
    /// The handler performs manual deserialization and validation of TokenAccount structure.
    #[account(
        mut,
        seeds = [b"room-vault", room.key().as_ref()],
        bump
    )]
    pub room_vault: AccountInfo<'info>,

    pub fee_token_mint: Account<'info, anchor_spl::token::Mint>,

    #[account(
        seeds = [b"token-registry-v2"],
        bump = token_registry.bump
    )]
    pub token_registry: Account<'info, TokenRegistry>,

    #[account(
        seeds = [b"global-config"],
        bump = global_config.bump
    )]
    pub global_config: Account<'info, GlobalConfig>,

    #[account(mut)]
    pub host: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, anchor_spl::token::Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(room_id: String)]
pub struct JoinRoom<'info> {
    #[account(
        mut,
        seeds = [b"room", room.host.as_ref(), room_id.as_bytes()],
        bump = room.bump
    )]
    pub room: Account<'info, Room>,

    #[account(
        init,
        payer = player,
        space = PlayerEntry::LEN,
        seeds = [b"player", room.key().as_ref(), player.key().as_ref()],
        bump
    )]
    pub player_entry: Account<'info, PlayerEntry>,

    /// Room vault token account - receives player's entry fee tokens.
    /// SECURITY: Properly typed as TokenAccount to ensure correct account validation.
    #[account(
        mut,
        seeds = [b"room-vault", room.key().as_ref()],
        bump
    )]
    pub room_vault: Account<'info, anchor_spl::token::TokenAccount>,

    #[account(mut)]
    pub player_token_account: Account<'info, anchor_spl::token::TokenAccount>,

    #[account(
        seeds = [b"global-config"],
        bump = global_config.bump
    )]
    pub global_config: Account<'info, GlobalConfig>,

    #[account(mut)]
    pub player: Signer<'info>,

    pub token_program: Program<'info, anchor_spl::token::Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(room_id: String)]
pub struct DeclareWinners<'info> {
    #[account(
        mut,
        seeds = [b"room", room.host.as_ref(), room_id.as_bytes()],
        bump = room.bump,
    )]
    pub room: Account<'info, Room>,

    #[account(mut)]
    pub host: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(room_id: String)]
pub struct EndRoom<'info> {
    #[account(
        mut,
        seeds = [b"room", host.key().as_ref(), room_id.as_bytes()],
        bump = room.bump,
    )]
    pub room: Account<'info, Room>,

    #[account(mut)]
    pub room_vault: Account<'info, anchor_spl::token::TokenAccount>,

    #[account(seeds = [b"global-config"], bump = global_config.bump)]
    pub global_config: Account<'info, GlobalConfig>,

    #[account(mut)]
    pub platform_token_account: Account<'info, anchor_spl::token::TokenAccount>,

    #[account(mut)]
    pub charity_token_account: Account<'info, anchor_spl::token::TokenAccount>,

    #[account(mut)]
    pub host_token_account: Account<'info, anchor_spl::token::TokenAccount>,

    #[account(mut)]
    pub host: Signer<'info>,

    pub token_program: Program<'info, anchor_spl::token::Token>,
}

#[derive(Accounts)]
pub struct InitializeTokenRegistry<'info> {
    #[account(
        init,
        payer = admin,
        space = TokenRegistry::LEN,
        seeds = [b"token-registry-v2"],
        bump
    )]
    pub token_registry: Account<'info, TokenRegistry>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddApprovedToken<'info> {
    #[account(
        mut,
        seeds = [b"token-registry-v2"],
        bump = token_registry.bump
    )]
    pub token_registry: Account<'info, TokenRegistry>,

    #[account(mut)]
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct RemoveApprovedToken<'info> {
    #[account(
        mut,
        seeds = [b"token-registry-v2"],
        bump = token_registry.bump
    )]
    pub token_registry: Account<'info, TokenRegistry>,

    #[account(mut)]
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(room_id: String)]
pub struct InitAssetRoom<'info> {
    #[account(
        init,
        payer = host,
        space = Room::LEN,
        seeds = [b"room", host.key().as_ref(), room_id.as_bytes()],
        bump
    )]
    pub room: Account<'info, Room>,

    /// CHECK: Room vault PDA
    #[account(
        seeds = [b"room-vault", room.key().as_ref()],
        bump
    )]
    pub room_vault: AccountInfo<'info>,

    pub fee_token_mint: Account<'info, anchor_spl::token::Mint>,

    #[account(
        seeds = [b"token-registry-v2"],
        bump = token_registry.bump
    )]
    pub token_registry: Account<'info, TokenRegistry>,

    #[account(
        seeds = [b"global-config"],
        bump = global_config.bump
    )]
    pub global_config: Account<'info, GlobalConfig>,

    #[account(mut)]
    pub host: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, anchor_spl::token::Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(room_id: String)]
pub struct AddPrizeAsset<'info> {
    #[account(
        mut,
        seeds = [b"room", room.host.as_ref(), room_id.as_bytes()],
        bump = room.bump
    )]
    pub room: Account<'info, Room>,

    #[account(mut)]
    pub prize_vault: Account<'info, anchor_spl::token::TokenAccount>,

    #[account(mut)]
    pub host_token_account: Account<'info, anchor_spl::token::TokenAccount>,

    #[account(mut)]
    pub host: Signer<'info>,

    pub token_program: Program<'info, anchor_spl::token::Token>,
}

#[derive(Accounts)]
#[instruction(room_id: String)]
pub struct RecoverRoom<'info> {
    #[account(
        mut,
        seeds = [b"room", room.host.as_ref(), room_id.as_bytes()],
        bump = room.bump
    )]
    pub room: Account<'info, Room>,

    #[account(
        mut,
        seeds = [b"room-vault", room.key().as_ref()],
        bump
    )]
    pub room_vault: Account<'info, anchor_spl::token::TokenAccount>,

    #[account(
        seeds = [b"global-config"],
        bump = global_config.bump
    )]
    pub global_config: Account<'info, GlobalConfig>,

    #[account(mut)]
    pub platform_token_account: Account<'info, anchor_spl::token::TokenAccount>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub token_program: Program<'info, anchor_spl::token::Token>,
}

#[derive(Accounts)]
pub struct SetEmergencyPause<'info> {
    #[account(
        mut,
        seeds = [b"global-config"],
        bump = global_config.bump
    )]
    pub global_config: Account<'info, GlobalConfig>,

    #[account(mut)]
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(room_id: String)]
pub struct CloseJoining<'info> {
    #[account(
        mut,
        seeds = [b"room", room.host.as_ref(), room_id.as_bytes()],
        bump = room.bump
    )]
    pub room: Account<'info, Room>,

    #[account(mut)]
    pub host: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(room_id: String)]
pub struct CleanupRoom<'info> {
    #[account(
        mut,
        seeds = [b"room", room.host.as_ref(), room_id.as_bytes()],
        bump = room.bump
    )]
    pub room: Account<'info, Room>,

    #[account(
        mut,
        seeds = [b"room-vault", room.key().as_ref()],
        bump
    )]
    pub room_vault: Account<'info, anchor_spl::token::TokenAccount>,

    #[account(
        seeds = [b"global-config"],
        bump = global_config.bump
    )]
    pub global_config: Account<'info, GlobalConfig>,

    #[account(mut)]
    pub caller: Signer<'info>,

    pub token_program: Program<'info, anchor_spl::token::Token>,
}
