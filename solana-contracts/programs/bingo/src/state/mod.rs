//! # Fundraisely State Management Module
//!
//! On-chain state structures for the Fundraisely smart contract.
//!
//! ## Overview
//!
//! This module defines the three core state structures that persist data on the Solana blockchain:
//!
//! 1. **GlobalConfig** (global_config.rs) - Platform-wide configuration (singleton)
//! 2. **Room** (room.rs) - Individual game room state (per-room PDA)
//! 3. **PlayerEntry** (player_entry.rs) - Player participation records (per-player-per-room PDA)
//!
//! ## Architecture Role
//!
//! State structures are stored as Program Derived Addresses (PDAs) on-chain, ensuring:
//!
//! - **Deterministic Addressing**: Accounts can be derived from seeds without additional storage
//! - **Security**: Only the program can sign transactions on behalf of PDAs
//! - **Cost Efficiency**: Rent-exempt accounts minimize ongoing costs
//! - **Data Integrity**: Anchor serialization/deserialization prevents data corruption
//!
//! ## PDA Derivation Patterns
//!
//! ```text
//! GlobalConfig PDA: ["global-config"]
//! Room PDA:         ["room", host_pubkey, room_id]
//! PlayerEntry PDA:  ["player", room_pubkey, player_pubkey]
//! Room Vault PDA:   ["room-vault", room_pubkey]
//! ```
//!
//! ## State Lifecycle
//!
//! ### 1. Initialization Phase
//! - Admin calls `initialize()` creating the GlobalConfig PDA
//! - Sets platform wallets, fee structures, and economic constraints
//! - One-time operation executed before any rooms can be created
//!
//! ### 2. Room Creation Phase
//! - Host calls `init_pool_room()` creating Room and RoomVault PDAs
//! - Room state: Ready (accepts players)
//! - Fee structure validated against GlobalConfig limits
//!
//! ### 3. Player Entry Phase
//! - Players call `join_room()` creating PlayerEntry PDAs
//! - Tokens transferred to RoomVault
//! - Room state: Active (first player triggers state change)
//! - Room counters updated (player_count, total_collected, etc.)
//!
//! ### 4. Game End Phase
//! - Host calls `end_room()` distributing funds from RoomVault
//! - Room state: Ended (immutable final state)
//! - Funds transferred to platform, host, charity, and winner wallets
//!
//! ## Frontend Integration
//!
//! The `useFundraiselyContract.ts` hook fetches and monitors these state accounts:
//!
//! ```typescript
//! // Fetch global configuration
//! const globalConfig = await program.account.globalConfig.fetch(globalConfigPDA);
//!
//! // Fetch room data
//! const room = await program.account.room.fetch(roomPDA);
//!
//! // Fetch player entry
//! const playerEntry = await program.account.playerEntry.fetch(playerEntryPDA);
//!
//! // Subscribe to account changes for real-time updates
//! program.account.room.subscribe(roomPDA, (room) => {
//!   updateUI(room);
//! });
//! ```
//!
//! ## Economic Model State Tracking
//!
//! State accounts maintain critical financial data for transparent accounting:
//!
//! - **Room.total_entry_fees**: Sum of all entry fees (split per economic model)
//! - **Room.total_extras_fees**: Sum of all extras (100% to charity)
//! - **Room.total_collected**: Grand total (entry + extras)
//! - **Room.charity_bps**: Calculated charity percentage (min 40%)
//! - **PlayerEntry.entry_paid**: Individual entry fee contribution
//! - **PlayerEntry.extras_paid**: Individual extras contribution
//!
//! This data enables:
//! - Transparent on-chain audit trails
//! - Real-time donation tracking
//! - Verifiable distribution calculations
//! - Post-game financial reconciliation
//!
//! ## Data Consistency
//!
//! All state mutations use Anchor's account constraint system:
//! - `#[account(mut)]`: Marks accounts that will be modified
//! - `#[account(seeds = [...], bump)]`: Validates PDA derivation
//! - `constraint = ...`: Custom validation rules
//!
//! This ensures state changes are atomic, validated, and impossible to forge.

pub mod global_config;
pub mod room;
pub mod player_entry;
pub mod token_registry;

pub use global_config::*;
pub use room::*;
pub use player_entry::*;
pub use token_registry::*;
