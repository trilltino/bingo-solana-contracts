//! # Fundraisely Error Codes
//!
//! Comprehensive error handling for the Fundraisely smart contract.
//!
//! ## Overview
//!
//! This module defines all custom error types returned by the Fundraisely program. These errors
//! provide clear, actionable feedback to frontend applications and users when transactions fail
//! validation or execution requirements.
//!
//! ## Role in Architecture
//!
//! Error codes serve multiple critical functions:
//!
//! 1. **Transaction Validation**: Pre-execution checks prevent invalid state changes
//! 2. **Security Enforcement**: Access control and permission validation
//! 3. **Economic Constraints**: Fee structure and allocation limit enforcement
//! 4. **Frontend UX**: Clear error messages for user-facing error handling
//! 5. **Debugging**: Specific error codes help identify issues during development
//!
//! ## Frontend Integration
//!
//! The `useFundraiselyContract.ts` hook catches these errors and maps them to user-friendly messages:
//!
//! ```typescript
//! try {
//!   await program.methods.joinRoom(roomId, extras).rpc();
//! } catch (error) {
//!   if (error.message.includes("MaxPlayersReached")) {
//!     showError("Room is full. Please try another room.");
//!   } else if (error.message.includes("RoomExpired")) {
//!     showError("This room has expired and is no longer accepting players.");
//!   }
//!   // ... additional error mapping
//! }
//! ```
//!
//! ## Error Categories
//!
//! ### Access Control Errors
//! - `Unauthorized`: Caller lacks permission for requested operation
//! - `HostCannotBeWinner`: Prevents hosts from awarding themselves prizes
//!
//! ### Room State Errors
//! - `RoomAlreadyExists`, `RoomNotFound`, `RoomNotReady`: Room lifecycle validation
//! - `RoomAlreadyEnded`, `RoomExpired`: Game completion and timing checks
//! - `InvalidRoomStatus`: State machine transition validation
//!
//! ### Player Participation Errors
//! - `PlayerAlreadyJoined`: Prevents duplicate entries
//! - `MaxPlayersReached`: Enforces room capacity limits
//!
//! ### Economic Model Errors
//! - `HostFeeTooHigh`: Enforces 5% maximum host fee
//! - `PrizePoolTooLow`: Ensures prize pool is greater than 0
//! - `PrizePoolTooHigh`: Enforces host + prize ≤ 40%
//! - `CharityBelowMinimum`: Enforces 40% minimum charity allocation
//! - `InvalidPrizeDistribution`: Ensures prize percentages sum to exactly 100%
//!
//! ### Input Validation Errors
//! - `InvalidEntryFee`: Entry fee must be > 0
//! - `InvalidWinners`: Winner list validation (1-3 winners, valid pubkeys)
//! - `InvalidRoomId`: Room ID length constraints (1-32 chars)
//! - `InvalidMemo`: Charity memo length constraints
//!
//! ### Safety Errors
//! - `ArithmeticOverflow`/`ArithmeticUnderflow`: Checked math safety
//! - `InsufficientBalance`: Token balance validation
//! - `EmergencyPause`: Global circuit breaker for security incidents
//!
//! ## Economic Model Enforcement
//!
//! Several errors specifically protect the economic model's integrity:
//!
//! - Charity allocation cannot fall below 40% of entry fees (CharityBelowMinimum)
//! - Host fees cannot exceed 5% (HostFeeTooHigh)
//! - Prize pools must be > 0 and host + prizes ≤ 40% (PrizePoolTooLow, PrizePoolTooHigh)
//! - Combined allocations must not exceed 100% (TotalAllocationTooHigh)
//!
//! These constraints are enforced at room creation and cannot be bypassed, ensuring transparent
//! and trustless charitable fundraising aligned with the platform's mission.

use anchor_lang::prelude::*;

#[error_code]
pub enum BingoError {
    #[msg("Unauthorized access")]
    Unauthorized,

    #[msg("Room already exists")]
    RoomAlreadyExists,

    #[msg("Room not found")]
    RoomNotFound,

    #[msg("Room not ready for players")]
    RoomNotReady,

    #[msg("Invalid room status")]
    InvalidRoomStatus,

    #[msg("Room already ended")]
    RoomAlreadyEnded,

    #[msg("Room has expired")]
    RoomExpired,

    #[msg("Player already joined")]
    PlayerAlreadyJoined,

    #[msg("Host cannot be a winner")]
    HostCannotBeWinner,

    #[msg("Invalid winners list")]
    InvalidWinners,

    #[msg("Token not approved")]
    TokenNotApproved,

    #[msg("Token is already in the approved registry")]
    TokenAlreadyApproved,

    #[msg("Token registry is full (max 50 tokens)")]
    TokenRegistryFull,

    #[msg("Invalid entry fee")]
    InvalidEntryFee,

    #[msg("Host fee exceeds maximum (5%)")]
    HostFeeTooHigh,

    #[msg("Prize pool exceeds maximum (40% minus host fee)")]
    PrizePoolTooHigh,

    #[msg("Prize pool must be greater than 0")]
    PrizePoolTooLow,

    #[msg("Charity allocation below minimum (40%)")]
    CharityBelowMinimum,

    #[msg("Total allocation exceeds maximum")]
    TotalAllocationTooHigh,

    #[msg("Prize distribution must sum to 100")]
    InvalidPrizeDistribution,

    #[msg("Insufficient balance")]
    InsufficientBalance,

    #[msg("Contract is paused")]
    EmergencyPause,

    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,

    #[msg("Arithmetic underflow")]
    ArithmeticUnderflow,

    #[msg("Invalid room ID (max 32 characters)")]
    InvalidRoomId,

    #[msg("Invalid memo (max 28 characters)")]
    InvalidMemo,

    #[msg("Room has reached maximum players")]
    MaxPlayersReached,

    #[msg("Invalid max_players (must be between 1 and 1000)")]
    InvalidMaxPlayers,

    #[msg("Token account mint does not match room token mint")]
    InvalidTokenMint,

    #[msg("Token account owner does not match expected winner")]
    InvalidTokenOwner,

    #[msg("Winners have already been declared for this room")]
    WinnersAlreadyDeclared,

    #[msg("Invalid prize amount (must be > 0)")]
    InvalidPrizeAmount,

    #[msg("Prize already deposited")]
    PrizeAlreadyDeposited,

    #[msg("Prize not deposited yet")]
    PrizeNotDeposited,

    #[msg("All prizes must be deposited before players can join")]
    PrizesNotFullyFunded,

    #[msg("Room cannot be recovered yet (not abandoned)")]
    RoomNotAbandoned,

    #[msg("Invalid player entry (winner did not join the room)")]
    InvalidPlayerEntry,

    #[msg("Invalid vault account (must be a valid TokenAccount)")]
    InvalidVaultAccount,

    #[msg("Invalid vault authority (vault must be owned by room PDA)")]
    InvalidVaultAuthority,

    #[msg("Room is no longer accepting new players")]
    JoiningClosed,

    #[msg("Vault must be empty before cleanup")]
    VaultNotEmpty,

    #[msg("Insufficient authority for this operation")]
    InsufficientAuthority,
}
