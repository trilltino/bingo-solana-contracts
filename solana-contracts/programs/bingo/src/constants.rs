//! # Program Constants
//!
//! Centralized constants for validation, limits, and configuration values.
//! These constants prevent magic numbers throughout the codebase and provide
//! compile-time validation of critical values.

/// Maximum length for room_id string (as documented in Room struct)
pub const MAX_ROOM_ID_LENGTH: usize = 32;

/// Minimum length for room_id string (must be at least 1 character)
pub const MIN_ROOM_ID_LENGTH: usize = 1;

/// Maximum length for charity_memo string (fits in SPL Token memo)
pub const MAX_CHARITY_MEMO_LENGTH: usize = 28;

/// Maximum number of winners in a room
pub const MAX_WINNERS: usize = 10;

/// Maximum entry fee in lamports/tokens (prevents accidental mistakes)
/// Set to 1_000_000_000_000 (1 trillion base units = 1M tokens with 6 decimals)
pub const MAX_ENTRY_FEE: u64 = 1_000_000_000_000;

/// Minimum entry fee (must be at least 1)
pub const MIN_ENTRY_FEE: u64 = 1;

/// Maximum number of approved tokens in TokenRegistry
pub const MAX_APPROVED_TOKENS: usize = 50;

/// Maximum number of prize assets per room
pub const MAX_PRIZE_ASSETS: usize = 3;

/// Maximum player count for a room (sanity check)
pub const MAX_PLAYER_COUNT: u32 = 10_000;

/// Basis points denominator (100% = 10000 BPS)
pub const BPS_DENOMINATOR: u64 = 10_000;

/// Maximum total BPS allocation (platform + host + prize pool + charity = 10000)
pub const MAX_TOTAL_BPS: u16 = 10_000;

// Compile-time assertions to ensure constants are valid
// These will fail at compile time if constraints are violated

const _: () = assert!(MAX_WINNERS <= 10, "MAX_WINNERS cannot exceed 10 (storage limitation)");
const _: () = assert!(MIN_ROOM_ID_LENGTH > 0, "MIN_ROOM_ID_LENGTH must be at least 1");
const _: () = assert!(MAX_ROOM_ID_LENGTH >= MIN_ROOM_ID_LENGTH, "MAX_ROOM_ID_LENGTH must be >= MIN_ROOM_ID_LENGTH");
const _: () = assert!(MAX_CHARITY_MEMO_LENGTH <= 32, "MAX_CHARITY_MEMO_LENGTH cannot exceed SPL Token memo limit");
const _: () = assert!(MIN_ENTRY_FEE > 0, "MIN_ENTRY_FEE must be at least 1");
const _: () = assert!(MAX_ENTRY_FEE > MIN_ENTRY_FEE, "MAX_ENTRY_FEE must be greater than MIN_ENTRY_FEE");
const _: () = assert!(BPS_DENOMINATOR == 10_000, "BPS_DENOMINATOR must be exactly 10000");
const _: () = assert!(MAX_TOTAL_BPS == 10_000, "MAX_TOTAL_BPS must equal BPS_DENOMINATOR");
