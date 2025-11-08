//! # Init Asset Room Instruction
//!
//! Creates an asset-based fundraising room where prizes are pre-deposited SPL tokens
//! rather than percentages of the entry fee pool.
//!
//! ## Overview
//!
//! Asset-based rooms differ from pool-based rooms:
//! - Host escrows specific SPL token prizes (up to 3) BEFORE players can join
//! - Entry fees go to platform (20%) and charity (75-80%)
//! - Host can take 0-5% of entry fees
//! - Winners receive the pre-escrowed assets

use anchor_lang::prelude::*;
use crate::state::{RoomStatus, PrizeMode, PrizeAsset};
use crate::errors::BingoError;
use crate::events::AssetRoomCreated;
use crate::constants::{
    MAX_ROOM_ID_LENGTH, MIN_ROOM_ID_LENGTH,
    MAX_ENTRY_FEE, MIN_ENTRY_FEE,
    MAX_CHARITY_MEMO_LENGTH, MAX_PLAYER_COUNT,
    BPS_DENOMINATOR
};

/// Create an asset-based room where prizes are pre-deposited tokens
pub fn handler(
    ctx: Context<crate::InitAssetRoom>,
    room_id: String,
    charity_wallet: Pubkey,
    entry_fee: u64,
    max_players: u32,
    host_fee_bps: u16,
    charity_memo: String,
    expiration_slots: Option<u64>,
    // Prize assets [1st, 2nd, 3rd] - mint and amount for each
    prize_1_mint: Pubkey,
    prize_1_amount: u64,
    prize_2_mint: Option<Pubkey>,
    prize_2_amount: Option<u64>,
    prize_3_mint: Option<Pubkey>,
    prize_3_amount: Option<u64>,
) -> Result<()> {
    // Validation
    require!(
        !ctx.accounts.global_config.emergency_pause,
        BingoError::EmergencyPause
    );

    // SECURITY: Vault must be uninitialized to prevent pre-initialization attacks
    // This ensures no malicious actor can create a vault with wrong parameters before room creation
    require!(
        ctx.accounts.room_vault.data_is_empty(),
        BingoError::InvalidVaultAccount
    );

    msg!("Creating room vault token account...");

    // Get rent-exempt balance for token account
    let rent = Rent::get()?;
    const TOKEN_ACCOUNT_SIZE: usize = 165;
    let lamports = rent.minimum_balance(TOKEN_ACCOUNT_SIZE);

    // Create account via CPI to System Program
    let create_account_ix = anchor_lang::solana_program::system_instruction::create_account(
        &ctx.accounts.host.key(),
        &ctx.accounts.room_vault.key(),
        lamports,
        TOKEN_ACCOUNT_SIZE as u64,
        &anchor_spl::token::ID,
    );

    anchor_lang::solana_program::program::invoke_signed(
        &create_account_ix,
        &[
            ctx.accounts.host.to_account_info(),
            ctx.accounts.room_vault.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        &[&[
            b"room-vault",
            ctx.accounts.room.key().as_ref(),
            &[ctx.bumps.room_vault],
        ]],
    )?;

    // Initialize as token account via CPI to Token Program
    // ✅ FIX: Use invoke_signed() with room_vault PDA seeds to provide signer
    // The account needs to be a signer for initialize_account3, and PDAs must sign via seeds
    let init_account_ix = anchor_spl::token::spl_token::instruction::initialize_account3(
        &anchor_spl::token::ID,
        &ctx.accounts.room_vault.key(),
        &ctx.accounts.fee_token_mint.key(),
        &ctx.accounts.room.key(), // owner is the room PDA
    )?;

    anchor_lang::solana_program::program::invoke_signed(
        &init_account_ix,
        &[
            ctx.accounts.room_vault.to_account_info(),
            ctx.accounts.fee_token_mint.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ],
        &[&[
            b"room-vault",
            ctx.accounts.room.key().as_ref(),
            &[ctx.bumps.room_vault],
        ]],
    )?;

    msg!("Room vault created successfully");

    // Validate token is approved in registry
    require!(
        ctx.accounts.token_registry.is_token_approved(&ctx.accounts.fee_token_mint.key()),
        BingoError::TokenNotApproved
    );

    // SECURITY: Validate room_id length (1-32 characters as documented)
    require!(
        room_id.len() >= MIN_ROOM_ID_LENGTH && room_id.len() <= MAX_ROOM_ID_LENGTH,
        BingoError::InvalidRoomId
    );

    // SECURITY: Validate entry_fee has reasonable upper bound to prevent mistakes
    require!(
        entry_fee >= MIN_ENTRY_FEE && entry_fee <= MAX_ENTRY_FEE,
        BingoError::InvalidEntryFee
    );

    // SECURITY: Validate charity_memo length (max 28 characters as documented)
    require!(
        charity_memo.len() <= MAX_CHARITY_MEMO_LENGTH,
        BingoError::InvalidMemo
    );

    // Validate max_players
    require!(
        max_players > 0 && max_players <= MAX_PLAYER_COUNT,
        BingoError::InvalidMaxPlayers
    );

    // Validate host fee (max 5%)
    require!(
        host_fee_bps <= ctx.accounts.global_config.max_host_fee_bps,
        BingoError::HostFeeTooHigh
    );

    // Validate prize amounts
    require!(prize_1_amount > 0, BingoError::InvalidPrizeAmount);
    if let Some(amt) = prize_2_amount {
        require!(amt > 0, BingoError::InvalidPrizeAmount);
    }
    if let Some(amt) = prize_3_amount {
        require!(amt > 0, BingoError::InvalidPrizeAmount);
    }

    // Initialize room
    let room = &mut ctx.accounts.room;
    room.room_id = room_id.clone();
    room.host = ctx.accounts.host.key();
    room.charity_wallet = charity_wallet;
    room.fee_token_mint = ctx.accounts.fee_token_mint.key();
    room.entry_fee = entry_fee;
    room.host_fee_bps = host_fee_bps;
    room.prize_pool_bps = 0; // No prize pool for asset-based rooms

    // Calculate charity percentage (entry fees minus platform and host fees)
    let platform_bps = ctx.accounts.global_config.platform_fee_bps;
    room.charity_bps = (BPS_DENOMINATOR as u16)
        .saturating_sub(platform_bps)
        .saturating_sub(host_fee_bps);

    // Asset-based rooms have higher charity allocation (75-80%)
    msg!("   Platform: {}bps, Host: {}bps, Charity: {}bps",
        platform_bps, host_fee_bps, room.charity_bps);

    room.prize_mode = PrizeMode::AssetBased;
    room.prize_distribution = vec![100, 0, 0]; // Not used for asset-based, but required
    room.status = RoomStatus::AwaitingFunding; // Waiting for prize deposits
    room.player_count = 0;
    room.max_players = max_players;
    room.total_collected = 0;
    room.total_entry_fees = 0;
    room.total_extras_fees = 0;
    room.joining_closed = false;
    room.ended = false;
    room.winners = Vec::new();

    // Set prize asset info (not yet deposited)
    room.prize_assets = [
        Some(PrizeAsset {
            mint: prize_1_mint,
            amount: prize_1_amount,
            deposited: false,
        }),
        prize_2_mint.and_then(|mint| {
            prize_2_amount.map(|amount| PrizeAsset {
                mint,
                amount,
                deposited: false,
            })
        }),
        prize_3_mint.and_then(|mint| {
            prize_3_amount.map(|amount| PrizeAsset {
                mint,
                amount,
                deposited: false,
            })
        }),
    ];

    let current_slot = Clock::get()?.slot;
    room.creation_slot = current_slot;

    // Set expiration slot if specified
    // SECURITY: Use checked_add to prevent overflow, return error instead of silently ignoring
    room.expiration_slot = if let Some(slots) = expiration_slots {
        current_slot.checked_add(slots).ok_or(BingoError::ArithmeticOverflow)?
    } else {
        0 // No expiration
    };

    room.charity_memo = charity_memo;
    room.bump = ctx.bumps.room;

    msg!("Asset room created: {}", room_id);
    msg!("   Entry fee: {} lamports", entry_fee);
    msg!("   Max players: {}", max_players);
    msg!("   Status: AwaitingFunding (host must deposit prizes)");

    // Count expected prizes
    let mut expected_prizes = 1u8; // prize_1 is mandatory
    if prize_2_mint.is_some() { expected_prizes += 1; }
    if prize_3_mint.is_some() { expected_prizes += 1; }

    // Emit event
    emit!(AssetRoomCreated {
        room: room.key(),
        room_id: room_id.clone(),
        host: ctx.accounts.host.key(),
        entry_fee,
        expected_prizes,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

// Note: Account struct is in lib.rs
