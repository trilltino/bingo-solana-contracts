//! # Game Instructions Module
//!
//! Instructions for game execution: winner declaration and fund distribution.
//!
//! This module contains the core game-ending operations that determine winners and
//! distribute accumulated funds to all stakeholders (platform, host, charity, winners).
//! The separation of declare_winners and end_room provides transparency and allows
//! verification before irreversible fund transfers.
//!
//! ## Instructions
//!
//! - **declare_winners**: Host declares 1-3 winners (transparent, verifiable)
//! - **end_room**: Distribute funds to all parties (platform, host, charity, winners)
//!
//! ## Instruction Flow
//!
//! ```text
//! 1. Host calls declare_winners(winners: [alice, bob, charlie])
//!    - Winners stored in Room.winners
//!    - WinnersDeclared event emitted
//!    - Frontend shows winner announcement
//!
//! 2. Host calls end_room()
//!    - Uses Room.winners (ignores end_room winners parameter if declared)
//!    - Validates winners joined the room
//!    - Calculates distribution amounts
//!    - Transfers tokens to all parties
//!    - RoomEnded event emitted
//!    - Room.ended = true (immutable)
//! ```
//!
//! ## Why Separate Instructions?
//!
//! - **Transparency**: Winners publicly declared before funds move
//! - **Verification**: Players can verify winner validity before distribution
//! - **Audit Trail**: Two separate events for better tracking
//! - **Flexibility**: Allows time between declaration and distribution
//! - **Compliance**: Meets requirements for separated winner declaration (per requirements doc)

pub mod declare_winners;
pub mod end_room;

// DeclareWinners and EndRoom structs are now in lib.rs for Anchor macro compatibility
