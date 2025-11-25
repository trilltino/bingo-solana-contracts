//! # Recover Room Instruction
//!
//! This administrative instruction allows the platform admin to recover funds from abandoned
//! or expired rooms that never completed. It implements a fair refund mechanism where 90% of
//! collected funds are returned to players and 10% goes to the platform as a recovery fee.
//! This prevents situations where funds get locked if a host abandons a room before ending it.
//! The instruction uses remaining_accounts to dynamically handle refunds to any number of players.

use crate::events::RoomRecovered;
use crate::{errors::BingoError, GlobalConfig, Room};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Transfer};

/// Recover an abandoned room - refund players
pub fn handler<'info>(
    ctx: Context<'_, '_, 'info, 'info, RecoverRoom<'info>>,
    _room_id: String,
) -> Result<()> {
    let room = &mut ctx.accounts.room;

    // Only admin can recover
    require!(
        ctx.accounts.admin.key() == ctx.accounts.global_config.admin,
        BingoError::Unauthorized
    );

    // Room must not be ended
    require!(!room.ended, BingoError::RoomAlreadyEnded);

    // Room should be expired or abandoned (we'll just check it has funds)
    require!(room.total_collected > 0, BingoError::InsufficientBalance);

    msg!("Recovering abandoned room: {}", room.room_id);
    msg!("Total collected: {}", room.total_collected);
    msg!("Player count: {}", room.player_count);

    // Calculate amounts
    let total_to_refund = room.total_collected;
    let platform_fee = total_to_refund
        .checked_mul(10)
        .and_then(|v| v.checked_div(100))
        .ok_or(BingoError::ArithmeticOverflow)?;

    let refund_per_player = total_to_refund
        .saturating_sub(platform_fee)
        .checked_div(room.player_count as u64)
        .unwrap_or(0);

    msg!("   Platform fee (10%): {}", platform_fee);
    msg!("   Refund per player (90%): {}", refund_per_player);

    // Transfer platform fee
    let room_key = room.key();
    let seeds = &[b"room-vault", room_key.as_ref(), &[ctx.bumps.room_vault]];
    let signer_seeds = &[&seeds[..]];

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.room_vault.to_account_info(),
            to: ctx.accounts.platform_token_account.to_account_info(),
            authority: ctx.accounts.room_vault.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(cpi_ctx, platform_fee)?;

    msg!("   Platform fee transferred");

    // Refund each player (uses remaining_accounts)
    for (i, account_info) in ctx.remaining_accounts.iter().enumerate() {
        if i % 2 == 1 {
            // Odd indices are token accounts
            let player_token_account = Account::<TokenAccount>::try_from(account_info)?;

            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.room_vault.to_account_info(),
                    to: player_token_account.to_account_info(),
                    authority: ctx.accounts.room_vault.to_account_info(),
                },
                signer_seeds,
            );

            token::transfer(cpi_ctx, refund_per_player)?;
            msg!("   Refunded player {}: {}", i / 2, refund_per_player);
        }
    }

    // Mark room as ended
    room.ended = true;
    room.status = crate::state::RoomStatus::Ended;

    msg!("Room recovered and players refunded");

    // Emit event
    emit!(RoomRecovered {
        room: room.key(),
        amount_recovered: total_to_refund,
        recipient: ctx.accounts.platform_token_account.owner,
        admin: ctx.accounts.admin.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

/// Context for recovering abandoned room
#[derive(Accounts)]
#[instruction(room_id: String)]
pub struct RecoverRoom<'info> {
    /// Room PDA account
    #[account(
        mut,
        seeds = [b"room", room.host.as_ref(), room_id.as_bytes()],
        bump = room.bump
    )]
    pub room: Account<'info, Room>,

    /// Room vault token account (source of refunds)
    #[account(
        mut,
        seeds = [b"room-vault", room.key().as_ref()],
        bump
    )]
    pub room_vault: Account<'info, anchor_spl::token::TokenAccount>,

    /// Global configuration PDA
    #[account(
        seeds = [b"global-config"],
        bump = global_config.bump
    )]
    pub global_config: Account<'info, GlobalConfig>,

    /// Platform wallet token account (receives recovery fee)
    #[account(mut)]
    pub platform_token_account: Account<'info, anchor_spl::token::TokenAccount>,

    /// Admin account recovering the room
    #[account(mut)]
    pub admin: Signer<'info>,

    /// Token program for token transfers
    pub token_program: Program<'info, anchor_spl::token::Token>,
}
