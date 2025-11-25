use anchor_lang::prelude::*;

use crate::constants::MAX_PLAYERS_LIMIT;
use crate::errors::BingoError;

/// Input validation helpers for untrusted user data.
pub struct InputValidator;

impl InputValidator {
    /// Validate room ID length and content.
    pub fn validate_room_id(room_id: &str) -> Result<()> {
        require!(
            !room_id.is_empty() && room_id.len() <= 32,
            BingoError::InvalidRoomId
        );

        require!(!room_id.contains('\0'), BingoError::InvalidRoomId);

        Ok(())
    }

    /// Validate charity memo length.
    pub fn validate_charity_memo(memo: &str) -> Result<()> {
        require!(memo.len() <= 28, BingoError::InvalidMemo);

        require!(!memo.contains('\0'), BingoError::InvalidMemo);

        Ok(())
    }

    /// Validate max players is reasonable.
    pub fn validate_max_players(max_players: u32) -> Result<()> {
        require!(
            max_players > 0 && max_players <= MAX_PLAYERS_LIMIT,
            BingoError::InvalidMaxPlayers
        );
        Ok(())
    }

    /// Validate winner count.
    pub fn validate_winner_count(count: usize, min: usize, max: usize) -> Result<()> {
        require!(count >= min && count <= max, BingoError::InvalidWinners);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_validator() {
        assert!(InputValidator::validate_room_id("test-room").is_ok());
        assert!(InputValidator::validate_room_id("a").is_ok());

        let long_id = "a".repeat(33);
        assert!(InputValidator::validate_room_id(&long_id).is_err());

        assert!(InputValidator::validate_room_id("").is_err());
    }
}
