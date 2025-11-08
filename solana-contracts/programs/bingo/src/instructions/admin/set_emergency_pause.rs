//! # Set Emergency Pause Instruction
//!
//! Allows admin to toggle the emergency pause flag on the global config.
//! When paused, critical operations (room creation, joining) are blocked.
//!
//! ## Security
//! - Only the admin specified in GlobalConfig can call this
//! - Emits EmergencyPauseToggled event for transparency
//!
//! ## Usage
//! ```typescript
//! await program.methods
//!   .setEmergencyPause(true) // or false to unpause
//!   .accounts({ globalConfig, admin })
//!   .rpc();
//! ```

use anchor_lang::prelude::*;
use crate::{BingoError, EmergencyPauseToggled, SetEmergencyPause};

pub fn handler(ctx: Context<SetEmergencyPause>, paused: bool) -> Result<()> {
    let global_config = &mut ctx.accounts.global_config;
    let admin = &ctx.accounts.admin;

    // Verify admin authority
    require!(
        global_config.admin == admin.key(),
        BingoError::Unauthorized
    );

    // Update pause state
    global_config.emergency_pause = paused;

    // Emit event
    emit!(EmergencyPauseToggled {
        paused,
        admin: admin.key(),
        slot: Clock::get()?.slot,
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!(
        "Emergency pause {}",
        if paused { "enabled" } else { "disabled" }
    );

    Ok(())
}
