//! # Add Prize Asset Instruction
//!
//! Escrows a prize asset into the room's prize vault for asset-based rooms

use crate::errors::BingoError;
use crate::events::PrizeAssetDeposited;
use crate::state::RoomStatus;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};

/// Escrow a prize asset into the room
pub fn handler(
    ctx: Context<AddPrizeAsset>,
    _room_id: String,
    prize_index: u8, // 0, 1, or 2
) -> Result<()> {
    let room = &mut ctx.accounts.room;

    // Only for asset-based rooms
    require!(
        room.prize_mode == crate::state::PrizeMode::AssetBased,
        BingoError::InvalidRoomStatus
    );

    // Must be host
    require!(
        ctx.accounts.host.key() == room.host,
        BingoError::Unauthorized
    );

    // Prize index must be valid
    require!(prize_index < 3, BingoError::InvalidWinners);

    // Store room key before mutable borrow
    let room_key = room.key();

    // Get prize asset info
    let prize_asset = room.prize_assets[prize_index as usize]
        .as_mut()
        .ok_or(BingoError::InvalidWinners)?;

    // Check not already deposited
    require!(!prize_asset.deposited, BingoError::PrizeAlreadyDeposited);

    // Store mint and amount before further borrows
    let prize_mint = prize_asset.mint;
    let prize_amount = prize_asset.amount;

    // Transfer tokens from host to prize vault
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.host_token_account.to_account_info(),
            to: ctx.accounts.prize_vault.to_account_info(),
            authority: ctx.accounts.host.to_account_info(),
        },
    );
    token::transfer(cpi_ctx, prize_asset.amount)?;

    // Mark as deposited
    prize_asset.deposited = true;

    msg!(
        "Prize {} deposited: {} tokens",
        prize_index + 1,
        prize_asset.amount
    );

    // Emit event
    emit!(PrizeAssetDeposited {
        room: room_key,
        prize_index,
        token_mint: prize_mint,
        amount: prize_amount,
        depositor: ctx.accounts.host.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    // Check if all prizes are now deposited
    let all_deposited = room
        .prize_assets
        .iter()
        .all(|asset| asset.as_ref().map_or(true, |a| a.deposited));

    if all_deposited {
        room.status = RoomStatus::Ready;
        msg!("   All prizes deposited - room is now Ready for players");
    } else {
        room.status = RoomStatus::PartiallyFunded;
        msg!("   Status: PartiallyFunded (more prizes needed)");
    }

    Ok(())
}

/// Context for adding prize asset
#[derive(Accounts)]
#[instruction(room_id: String)]
pub struct AddPrizeAsset<'info> {
    /// Room PDA account
    #[account(
        mut,
        seeds = [b"room", room.host.as_ref(), room_id.as_bytes()],
        bump = room.bump
    )]
    pub room: Account<'info, Room>,

    /// Prize vault token account (destination)
    #[account(mut)]
    pub prize_vault: Account<'info, anchor_spl::token::TokenAccount>,

    /// Host token account (source of funds)
    #[account(mut)]
    pub host_token_account: Account<'info, anchor_spl::token::TokenAccount>,

    /// Host account depositing the prize
    #[account(mut)]
    pub host: Signer<'info>,

    /// Token program for token transfers
    pub token_program: Program<'info, anchor_spl::token::Token>,
}
