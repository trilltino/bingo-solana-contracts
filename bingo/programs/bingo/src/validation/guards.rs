use anchor_lang::prelude::*;

use crate::errors::BingoError;

/// Reentrancy guard helpers used by long-running instructions.
pub struct ReentrancyGuard;

impl ReentrancyGuard {
    /// Assert that a room has not been processed yet.
    pub fn check_room_not_ended(ended: bool) -> Result<()> {
        require!(!ended, BingoError::RoomAlreadyEnded);
        Ok(())
    }

    /// Update ended flag and status following the checks-effects-interactions pattern.
    pub fn set_ended_flag<T>(ended: &mut bool, status: &mut T, new_status: T) -> Result<()> {
        require!(!*ended, BingoError::RoomAlreadyEnded);
        *ended = true;
        *status = new_status;
        Ok(())
    }
}

/// Emergency pause guard that enforces the global circuit breaker.
pub struct EmergencyGuard;

impl EmergencyGuard {
    /// Reject execution when the platform admin has paused the program.
    pub fn check_not_paused(paused: bool) -> Result<()> {
        require!(!paused, BingoError::EmergencyPause);
        Ok(())
    }
}
