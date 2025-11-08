//! # Player Instructions Module
//!
//! Instructions for player participation in fundraising rooms.
//!
//! This module contains operations that players execute to join rooms, pay entry fees,
//! and contribute to charitable causes. Player actions are the primary mechanism for
//! accumulating funds that will be distributed to winners and charities.
//!
//! ## Instructions
//!
//! - **join_room**: Pay entry fee + optional extras to join a room
//!
//! ## Future Player Instructions
//!
//! - **leave_room**: Exit room before game starts (if allowed by host)
//! - **ready_up**: Signal readiness to start game (for turn-based modes)
//! - **add_extras**: Contribute additional charity donation after joining

pub mod join_room;

// JoinRoom struct is now in lib.rs for Anchor macro compatibility
