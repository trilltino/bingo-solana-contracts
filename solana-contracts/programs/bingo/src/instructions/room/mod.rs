//! # Room Instructions Module
//!
//! Instructions for creating and managing fundraising game rooms.
//!
//! This module contains operations related to room lifecycle: creation, configuration,
//! and closure. Rooms are the core fundraising units where hosts configure prizes,
//! fees, and charity destinations.
//!
//! ## Instructions
//!
//! - **init_pool_room**: Create new room with pool-based prize distribution
//!
//! ## Future Room Instructions
//!
//! - **init_asset_room**: Create room with pre-deposited prize assets (Phase 2)
//! - **deposit_prize_asset**: Add NFT/token prizes to asset room (Phase 2)
//! - **close_joining**: Stop accepting new players before max_players reached
//! - **update_expiration**: Extend or shorten room expiration time

pub mod init_pool_room;
pub mod close_joining;
pub mod cleanup_room;

// InitPoolRoom struct is now in lib.rs for Anchor macro compatibility
