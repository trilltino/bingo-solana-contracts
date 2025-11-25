//! # Room State
//!
//! Individual game room state and economic tracking for the Fundraisely smart contract.
//!
//! ## Overview
//!
//! Room is a Program Derived Address (PDA) that represents a single fundraising game instance.
//! Each room maintains its own fee structure (within GlobalConfig limits), player roster,
//! financial accounting, and lifecycle state. Rooms are the primary unit of fundraising activity.
//!
//! ## Architecture Role
//!
//! Room accounts serve as:
//!
//! 1. **Game State Manager**: Tracks lifecycle (Ready → Active → Ended)
//! 2. **Financial Ledger**: Records all entry fees, extras, and distribution calculations
//! 3. **Configuration Store**: Defines fee splits, prize distribution, and timing
//! 4. **Security Boundary**: Each room has isolated vault for SPL token custody
//!
//! ## PDA Derivation
//!
//! ```text
//! Seeds: ["room", host_pubkey, room_id]
//! Bump: Stored in Room.bump
//! ```
//!
//! This allows deterministic room addresses based on host and room_id, preventing
//! duplicate rooms and enabling efficient lookups.
//!
//! ## Room Lifecycle
//!
//! ### 1. Ready State (Initial)
//! - Room created via `init_pool_room` instruction
//! - Accepts player entries
//! - Fee structure locked in at creation
//! - No players have joined yet
//!
//! ### 2. Active State (In Progress)
//! - First player joins via `join_room` instruction
//! - Subsequent players continue joining (up to max_players)
//! - Funds accumulate in room_vault
//! - Can be ended by host at any time (or by anyone after expiration)
//!
//! ### 3. Ended State (Final)
//! - Host calls `end_room` with winner list
//! - Funds distributed from vault to all recipients
//! - Room becomes immutable (no further state changes)
//! - Historical record maintained on-chain
//!
//! ## Economic Model Per Room
//!
//! Each room defines its own fee structure within platform constraints:
//!
//! ### Fee Allocation (Total Pool)
//! ```text
//! Platform Fee: 20% (fixed by GlobalConfig)
//! Host Fee:     host_fee_bps (0-5%, host chooses)
//! Prize Pool:   prize_pool_bps (0-35%, host chooses)
//! Charity:      charity_bps (calculated remainder, min 40%)
//! ```
//!
//! ### Distribution Calculation
//! ```rust
//! let total_pool = total_entry_fees + total_extras_fees;
//! let platform_fee = total_pool * 20 / 100;
//! let host_fee = total_pool * host_fee_bps / 10000;
//! let prize_amount = total_pool * prize_pool_bps / 10000;
//! let charity_amount = total_pool - platform_fee - host_fee - prize_amount;
//! ```
//!
//! ## Prize Distribution
//!
//! - **prize_mode**: Currently PoolSplit (prizes from collected fees)
//! - **prize_distribution**: Vec of percentages [1st, 2nd, 3rd] (must sum to 100)
//!
//! Example: [50, 30, 20] means:
//! - 1st place: 50% of prize_amount
//! - 2nd place: 30% of prize_amount
//! - 3rd place: 20% of prize_amount
//!
//! ## Room Expiration
//!
//! - **expiration_slot**: Solana slot when room expires (0 = no expiration)
//! - After expiration, anyone can end the room (not just host)
//! - Prevents abandoned rooms from locking funds indefinitely
//! - Typical expiration: ~43,200 slots (approximately 24 hours)
//!
//! ## Financial Tracking
//!
//! Room maintains three critical counters for transparent accounting:
//!
//! - **total_collected**: Sum of all funds received
//! - **total_entry_fees**: Sum of required entry fees (used for analytics)
//! - **total_extras_fees**: Sum of optional extras (tracked separately for reporting)
//!
//! These enable:
//! - On-chain audit trails
//! - Verifiable distribution calculations
//! - Real-time donation tracking for donors
//!
//! ## Frontend Integration
//!
//! The `useFundraiselyContract.ts` hook interacts with Room accounts:
//!
//! ### Room Creation
//! ```typescript
//! const [roomPDA] = PublicKey.findProgramAddressSync(
//!   [Buffer.from("room"), host.toBuffer(), Buffer.from(roomId)],
//!   program.programId
//! );
//!
//! await program.methods
//!   .initPoolRoom(
//!     roomId,
//!     entryFee,
//!     maxPlayers,
//!     hostFeeBps,
//!     prizePoolBps,
//!     firstPlacePct,
//!     secondPlacePct,
//!     thirdPlacePct,
//!     charityMemo,
//!     expirationSlots
//!   )
//!   .accounts({ room: roomPDA, ... })
//!   .rpc();
//! ```
//!
//! ### Room Monitoring
//! ```typescript
//! // Fetch current room state
//! const room = await program.account.room.fetch(roomPDA);
//!
//! // Subscribe to room changes
//! const subscription = program.account.room.subscribe(roomPDA, (room) => {
//!   updateUI({
//!     playerCount: room.playerCount,
//!     totalCollected: room.totalCollected,
//!     status: room.status,
//!   });
//! });
//! ```
//!
//! ### Ending Room
//! ```typescript
//! await program.methods
//!   .endRoom(roomId, [winner1, winner2, winner3])
//!   .accounts({ room: roomPDA, ... })
//!   .remainingAccounts([
//!     { pubkey: winner1TokenAccount, isSigner: false, isWritable: true },
//!     { pubkey: winner2TokenAccount, isSigner: false, isWritable: true },
//!     { pubkey: winner3TokenAccount, isSigner: false, isWritable: true },
//!   ])
//!   .rpc();
//! ```
//!
//! ## Security Considerations
//!
//! - **Host Authority**: Only host can end room (unless expired)
//! - **Host Cannot Win**: Explicit check prevents host from awarding themselves prizes
//! - **Capacity Limits**: max_players prevents unbounded state growth
//! - **Immutable After End**: ended flag prevents double-distribution exploits
//! - **PDA Vault**: Room vault secured by program-controlled PDA
//!
//! ## Data Structure Layout
//!
//! Account size: 263 bytes
//! - Discriminator: 8 bytes
//! - Strings: 36 bytes (room_id) + 32 bytes (charity_memo)
//! - Pubkeys: 64 bytes (host, fee_token_mint)
//! - Amounts: 24 bytes (entry_fee, totals)
//! - Fees: 6 bytes (host_fee_bps, prize_pool_bps, charity_bps)
//! - Counters: 8 bytes (player_count, max_players)
//! - Enums/Flags: 3 bytes (prize_mode, status, ended)
//! - Timing: 16 bytes (creation_slot, expiration_slot)
//! - Prize distribution: 10 bytes (Vec<u16>)
//! - Bump: 1 byte

use anchor_lang::prelude::*;

/// Asset prize information for asset-based rooms
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub struct PrizeAsset {
    /// Token mint for this prize
    pub mint: Pubkey,
    /// Amount of tokens for this prize
    pub amount: u64,
    /// Whether this prize has been deposited/escrowed
    pub deposited: bool,
}

/// Prize distribution mode
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum PrizeMode {
    /// Prizes allocated from percentage of collected entry fees
    PoolSplit,
    /// Pre-deposited prize assets
    AssetBased,
}

/// Room lifecycle state
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum RoomStatus {
    /// Asset room waiting for prize deposits (future feature)
    AwaitingFunding,
    /// Some prizes deposited (future feature)
    PartiallyFunded,
    /// Ready to accept players
    Ready,
    /// Players have joined, game is active
    Active,
    /// Game completed, funds distributed
    Ended,
}

/// Individual game room state and configuration
///
/// Tracks all financial and state information for a single fundraising game.
/// Created by hosts via init_pool_room, becomes immutable after ending.
#[account]
#[derive(Debug)]
pub struct Room {
    /// Unique room identifier (max 32 bytes)
    pub room_id: String,

    /// Host's public key
    pub host: Pubkey,

    /// Charity wallet address (per-room, from The Giving Block or custom)
    /// Receives the charity portion (remainder) of the total pool
    pub charity_wallet: Pubkey,

    /// Token mint for entry fees
    pub fee_token_mint: Pubkey,

    /// Entry fee amount in token base units
    pub entry_fee: u64,

    /// Host fee in basis points (0-500 = 0-5%)
    pub host_fee_bps: u16,

    /// Prize pool in basis points (0-4000 = 0-40%)
    pub prize_pool_bps: u16,

    /// Charity percentage in basis points (calculated)
    pub charity_bps: u16,

    /// Prize distribution mode
    pub prize_mode: PrizeMode,

    /// Prize distribution percentages [1st, 2nd, 3rd]
    pub prize_distribution: Vec<u16>,

    /// Room status
    pub status: RoomStatus,

    /// Number of players joined
    pub player_count: u32,

    /// Maximum number of players allowed
    pub max_players: u32,

    /// Total amount collected from all players
    pub total_collected: u64,

    /// Total from entry fees only
    pub total_entry_fees: u64,

    /// Total from extras payments
    pub total_extras_fees: u64,

    /// Whether new players can join (host can close joining early)
    pub joining_closed: bool,

    /// Game ended flag
    pub ended: bool,

    /// Slot when room was created
    pub creation_slot: u64,

    /// Slot when room expires (0 = no expiration)
    pub expiration_slot: u64,

    /// Charity memo for transfers
    pub charity_memo: String,

    /// Declared winners (1-10, set by declare_winners instruction)
    pub winners: Vec<Pubkey>,

    /// Prize assets for asset-based rooms (None for pool-based rooms)
    /// [1st place, 2nd place, 3rd place]
    pub prize_assets: [Option<PrizeAsset>; 3],

    /// PDA bump seed
    pub bump: u8,
}

impl Room {
    pub const LEN: usize = 8 + // discriminator
        (4 + 32) + // room_id (String)
        32 + // host
        32 + // charity_wallet
        32 + // fee_token_mint
        8 + // entry_fee
        2 + // host_fee_bps
        2 + // prize_pool_bps
        2 + // charity_bps
        1 + // prize_mode
        (4 + 3 * 2) + // prize_distribution (Vec<u16>)
        1 + // status
        4 + // player_count
        4 + // max_players
        8 + // total_collected
        8 + // total_entry_fees
        8 + // total_extras_fees
        1 + // joining_closed
        1 + // ended
        8 + // creation_slot
        8 + // expiration_slot
        (4 + 28) + // charity_memo (String)
        (3 * (1 + 32)) + // winners ([Option<Pubkey>; 3])
        (3 * (1 + 32 + 8 + 1)) + // prize_assets ([Option<PrizeAsset>; 3])
        1; // bump
}
