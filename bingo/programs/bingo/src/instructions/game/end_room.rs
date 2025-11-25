//! # End Room Instruction
//!
//! Finalize room, distribute prizes, and transfer charity donations.
//!
//! ## Charity Wallet Support
//!
//! This instruction supports dynamic charity wallet addresses, enabling integration with
//! The Giving Block (TGB) API and other systems that provide per-transaction charity addresses.
//!
//! The `charity_token_account` parameter accepts ANY valid SPL TokenAccount and is NOT
//! validated against `GlobalConfig.charity_wallet`. This allows:
//!
//! - **TGB Dynamic Wallets**: Each transaction can use a different charity wallet address
//!   returned by the TGB API for that specific transaction/intent.
//! - **Custom Charity Wallets**: Hosts can specify custom charity wallets per-room or per-transaction.
//! - **Per-Room Configuration**: Rooms can route charity donations to different wallets.
//!
//! Security: The charity_token_account is validated as a valid SPL TokenAccount by Anchor,
//! ensuring it's a properly formatted token account, but the owner wallet address is not restricted.

use crate::errors::BingoError;
use crate::events::RoomEnded;
use crate::state::{PrizeMode, RoomStatus};
use crate::utils::calculate_bps;
use crate::validation::{EmergencyGuard, ReentrancyGuard};
use anchor_lang::prelude::*;

/// End room and distribute prizes to winners
pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, EndRoom<'info>>,
    _room_id: String,
    winners: Vec<Pubkey>,
) -> Result<()> {
    // REENTRANCY & EMERGENCY PROTECTION
    EmergencyGuard::check_not_paused(ctx.accounts.global_config.emergency_pause)?;
    ReentrancyGuard::check_room_not_ended(ctx.accounts.room.ended)?;

    // Set flags before any external calls (checks-effects-interactions pattern)
    // Validate room status and set ended flag immediately to prevent reentrancy
    require!(
        ctx.accounts.room.status == RoomStatus::Active,
        BingoError::InvalidRoomStatus
    );

    // Set ended flag and status before making external calls (checks-effects-interactions pattern)
    ctx.accounts.room.ended = true;
    ctx.accounts.room.status = RoomStatus::Ended;

    // Read room data and validate
    let current_slot = Clock::get()?.slot;
    let is_expired =
        ctx.accounts.room.expiration_slot > 0 && current_slot >= ctx.accounts.room.expiration_slot;

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

    // Calculate total pool (entry fees + extras)
    let entry_fees_total = ctx.accounts.room.total_entry_fees;
    let extras_total = ctx.accounts.room.total_extras_fees;
    let total_pool = entry_fees_total
        .checked_add(extras_total)
        .ok_or(BingoError::ArithmeticOverflow)?;

    // Apply percentage splits to TOTAL POOL (not just entry fees)
    let platform_fee = calculate_bps(total_pool, ctx.accounts.global_config.platform_fee_bps)?;
    let host_fee = calculate_bps(total_pool, ctx.accounts.room.host_fee_bps)?;

    // Calculate prize pool amount based on prize mode
    let prize_amount = match ctx.accounts.room.prize_mode {
        PrizeMode::AssetBased => 0u64,
        PrizeMode::PoolSplit => calculate_bps(total_pool, ctx.accounts.room.prize_pool_bps)?,
    };

    // Charity gets remainder
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

    // Prepare PDA signer seeds - inline to reduce stack allocation
    let bump = ctx.accounts.room.bump;
    let bump_bytes = [bump];

    // Transfer platform fee
    if platform_fee > 0 {
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: ctx.accounts.room_vault.to_account_info(),
                    to: ctx.accounts.platform_token_account.to_account_info(),
                    authority: ctx.accounts.room.to_account_info(),
                },
                &[&[
                    b"room".as_ref(),
                    ctx.accounts.room.host.as_ref(),
                    ctx.accounts.room.room_id.as_bytes(),
                    &bump_bytes,
                ]],
            ),
            platform_fee,
        )?;
    }

    // Transfer host fee
    if host_fee > 0 {
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: ctx.accounts.room_vault.to_account_info(),
                    to: ctx.accounts.host_token_account.to_account_info(),
                    authority: ctx.accounts.room.to_account_info(),
                },
                &[&[
                    b"room".as_ref(),
                    ctx.accounts.room.host.as_ref(),
                    ctx.accounts.room.room_id.as_bytes(),
                    &bump_bytes,
                ]],
            ),
            host_fee,
        )?;
    }

    // Transfer charity donation
    //
    // IMPORTANT: The charity_token_account is NOT validated against GlobalConfig.charity_wallet.
    // This allows the frontend to pass ANY valid token account, enabling support for:
    //
    // 1. The Giving Block (TGB) dynamic wallet addresses - Each transaction can use a different
    //    charity wallet address returned by the TGB API for that specific transaction/intent.
    //
    // 2. Custom charity wallets - Hosts can specify custom charity wallets per-room or per-transaction.
    //
    // 3. Per-room charity configuration - Rooms can have different charity wallets stored in
    //    room.charity_wallet (though this is not currently validated either).
    //
    // Security: The charity_token_account is still validated as a valid SPL TokenAccount by Anchor,
    // but its owner (the actual charity wallet) is not restricted to GlobalConfig.charity_wallet.
    // This provides flexibility for dynamic charity routing while maintaining token account safety.
    if charity_amount > 0 {
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: ctx.accounts.room_vault.to_account_info(),
                    to: ctx.accounts.charity_token_account.to_account_info(),
                    authority: ctx.accounts.room.to_account_info(),
                },
                &[&[
                    b"room".as_ref(),
                    ctx.accounts.room.host.as_ref(),
                    ctx.accounts.room.room_id.as_bytes(),
                    &bump_bytes,
                ]],
            ),
            charity_amount,
        )?;
    }

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

            // Process each unique winner (track which indices we've already handled)
            let mut processed: [bool; 10] = [false; 10];

            for i in 0..winners_to_use.len().min(10) {
                if processed[i] || i >= ctx.accounts.room.prize_distribution.len() {
                    continue;
                }

                let winner_pubkey = winners_to_use[i];
                let winner_distribution = ctx.accounts.room.prize_distribution[i];

                if winner_distribution == 0 {
                    continue;
                }

                // Calculate this position's prize amount
                let mut total_amount =
                    (prize_amount as u128 * winner_distribution as u128 / 100) as u64;

                // Aggregate any duplicate entries for this winner (same pubkey in other positions)
                for j in (i + 1)..winners_to_use.len().min(10) {
                    if !processed[j]
                        && j < ctx.accounts.room.prize_distribution.len()
                        && winners_to_use[j] == winner_pubkey
                    {
                        let additional_amount = (prize_amount as u128
                            * ctx.accounts.room.prize_distribution[j] as u128
                            / 100) as u64;
                        total_amount += additional_amount;
                        processed[j] = true;
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
                    let winner_token_account = anchor_spl::token::TokenAccount::try_deserialize(
                        &mut &token_account_data[..],
                    )?;

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

                    // Transfer aggregated prize amount to winner
                    anchor_spl::token::transfer(
                        CpiContext::new_with_signer(
                            ctx.accounts.token_program.to_account_info(),
                            anchor_spl::token::Transfer {
                                from: ctx.accounts.room_vault.to_account_info(),
                                to: winner_token_account_info.to_account_info(),
                                authority: ctx.accounts.room.to_account_info(),
                            },
                            &[&[
                                b"room".as_ref(),
                                ctx.accounts.room.host.as_ref(),
                                ctx.accounts.room.room_id.as_bytes(),
                                &bump_bytes,
                            ]],
                        ),
                        total_amount,
                    )?;

                    msg!(
                        "   Winner: {} receives {} tokens (aggregated)",
                        winner_pubkey,
                        total_amount
                    );
                }

                processed[i] = true;
            }
        }
        PrizeMode::AssetBased => {
            // Asset-based prize distribution
            // Distribute pre-deposited prize assets to winners
            // remaining_accounts structure for asset mode:
            // [0..winners_count] = winner token accounts for fee_token_mint (for any pool split if applicable)
            // [winners_count..winners_count*2] = winner token accounts for prize assets
            // [winners_count*2..winners_count*2+3] = prize vault accounts

            msg!(
                "Distributing asset-based prizes to {} winners",
                winners_to_use.len()
            );

            let prize_vault_offset = winners_to_use.len() * 2;

            for (prize_index, prize_asset_opt) in ctx.accounts.room.prize_assets.iter().enumerate()
            {
                if let Some(prize_asset) = prize_asset_opt {
                    // Only distribute if prize was deposited and we have a winner for this position
                    if prize_asset.deposited && prize_index < winners_to_use.len() {
                        let winner = winners_to_use[prize_index];
                        let winner_token_account_info =
                            &ctx.remaining_accounts[winners_to_use.len() + prize_index];
                        let prize_vault_info =
                            &ctx.remaining_accounts[prize_vault_offset + prize_index];

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
                        let winner_token_account =
                            anchor_spl::token::TokenAccount::try_deserialize(
                                &mut &winner_token_data[..],
                            )?;

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

                        // Derive prize vault PDA seeds for signing
                        // Note: Prize vaults are controlled by the room PDA
                        let room_key = ctx.accounts.room.key();
                        let prize_index_bytes = [prize_index as u8];

                        let (expected_prize_vault, _prize_vault_bump) =
                            Pubkey::find_program_address(
                                &[b"prize-vault", room_key.as_ref(), &prize_index_bytes],
                                ctx.program_id,
                            );

                        // Verify the prize vault matches expected PDA
                        require!(
                            prize_vault_info.key() == expected_prize_vault,
                            BingoError::InvalidVaultAccount
                        );

                        // Transfer prize asset from prize vault to winner
                        // The prize vault's authority is the room PDA
                        anchor_spl::token::transfer(
                            CpiContext::new_with_signer(
                                ctx.accounts.token_program.to_account_info(),
                                anchor_spl::token::Transfer {
                                    from: prize_vault_info.to_account_info(),
                                    to: winner_token_account_info.to_account_info(),
                                    authority: ctx.accounts.room.to_account_info(),
                                },
                                &[&[
                                    b"room".as_ref(),
                                    ctx.accounts.room.host.as_ref(),
                                    ctx.accounts.room.room_id.as_bytes(),
                                    &bump_bytes,
                                ]],
                            ),
                            prize_asset.amount,
                        )?;

                        msg!(
                            "   Prize {}: Winner {} receives {} of token {}",
                            prize_index + 1,
                            winner,
                            prize_asset.amount,
                            prize_asset.mint
                        );
                    }
                }
            }
        }
    }

    msg!("Room ended and prizes distributed");
    msg!(
        "   Entry fees: {}, Extras: {}, Total pool: {}",
        entry_fees_total,
        extras_total,
        total_pool
    );
    msg!(
        "   Platform: {}, Host: {}, Charity: {}, Prizes: {}",
        platform_fee,
        host_fee,
        charity_amount,
        prize_amount
    );

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

/// Context for ending a room and distributing funds
#[derive(Accounts)]
#[instruction(room_id: String)]
pub struct EndRoom<'info> {
    /// Room PDA account
    #[account(
        mut,
        seeds = [b"room", host.key().as_ref(), room_id.as_bytes()],
        bump = room.bump,
    )]
    pub room: Account<'info, Room>,

    /// Room vault token account (source of funds)
    #[account(mut)]
    pub room_vault: Account<'info, anchor_spl::token::TokenAccount>,

    /// Global configuration PDA
    #[account(seeds = [b"global-config"], bump = global_config.bump)]
    pub global_config: Account<'info, GlobalConfig>,

    /// Platform wallet token account (receives platform fees)
    /// Must be owned by GlobalConfig.platform_wallet
    #[account(mut)]
    pub platform_token_account: Account<'info, anchor_spl::token::TokenAccount>,

    /// Charity wallet token account (receives charity donations)
    ///
    /// IMPORTANT: This account is NOT validated against GlobalConfig.charity_wallet.
    /// The frontend can pass ANY valid SPL TokenAccount, enabling support for:
    /// - The Giving Block (TGB) dynamic wallet addresses (different address per transaction)
    /// - Custom charity wallets per-room or per-transaction
    /// - Per-room charity configuration
    ///
    /// Security: Anchor validates this is a valid TokenAccount, but the owner wallet
    /// is not restricted. This provides flexibility for dynamic charity routing.
    #[account(mut)]
    pub charity_token_account: Account<'info, anchor_spl::token::TokenAccount>,

    /// Host wallet token account (receives host fees)
    /// Must be owned by the room host
    #[account(mut)]
    pub host_token_account: Account<'info, anchor_spl::token::TokenAccount>,

    /// Host account ending the room
    #[account(mut)]
    pub host: Signer<'info>,

    /// Token program for token transfers
    pub token_program: Program<'info, anchor_spl::token::Token>,
}

// Note: EndRoom struct moved to lib.rs for Anchor macro compatibility
