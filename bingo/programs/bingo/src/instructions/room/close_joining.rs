//! # Close Joining Instruction
//!
//! Allows host to close a room to new players before reaching max capacity.
//! Useful when host wants to start the game early or lock the player list.
//!
//! ## Security
//! - Only the room host can call this
//! - Can only be called before room has ended
//! - Cannot be reversed (one-way operation)
//!
//! ## Usage
//! ```typescript
//! await program.methods
//!   .closeJoining(roomId)
//!   .accounts({ room, host })
//!   .rpc();
//! ```

use crate::{BingoError, JoiningClosed as JoiningClosedEvent, Room};
use anchor_lang::prelude::*;

pub fn handler(ctx: Context<CloseJoining>, room_id: String) -> Result<()> {
    let room = &mut ctx.accounts.room;
    let host = &ctx.accounts.host;

    // Verify host authority
    require!(room.host == host.key(), BingoError::Unauthorized);

    // Cannot close joining if room already ended
    require!(!room.ended, BingoError::RoomAlreadyEnded);

    // Cannot close joining if already closed
    require!(!room.joining_closed, BingoError::JoiningClosed);

    // Close joining
    room.joining_closed = true;

    // Emit event
    emit!(JoiningClosedEvent {
        room: room.key(),
        room_id: room_id.clone(),
        host: host.key(),
        player_count: room.player_count,
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!(
        "Joining closed for room {} with {} players",
        room_id,
        room.player_count
    );

    Ok(())
}

/// Context for closing room joining
#[derive(Accounts)]
#[instruction(room_id: String)]
pub struct CloseJoining<'info> {
    /// Room PDA account
    #[account(
        mut,
        seeds = [b"room", room.host.as_ref(), room_id.as_bytes()],
        bump = room.bump
    )]
    pub room: Account<'info, Room>,

    /// Host account closing the room
    #[account(mut)]
    pub host: Signer<'info>,
}
