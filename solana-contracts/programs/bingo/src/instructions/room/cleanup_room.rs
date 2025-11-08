//! # Cleanup Room Instruction
//!
//! Closes the room vault and reclaims rent after a room has ended and distributed all funds.
//! This is an important housekeeping operation to recover rent lamports.
//!
//! ## Security
//! - Can only be called by room host or admin
//! - Room must be ended
//! - Vault must be empty (balance = 0)
//! - Closes the vault token account to reclaim rent
//!
//! ## Usage
//! ```typescript
//! await program.methods
//!   .cleanupRoom(roomId)
//!   .accounts({ room, roomVault, host, tokenProgram })
//!   .rpc();
//! ```

use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, CloseAccount};
use crate::{BingoError, RoomCleaned, CleanupRoom};
use crate::state::PrizeMode;

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, CleanupRoom<'info>>, room_id: String) -> Result<()> {
    let room = &ctx.accounts.room;
    let room_vault = &ctx.accounts.room_vault;
    let caller = &ctx.accounts.caller;
    let global_config = &ctx.accounts.global_config;

    // Verify caller is either host or admin
    let is_host = room.host == caller.key();
    let is_admin = global_config.admin == caller.key();
    require!(
        is_host || is_admin,
        BingoError::InsufficientAuthority
    );

    // Room must be ended
    require!(
        room.ended,
        BingoError::InvalidRoomStatus
    );

    // Vault must be empty
    require!(
        room_vault.amount == 0,
        BingoError::VaultNotEmpty
    );

    // Get rent before closing - start with room_vault rent
    let mut rent_reclaimed = room_vault.to_account_info().lamports();

    // Close the vault account using room PDA signer (vault owner is room PDA)
    let room_key = room.key();
    let room_signer_seeds: &[&[&[u8]]] = &[&[
        b"room",
        room.host.as_ref(),
        room.room_id.as_bytes(),
        &[room.bump],
    ]];

    let cpi_accounts = CloseAccount {
        account: room_vault.to_account_info(),
        destination: caller.to_account_info(),
        authority: room.to_account_info(), // room PDA is authority (owner of vault)
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, room_signer_seeds);

    token::close_account(cpi_ctx)?;

    // Close prize vault accounts for asset-based rooms
    // Prize vaults are passed as remaining accounts (optional) in order: [0, 1, 2]
    if room.prize_mode == PrizeMode::AssetBased && ctx.remaining_accounts.len() >= 3 {
        for prize_index in 0..3 {
            // Derive prize vault PDA to verify (bump not needed for verification)
            let (prize_vault_pda, _prize_vault_bump) = Pubkey::find_program_address(
                &[
                    b"prize-vault",
                    room_key.as_ref(),
                    &[prize_index as u8],
                ],
                ctx.program_id,
            );

            // Get prize vault account from remaining accounts
            let prize_vault_info = &ctx.remaining_accounts[prize_index];
            
            // Verify it's the correct prize vault
            if prize_vault_info.key() == prize_vault_pda {
                // Check if account exists and has data
                if !prize_vault_info.data_is_empty() {
                    // Try to deserialize as TokenAccount to verify it's empty
                    if let Ok(prize_vault_account) = TokenAccount::try_deserialize(&mut &prize_vault_info.data.borrow()[..]) {
                        if prize_vault_account.amount == 0 {
                            // Get rent before closing
                            let prize_vault_rent = prize_vault_info.lamports();
                            rent_reclaimed = rent_reclaimed
                                .checked_add(prize_vault_rent)
                                .ok_or(BingoError::ArithmeticOverflow)?;

                            // Close prize vault using room PDA as authority
                            // Prize vault owner is room PDA, so use room PDA signer seeds
                            let room_signer_seeds: &[&[&[u8]]] = &[&[
                                b"room",
                                room.host.as_ref(),
                                room.room_id.as_bytes(),
                                &[room.bump],
                            ]];

                            let prize_vault_cpi_accounts = CloseAccount {
                                account: prize_vault_info.to_account_info(),
                                destination: caller.to_account_info(),
                                authority: ctx.accounts.room.to_account_info(), // room PDA is authority
                            };

                            let prize_vault_cpi_ctx = CpiContext::new_with_signer(
                                ctx.accounts.token_program.to_account_info(),
                                prize_vault_cpi_accounts,
                                room_signer_seeds,
                            );

                            token::close_account(prize_vault_cpi_ctx)?;

                            msg!("Closed prize vault {} for prize index {}", prize_vault_pda, prize_index);
                        }
                    }
                }
            }
        }
    }

    // Close the room account itself - the close constraint will handle this safely
    // SECURITY: Using Anchor's close constraint (see CleanupRoom struct in lib.rs)
    // This automatically zeros the account discriminator and transfers rent to caller
    let room_rent = ctx.accounts.room.to_account_info().lamports();
    rent_reclaimed = rent_reclaimed
        .checked_add(room_rent)
        .ok_or(BingoError::ArithmeticOverflow)?;

    // Emit event
    emit!(RoomCleaned {
        room: room.key(),
        room_id: room_id.clone(),
        rent_reclaimed,
        recipient: caller.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!(
        "Room {} cleaned up, {} lamports reclaimed",
        room_id,
        rent_reclaimed
    );

    Ok(())
}
