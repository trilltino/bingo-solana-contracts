//! # Global Program Constants
//!
//! Centralized seeds, denominators, and sizing constraints shared across the
//! Fundraisely program. Keeping these values in one place guarantees
//! deterministic PDAs and enforces economic invariants.

/// PDA seed for the singleton global configuration account.
pub const GLOBAL_CONFIG_SEED: &[u8] = b"global-config";

/// PDA seed for the token registry allowlist.
pub const TOKEN_REGISTRY_SEED: &[u8] = b"token-registry";

/// PDA seed prefix for room accounts (`["room", host, room_id]`).
pub const ROOM_SEED: &[u8] = b"room";

/// PDA seed prefix for player entries (`["player", room, player]`).
pub const PLAYER_ENTRY_SEED: &[u8] = b"player";

/// PDA seed prefix for room vault accounts (`["room-vault", room]`).
pub const ROOM_VAULT_SEED: &[u8] = b"room-vault";

/// PDA seed prefix for asset prize vaults (`["prize-vault", room, index]`).
pub const PRIZE_VAULT_SEED: &[u8] = b"prize-vault";

/// Basis points denominator used for all percentage math (10_000 = 100%).
pub const BPS_DENOMINATOR: u64 = 10_000;

/// Maximum supported player count (prevents unbounded account growth).
pub const MAX_PLAYERS_LIMIT: u32 = 10_000;

/// Maximum safe u64 amount before triggering overflow protection.
pub const MAX_SAFE_AMOUNT: u64 = 9_223_372_036_854_775_807 / 2;

/// Minimum dust threshold to guard against spam/DoS style deposits.
pub const MIN_DUST_THRESHOLD: u64 = 1_000;
