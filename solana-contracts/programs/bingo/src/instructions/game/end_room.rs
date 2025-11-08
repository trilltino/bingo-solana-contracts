//! # End Room Instruction
//!
//! Finalize room, distribute prizes, and transfer charity donations.

use anchor_lang::prelude::*;
use crate::state::{RoomStatus, PrizeMode};
use crate::errors::BingoError;
use crate::events::RoomEnded;
use crate::instructions::utils::calculate_bps;

/// Helper function to perform a token transfer with room authority
/// Reduces stack usage by extracting repeated CPI logic
#[inline(never)]
fn transfer_from_room<'info>(
    token_program: &AccountInfo<'info>,
    from: &AccountInfo<'info>,
    to: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
    amount: u64,
) -> Result<()> {
    if amount > 0 {
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                token_program.clone(),
                anchor_spl::token::Transfer {
                    from: from.clone(),
                    to: to.clone(),
                    authority: authority.clone(),
                },
                signer_seeds,
            ),
            amount,
        )?;
    }
    Ok(())
}

/// End room and distribute prizes to winners
pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, crate::EndRoom<'info>>,
    _room_id: String,
    winners: Vec<Pubkey>,
) -> Result<()> {
    // Check room not ended
    require!(!ctx.accounts.room.ended, BingoError::RoomAlreadyEnded);

    require!(
        ctx.accounts.room.status == RoomStatus::Active,
        BingoError::InvalidRoomStatus
    );

    // Set ended flag immediately to prevent reentrancy
    ctx.accounts.room.ended = true;
    ctx.accounts.room.status = RoomStatus::Ended;

    // Read room data and validate
    let current_slot = Clock::get()?.slot;
    let is_expired = ctx.accounts.room.expiration_slot > 0 && current_slot >= ctx.accounts.room.expiration_slot;

    // Validation - only host can end room, unless it's expired (anyone can close expired rooms)
    if !is_expired {
        require!(
            ctx.accounts.host.key() == ctx.accounts.room.host,
            BingoError::Unauthorized
        );
    }

    // Determine which winners to use:
    // 1. If winners were declared via declare_winners instruction, use those (room.winners)
    // 2. Otherwise, use the passed-in winners parameter (backward compatibility)
    // Use a reference to avoid cloning and reduce stack usage
    let winners_to_use: &[Pubkey] = if !ctx.accounts.room.winners.is_empty() {
        // Winners were declared via declare_winners instruction
        &ctx.accounts.room.winners
    } else {
        // No declared winners, use passed-in parameter (old flow for backward compatibility)
        // Validate winner count (1-10)
        require!(
            winners.len() > 0 && winners.len() <= 10,
            BingoError::InvalidWinners
        );

        // Validate host is not a winner
        require!(
            !winners.contains(&ctx.accounts.room.host),
            BingoError::HostCannotBeWinner
        );

        &winners
    };

    // Calculate fee distribution
    let entry_fees_total = ctx.accounts.room.total_entry_fees;
    let extras_total = ctx.accounts.room.total_extras_fees;
    
    // Calculate total pool (entry fees + extras)
    // Both entry fees and extras should be split proportionally among all recipients
    let total_pool = entry_fees_total
        .checked_add(extras_total)
        .ok_or(BingoError::ArithmeticOverflow)?;

    // Apply percentage splits to total pool (entry fees + extras)
    // All recipients (platform, host, prize pool, charity) share proportionally in both entry fees and extras
    let platform_fee = calculate_bps(total_pool, ctx.accounts.global_config.platform_fee_bps)?;
    let host_fee = calculate_bps(total_pool, ctx.accounts.room.host_fee_bps)?;

    // Calculate prize pool amount based on prize mode
    // For AssetBased rooms: prizes are pre-deposited, so prize_amount from pool = 0
    // For PoolSplit rooms: prizes come from the entry fee pool
    let prize_amount = match ctx.accounts.room.prize_mode {
        PrizeMode::AssetBased => 0u64, // Prizes pre-deposited, don't take from pool
        PrizeMode::PoolSplit => calculate_bps(total_pool, ctx.accounts.room.prize_pool_bps)?,
    };

    // Charity gets remainder of total pool (calculated from total, not just entry fees)
    let charity_amount = total_pool
        .checked_sub(platform_fee)
        .and_then(|v| v.checked_sub(host_fee))
        .and_then(|v| v.checked_sub(prize_amount))
        .ok_or(BingoError::ArithmeticUnderflow)?;

    // Save values for event - OPTIMIZATION: pre-calculate to reduce stack usage
    let player_count = ctx.accounts.room.player_count;
    let room_key = ctx.accounts.room.key();
    let token_prog_key = ctx.accounts.token_program.key();
    let timestamp = Clock::get()?.unix_timestamp;

    // Prepare PDA signer seeds ONCE - reused for all transfers
    let bump = ctx.accounts.room.bump;
    let bump_bytes = [bump];
    let room_seeds: &[&[u8]] = &[
        b"room".as_ref(),
        ctx.accounts.room.host.as_ref(),
        ctx.accounts.room.room_id.as_bytes(),
        &bump_bytes
    ];
    let signer_seeds = &[room_seeds];

    // Transfer platform fee using helper function
    transfer_from_room(
        &ctx.accounts.token_program.to_account_info(),
        &ctx.accounts.room_vault.to_account_info(),
        &ctx.accounts.platform_token_account.to_account_info(),
        &ctx.accounts.room.to_account_info(),
        signer_seeds,
        platform_fee,
    )?;

    // Transfer host fee using helper function
    transfer_from_room(
        &ctx.accounts.token_program.to_account_info(),
        &ctx.accounts.room_vault.to_account_info(),
        &ctx.accounts.host_token_account.to_account_info(),
        &ctx.accounts.room.to_account_info(),
        signer_seeds,
        host_fee,
    )?;

    // Transfer charity donation using helper function
    transfer_from_room(
        &ctx.accounts.token_program.to_account_info(),
        &ctx.accounts.room_vault.to_account_info(),
        &ctx.accounts.charity_token_account.to_account_info(),
        &ctx.accounts.room.to_account_info(),
        signer_seeds,
        charity_amount,
    )?;

    // Distribute prizes to winners based on prize mode
    require!(
        ctx.remaining_accounts.len() >= winners_to_use.len(),
        BingoError::InvalidWinners
    );

    match ctx.accounts.room.prize_mode {
        PrizeMode::PoolSplit => {
            // Pool-based prize distribution with duplicate winner aggregation
            // When there are fewer players than prize positions, the same winner may appear
            // multiple times (e.g., [alice, alice, alice] for a 1-player game).
            // We aggregate prize amounts per unique winner to avoid borrowing the same
            // token account multiple times, which would cause AccountBorrowFailed.

            // OPTIMIZATION: Use u16 bitfield instead of [bool; 10] to reduce stack usage
            // Each bit represents whether index i has been processed (max 10 winners = 10 bits)
            let mut processed_bits: u16 = 0;

            for i in 0..winners_to_use.len().min(10) {
                // Check if this index was already processed (test bit i)
                if (processed_bits & (1 << i)) != 0 || i >= ctx.accounts.room.prize_distribution.len() {
                    continue;
                }

                let winner_pubkey = winners_to_use[i];
                let winner_distribution = ctx.accounts.room.prize_distribution[i];

                if winner_distribution == 0 {
                    continue;
                }

                // Calculate this position's prize amount
                // prize_amount is already calculated as: entry_fees_total * prize_pool_bps / 10000
                // winner_distribution is stored as 0-100 (e.g., 80 for 80%, 20 for 20%)
                // Use u128 to avoid overflow and ensure precision
                // Formula: prize_amount * winner_distribution / 100
                let mut total_amount = (prize_amount as u128)
                    .checked_mul(winner_distribution as u128)
                    .and_then(|v| v.checked_div(100))
                    .ok_or(BingoError::ArithmeticOverflow)? as u64;

                // Aggregate any duplicate entries for this winner (same pubkey in other positions)
                for j in (i + 1)..winners_to_use.len().min(10) {
                    // Check if index j was already processed
                    if (processed_bits & (1 << j)) == 0 &&
                       j < ctx.accounts.room.prize_distribution.len() &&
                       winners_to_use[j] == winner_pubkey {
                        let additional_amount = (prize_amount as u128)
                            .checked_mul(ctx.accounts.room.prize_distribution[j] as u128)
                            .and_then(|v| v.checked_div(100))
                            .ok_or(BingoError::ArithmeticOverflow)? as u64;
                        total_amount += additional_amount;
                        // Mark index j as processed (set bit j)
                        processed_bits |= 1 << j;
                    }
                }

                // Transfer aggregated amount to this winner
                if total_amount > 0 && i < ctx.remaining_accounts.len() {
                    let winner_token_account_info = &ctx.remaining_accounts[i];

                    // Verify the account is owned by the token program
                    require!(
                        winner_token_account_info.owner == &token_prog_key,
                        BingoError::InvalidWinners
                    );

                    // Deserialize and validate the token account (only once per unique winner)
                    let token_account_data = winner_token_account_info.try_borrow_data()?;
                    let winner_token_account = anchor_spl::token::TokenAccount::try_deserialize(&mut &token_account_data[..])?;

                    // Verify token account mint matches room's fee token mint
                    require!(
                        winner_token_account.mint == ctx.accounts.room.fee_token_mint,
                        BingoError::InvalidTokenMint
                    );

                    // Verify token account owner matches winner pubkey
                    require!(
                        winner_token_account.owner == winner_pubkey,
                        BingoError::InvalidTokenOwner
                    );

                    drop(token_account_data); // Release borrow before transfer

                    // Transfer aggregated prize amount to winner using helper function
                    transfer_from_room(
                        &ctx.accounts.token_program.to_account_info(),
                        &ctx.accounts.room_vault.to_account_info(),
                        winner_token_account_info,
                        &ctx.accounts.room.to_account_info(),
                        signer_seeds,
                        total_amount,
                    )?;

                    msg!("   Winner: {} receives {} tokens (aggregated)", winner_pubkey, total_amount);
                }

                // Mark index i as processed (set bit i)
                processed_bits |= 1 << i;
            }
        },
        PrizeMode::AssetBased => {
            // Asset-based prize distribution
            // Distribute pre-deposited prize assets to winners
            // remaining_accounts structure for asset mode:
            // [0..winners_count] = winner token accounts for fee_token_mint (for any pool split if applicable)
            // [winners_count..winners_count*2] = winner token accounts for prize assets
            // [winners_count*2..winners_count*2+3] = prize vault accounts

            msg!("Distributing asset-based prizes to {} winners", winners_to_use.len());

            let prize_vault_offset = winners_to_use.len() * 2;

            for (prize_index, prize_asset_opt) in ctx.accounts.room.prize_assets.iter().enumerate() {
                if let Some(prize_asset) = prize_asset_opt {
                    // Only distribute if prize was deposited and we have a winner for this position
                    if prize_asset.deposited && prize_index < winners_to_use.len() {
                        let winner = winners_to_use[prize_index];
                        let winner_token_account_info = &ctx.remaining_accounts[winners_to_use.len() + prize_index];
                        let prize_vault_info = &ctx.remaining_accounts[prize_vault_offset + prize_index];

                        // Verify prize vault is owned by token program
                        require!(
                            prize_vault_info.owner == &token_prog_key,
                            BingoError::InvalidVaultAccount
                        );

                        // Verify winner token account is owned by token program
                        require!(
                            winner_token_account_info.owner == &token_prog_key,
                            BingoError::InvalidWinners
                        );

                        // Deserialize and validate winner token account
                        let winner_token_data = winner_token_account_info.try_borrow_data()?;
                        let winner_token_account = anchor_spl::token::TokenAccount::try_deserialize(&mut &winner_token_data[..])?;

                        // Verify winner token account mint matches prize asset mint
                        require!(
                            winner_token_account.mint == prize_asset.mint,
                            BingoError::InvalidTokenMint
                        );

                        // Verify winner token account owner matches winner pubkey
                        require!(
                            winner_token_account.owner == winner,
                            BingoError::InvalidTokenOwner
                        );

                        drop(winner_token_data); // Release borrow before transfer

                        // Derive prize vault PDA seeds for signing
                        // Note: Prize vaults are controlled by the room PDA
                        let room_key = ctx.accounts.room.key();
                        let prize_index_bytes = [prize_index as u8];

                        let (expected_prize_vault, _prize_vault_bump) = Pubkey::find_program_address(
                            &[
                                b"prize-vault",
                                room_key.as_ref(),
                                &prize_index_bytes,
                            ],
                            ctx.program_id,
                        );

                        // Verify the prize vault matches expected PDA
                        require!(
                            prize_vault_info.key() == expected_prize_vault,
                            BingoError::InvalidVaultAccount
                        );

                        // Transfer prize asset from prize vault to winner using helper function
                        // The prize vault's authority is the room PDA
                        transfer_from_room(
                            &ctx.accounts.token_program.to_account_info(),
                            prize_vault_info,
                            winner_token_account_info,
                            &ctx.accounts.room.to_account_info(),
                            signer_seeds,
                            prize_asset.amount,
                        )?;

                        msg!("   Prize {}: Winner {} receives {} of token {}",
                            prize_index + 1, winner, prize_asset.amount, prize_asset.mint);
                    }
                }
            }
        }
    }

    msg!("Room ended and prizes distributed");
    msg!("   Entry fees: {}, Extras: {} (split proportionally)", entry_fees_total, extras_total);
    msg!("   Total pool: {}, Platform: {}, Host: {}, Charity: {}, Prizes: {}",
        total_pool, platform_fee, host_fee, charity_amount, prize_amount);

    // Emit event for off-chain indexers and frontend
    // OPTIMIZATION: Convert slice to Vec only when needed for event
    emit!(RoomEnded {
        room: room_key,
        winners: winners_to_use.to_vec(),
        platform_amount: platform_fee,
        host_amount: host_fee,
        charity_amount,
        prize_amount,
        total_players: player_count,
        timestamp,
    });

    Ok(())
}

// Note: EndRoom struct moved to lib.rs for Anchor macro compatibility
