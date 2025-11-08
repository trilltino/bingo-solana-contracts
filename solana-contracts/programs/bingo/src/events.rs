//! # Event Definitions
//!
//! Events emitted by the Fundraisely program for off-chain indexing and real-time UI updates.
//!
//! ## Purpose
//!
//! Events enable:
//! - **Frontend Real-time Updates**: WebSocket listeners can update UI instantly when events fire
//! - **Historical Data**: Indexers can build complete transaction history
//! - **Analytics**: Track platform metrics (total raised, active rooms, player participation)
//! - **Audit Trail**: Immutable record of all platform activity
//!
//! ## Frontend Integration Example
//!
//! ```typescript
//! const listenerId = program.addEventListener("RoomCreated", (event, slot) => {
//!   console.log(`New room: ${event.roomId} by ${event.host}`);
//!   refreshRoomList();
//! });
//! ```

use anchor_lang::prelude::*;

/// Emitted when a new fundraising room is created
///
/// Allows frontends to display new rooms immediately and indexers to track all rooms.
#[event]
pub struct RoomCreated {
    /// PDA address of the created room
    pub room: Pubkey,

    /// Human-readable room identifier (max 32 chars)
    pub room_id: String,

    /// Host's wallet address
    pub host: Pubkey,

    /// Entry fee amount in token's base units
    pub entry_fee: u64,

    /// Maximum number of players allowed
    pub max_players: u32,

    /// Slot number when room expires (0 = no expiration)
    pub expiration_slot: u64,

    /// Unix timestamp of room creation
    pub timestamp: i64,
}

/// Emitted when a player joins a room
///
/// Enables real-time player count updates and participation tracking.
#[event]
pub struct PlayerJoined {
    /// Room PDA that was joined
    pub room: Pubkey,

    /// Player's wallet address
    pub player: Pubkey,

    /// Total amount paid (entry fee + extras)
    pub amount_paid: u64,

    /// Voluntary extra donation amount
    pub extras_paid: u64,

    /// Current number of players in room after this join
    pub player_count: u32,

    /// Unix timestamp of join
    pub timestamp: i64,
}

/// Emitted when winners are declared for a room
///
/// Separates winner declaration from fund distribution for transparency.
/// Must be called before end_room.
#[event]
pub struct WinnersDeclared {
    /// Room PDA for which winners were declared
    pub room: Pubkey,

    /// List of declared winners (1-10 winners)
    pub winners: Vec<Pubkey>,

    /// Unix timestamp of winner declaration
    pub timestamp: i64,
}

/// Emitted when a room ends and funds are distributed
///
/// Critical for verifying transparent fund distribution and charitable impact.
#[event]
pub struct RoomEnded {
    /// Room PDA that ended
    pub room: Pubkey,

    /// List of winner wallet addresses (1-3 winners)
    pub winners: Vec<Pubkey>,

    /// Amount sent to platform wallet
    pub platform_amount: u64,

    /// Amount sent to host wallet
    pub host_amount: u64,

    /// Amount sent to charity (calculated from total pool proportionally, including extras)
    pub charity_amount: u64,

    /// Total prize pool distributed to winners
    pub prize_amount: u64,

    /// Total number of players who participated
    pub total_players: u32,

    /// Unix timestamp of room end
    pub timestamp: i64,
}

/// Emitted when a prize asset is deposited into an asset-based room
///
/// Tracks deposit of pre-determined prizes for asset mode.
#[event]
pub struct PrizeAssetDeposited {
    /// Room PDA receiving the prize asset
    pub room: Pubkey,

    /// Index of the prize (0 = 1st place, 1 = 2nd place, etc.)
    pub prize_index: u8,

    /// Token mint address of the deposited asset
    pub token_mint: Pubkey,

    /// Amount of tokens deposited
    pub amount: u64,

    /// Depositor's wallet address
    pub depositor: Pubkey,

    /// Unix timestamp of deposit
    pub timestamp: i64,
}

/// Emitted when a prize asset is withdrawn (undeposited) from an asset-based room
///
/// Only host can withdraw before room starts.
#[event]
pub struct PrizeAssetWithdrawn {
    /// Room PDA from which asset was withdrawn
    pub room: Pubkey,

    /// Index of the prize that was withdrawn
    pub prize_index: u8,

    /// Token mint address of withdrawn asset
    pub token_mint: Pubkey,

    /// Amount of tokens withdrawn
    pub amount: u64,

    /// Recipient wallet (usually host)
    pub recipient: Pubkey,

    /// Unix timestamp of withdrawal
    pub timestamp: i64,
}

/// Emitted when a new asset-based room is created
///
/// Asset rooms use pre-deposited prizes instead of pool splitting.
#[event]
pub struct AssetRoomCreated {
    /// PDA address of the created asset room
    pub room: Pubkey,

    /// Human-readable room identifier
    pub room_id: String,

    /// Host's wallet address
    pub host: Pubkey,

    /// Entry fee amount
    pub entry_fee: u64,

    /// Number of expected prizes
    pub expected_prizes: u8,

    /// Unix timestamp of creation
    pub timestamp: i64,
}

/// Emitted when a token is approved by admin
///
/// Tracks which SPL tokens are allowed for entry fees.
#[event]
pub struct TokenApproved {
    /// Token mint that was approved
    pub token_mint: Pubkey,

    /// Admin who approved the token
    pub admin: Pubkey,

    /// Unix timestamp of approval
    pub timestamp: i64,
}

/// Emitted when a token is removed from approved list
///
/// Admin can revoke token approval if needed.
#[event]
pub struct TokenRemoved {
    /// Token mint that was removed
    pub token_mint: Pubkey,

    /// Admin who removed the token
    pub admin: Pubkey,

    /// Unix timestamp of removal
    pub timestamp: i64,
}

/// Emitted when a room is recovered by admin
///
/// Emergency recovery for stuck funds or abandoned rooms.
#[event]
pub struct RoomRecovered {
    /// Room PDA that was recovered
    pub room: Pubkey,

    /// Amount recovered from vault
    pub amount_recovered: u64,

    /// Recipient of recovered funds
    pub recipient: Pubkey,

    /// Admin who performed recovery
    pub admin: Pubkey,

    /// Unix timestamp of recovery
    pub timestamp: i64,
}

/// Emitted when emergency pause is toggled by admin
#[event]
pub struct EmergencyPauseToggled {
    /// New pause state
    pub paused: bool,

    /// Admin who toggled the pause
    pub admin: Pubkey,

    /// Slot when toggle occurred
    pub slot: u64,

    /// Unix timestamp
    pub timestamp: i64,
}

/// Emitted when host closes joining for a room
#[event]
pub struct JoiningClosed {
    /// Room PDA that was closed
    pub room: Pubkey,

    /// Room identifier
    pub room_id: String,

    /// Host who closed joining
    pub host: Pubkey,

    /// Current player count when closed
    pub player_count: u32,

    /// Unix timestamp
    pub timestamp: i64,
}

/// Emitted when a room is cleaned up and vaults closed
#[event]
pub struct RoomCleaned {
    /// Room PDA that was cleaned
    pub room: Pubkey,

    /// Room identifier
    pub room_id: String,

    /// Rent lamports reclaimed from vault closure
    pub rent_reclaimed: u64,

    /// Recipient of reclaimed rent
    pub recipient: Pubkey,

    /// Unix timestamp
    pub timestamp: i64,
}

/// Emitted when global configuration is updated by admin
#[event]
pub struct GlobalConfigUpdated {
    /// Admin who performed the update
    pub admin: Pubkey,

    /// New platform wallet (if updated)
    pub platform_wallet: Option<Pubkey>,

    /// New charity wallet (if updated)
    pub charity_wallet: Option<Pubkey>,

    /// New platform fee BPS (if updated)
    pub platform_fee_bps: Option<u16>,

    /// New max host fee BPS (if updated)
    pub max_host_fee_bps: Option<u16>,

    /// New max prize pool BPS (if updated)
    pub max_prize_pool_bps: Option<u16>,

    /// New min charity BPS (if updated)
    pub min_charity_bps: Option<u16>,

    /// Unix timestamp
    pub timestamp: i64,
}
