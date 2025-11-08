//! # Add Prize Asset Instruction
//!
//! Escrows a prize asset into the room's prize vault for asset-based rooms

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};
use crate::state::RoomStatus;
use crate::errors::BingoError;
use crate::events::PrizeAssetDeposited;

/// Escrow a prize asset into the room
pub fn handler(
    ctx: Context<crate::AddPrizeAsset>,
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

    // Manually derive prize vault PDA and validate it matches the provided account
    let (expected_prize_vault, bump) = Pubkey::find_program_address(
        &[
            b"prize-vault",
            room_key.as_ref(),
            &[prize_index],
        ],
        ctx.program_id,
    );
    require!(
        ctx.accounts.prize_vault.key() == expected_prize_vault,
        BingoError::InvalidVaultAccount
    );

    // Get prize asset info
    let prize_asset = room.prize_assets[prize_index as usize]
        .as_mut()
        .ok_or(BingoError::InvalidWinners)?;

    // Check not already deposited
    require!(!prize_asset.deposited, BingoError::PrizeAlreadyDeposited);

    // Store mint and amount before further borrows
    let prize_mint = prize_asset.mint;
    let prize_amount = prize_asset.amount;

    // Verify prize mint matches
    require!(
        ctx.accounts.prize_mint.key() == prize_mint,
        BingoError::InvalidTokenMint
    );

    // Initialize prize vault if it doesn't exist or isn't a TokenAccount yet
    // UncheckedAccount allows us to accept uninitialized accounts
    let prize_vault_info = ctx.accounts.prize_vault.to_account_info();
    let needs_init = {
        if prize_vault_info.data_is_empty() {
            true
        } else {
            // Check if it's already a valid TokenAccount
            use anchor_spl::token::TokenAccount;
            match TokenAccount::try_deserialize(&mut prize_vault_info.data.borrow().as_ref()) {
                Ok(_) => false, // Already a TokenAccount
                Err(_) => true, // Not a TokenAccount, needs initialization
            }
        }
    };

    if needs_init {
        msg!("Creating and initializing prize vault token account...");

        // Step 1: Create the account if it doesn't exist
        if prize_vault_info.data_is_empty() {
            msg!("Creating prize vault account...");

            // Get rent-exempt balance for token account
            let rent = Rent::get()?;
            const TOKEN_ACCOUNT_SIZE: usize = 165;
            let lamports = rent.minimum_balance(TOKEN_ACCOUNT_SIZE);

            // Create account via CPI to System Program
            let create_account_ix = anchor_lang::solana_program::system_instruction::create_account(
                &ctx.accounts.host.key(), // payer
                &prize_vault_info.key(),
                lamports,
                TOKEN_ACCOUNT_SIZE as u64,
                &anchor_spl::token::ID,
            );

            anchor_lang::solana_program::program::invoke_signed(
                &create_account_ix,
                &[
                    ctx.accounts.host.to_account_info(),
                    prize_vault_info.clone(),
                    ctx.accounts.system_program.to_account_info(),
                ],
                &[&[
                    b"prize-vault",
                    room_key.as_ref(),
                    &[prize_index],
                    &[bump],
                ]],
            )?;

            msg!("Prize vault account created");
        }

        // Step 2: Initialize as token account via CPI to Token Program
        // ✅ FIX: Use invoke_signed() with prize_vault PDA seeds to provide signer
        let init_account_ix = anchor_spl::token::spl_token::instruction::initialize_account3(
            &anchor_spl::token::ID,
            &prize_vault_info.key(),
            &ctx.accounts.prize_mint.key(),
            &room_key, // owner is the room PDA
        )?;

        anchor_lang::solana_program::program::invoke_signed(
            &init_account_ix,
            &[
                prize_vault_info.clone(),
                ctx.accounts.prize_mint.to_account_info(),
                ctx.accounts.rent.to_account_info(),
            ],
            &[&[
                b"prize-vault",
                room_key.as_ref(),
                &[prize_index],
                &[bump],
            ]],
        )?;

        msg!("Prize vault initialized successfully");
    } else {
        // SECURITY: Validate existing vault is a proper TokenAccount
        use anchor_spl::token::TokenAccount;

        let vault_data = prize_vault_info.try_borrow_data()
            .map_err(|_| BingoError::InvalidVaultAccount)?;

        let vault_account = TokenAccount::try_deserialize(&mut vault_data.as_ref())
            .map_err(|_| BingoError::InvalidVaultAccount)?;

        // Verify vault has correct mint
        require!(
            vault_account.mint == ctx.accounts.prize_mint.key(),
            BingoError::InvalidTokenMint
        );

        // Verify vault authority is the room PDA
        require!(
            vault_account.owner == room_key,
            BingoError::InvalidVaultAuthority
        );
    }

    // Transfer tokens from host to prize vault
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.host_token_account.to_account_info(),
            to: prize_vault_info.clone(),
            authority: ctx.accounts.host.to_account_info(),
        },
    );
    token::transfer(cpi_ctx, prize_asset.amount)?;

    // Mark as deposited
    prize_asset.deposited = true;

    msg!("Prize {} deposited: {} tokens", prize_index + 1, prize_asset.amount);

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
    let all_deposited = room.prize_assets.iter().all(|asset| {
        asset.as_ref().map_or(true, |a| a.deposited)
    });

    if all_deposited {
        room.status = RoomStatus::Ready;
        msg!("   All prizes deposited - room is now Ready for players");
    } else {
        room.status = RoomStatus::PartiallyFunded;
        msg!("   Status: PartiallyFunded (more prizes needed)");
    }

    Ok(())
}
