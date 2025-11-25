//! # Init Pool Room Instruction
//!
//! Creates a new fundraising game room with pool-based prize distribution where winners receive
//! a percentage of collected entry fees.
//!
//! ## Overview
//!
//! This instruction allows anyone to become a "host" and create a fundraising room. Hosts configure
//! the room's economic parameters (entry fee, prize split, host compensation) within the constraints
//! set by GlobalConfig. Upon creation, a Room PDA and associated token vault are initialized to
//! accept player entries and hold funds until distribution.
//!
//! ## Role in Program Architecture
//!
//! Init Pool Room is the **second instruction** in the typical program flow:
//!
//! ```text
//! 1. initialize        ← Admin creates GlobalConfig (one-time)
//! 2. init_pool_room    ← Host creates room (THIS INSTRUCTION, can happen many times)
//! 3. join_room         ← Players enter the room
//! 4. end_room          ← Host distributes funds and closes room
//! ```
//!
//! This instruction bridges the global configuration with individual game instances. Each room
//! operates independently with its own vault, but all rooms must respect the platform's economic
//! constraints (40% minimum to charity).
//!
//! ## What This Instruction Does
//!
//! 1. **Creates Room PDA**: Initializes a Room account using seeds ["room", host_pubkey, room_id]
//! 2. **Creates Room Vault**: Initializes an SPL token account that holds player funds
//! 3. **Validates Fee Structure**: Ensures host_fee + prize_pool don't violate platform rules
//! 4. **Calculates Charity Allocation**: Computes charity_bps as remainder (40%+ guaranteed)
//! 5. **Sets Room Parameters**: Stores entry fee, max players, prize distribution, expiration
//! 6. **Emits RoomCreated Event**: Notifies frontend/indexers that a new room is available
//!
//! ## Economic Parameters
//!
//! Hosts customize their room within these constraints:
//!
//! ```text
//! Required Inputs:
//!   - entry_fee: Amount players must pay (in token base units, e.g., 1000000 = 1 USDC)
//!   - max_players: Room capacity (1-1000 players)
//!   - host_fee_bps: Host compensation (0-500 = 0-5%)
//!   - prize_pool_bps: Prize pool size (must be > 0)
//!   - [first|second|third]_place_pct: Prize split percentages (must sum to 100)
//!
//! Validation Rules:
//!   - prize_pool_bps must be > 0
//!   - host_fee_bps + prize_pool_bps must not exceed 4000 (40% combined)
//!   - charity_bps = 10000 - platform_fee(2000) - host_fee_bps - prize_pool_bps
//!   - charity_bps must be >= 4000 (40%), enforced by validation
//!
//! Example 1: Generous host maximizing charity
//!   host_fee_bps: 0 (0%)
//!   prize_pool_bps: 2000 (20%)
//!   → charity_bps: 6000 (60%)  ← 60% to charity!
//!
//! Example 2: Competitive room maximizing prizes
//!   host_fee_bps: 500 (5%)
//!   prize_pool_bps: 3500 (35%)
//!   → charity_bps: 4000 (40%)  ← Exactly 40% minimum to charity
//!
//! Example 3: Host takes nothing, maximum for prizes
//!   host_fee_bps: 0 (0%)
//!   prize_pool_bps: 4000 (40%)
//!   → charity_bps: 4000 (40%)  ← Exactly 40% minimum to charity
//! ```
//!
//! ## Prize Distribution
//!
//! Hosts specify how prizes are split among up to 3 winners:
//!
//! ```text
//! - Winner-takes-all: [100, 0, 0]
//! - Top-heavy: [70, 20, 10]
//! - Balanced: [50, 30, 20]
//! - Percentages must sum to exactly 100
//! ```
//!
//! ## Room Expiration
//!
//! Optional expiration prevents rooms from remaining open indefinitely:
//!
//! ```text
//! - expiration_slots: Number of slots after which room expires
//! - Typical value: 43,200 (approximately 24 hours on Solana)
//! - After expiration, anyone can end the room (not just host)
//! - Prevents abandoned rooms from locking player funds
//! - Set to None/0 for no expiration (manual host closure required)
//! ```
//!
//! ## PDA Security
//!
//! Two accounts are created with deterministic addresses:
//!
//! ```text
//! Room PDA:
//!   seeds: ["room", host_pubkey, room_id]
//!   authority: Program (only program can sign for Room operations)
//!
//! Room Vault PDA:
//!   seeds: ["room-vault", room_pda]
//!   authority: Room PDA (only Room can authorize token transfers out)
//! ```
//!
//! This nesting (Program controls Room, Room controls Vault) ensures trustless fund custody.
//! Players' tokens are safe from host tampering; only the end_room instruction can move funds.
//! Any unused prize pool percentage goes directly to charity.
//!
//! ## Frontend Integration
//!
//! The `useFundraiselyContract.ts` hook's `createRoom()` function calls this instruction:
//!
//! ```typescript
//! const createRoom = async (params: CreateRoomParams) => {
//!   const [roomPDA] = PublicKey.findProgramAddressSync(
//!     [
//!       Buffer.from("room"),
//!       wallet.publicKey.toBuffer(),
//!       Buffer.from(params.roomId)
//!     ],
//!     program.programId
//!   );
//!
//!   const [vaultPDA] = PublicKey.findProgramAddressSync(
//!     [Buffer.from("room-vault"), roomPDA.toBuffer()],
//!     program.programId
//!   );
//!
//!   await program.methods
//!     .initPoolRoom(
//!       params.roomId,
//!       params.entryFee,
//!       params.maxPlayers,
//!       params.hostFeeBps,
//!       params.prizePoolBps,
//!       params.firstPlacePct,
//!       params.secondPlacePct,
//!       params.thirdPlacePct,
//!       params.charityMemo,
//!       params.expirationSlots
//!     )
//!     .accounts({
//!       room: roomPDA,
//!       roomVault: vaultPDA,
//!       feeTokenMint: params.tokenMint,
//!       globalConfig: globalConfigPDA,
//!       host: wallet.publicKey,
//!       systemProgram: SystemProgram.programId,
//!       tokenProgram: TOKEN_PROGRAM_ID,
//!       rent: SYSVAR_RENT_PUBKEY,
//!     })
//!     .rpc();
//! };
//! ```
//!
//! ## Validation Rules
//!
//! The instruction enforces these constraints:
//!
//! 1. **Emergency Pause Check**: Fails if GlobalConfig.emergency_pause is true
//! 2. **Room ID Length**: 1-32 characters (prevents storage bloat)
//! 3. **Entry Fee**: Must be > 0 (free rooms not allowed)
//! 4. **Max Players**: 1-1000 (prevents DoS via unbounded storage)
//! 5. **Host Fee**: 0-500 bps (0-5%, enforced by GlobalConfig.max_host_fee_bps)
//! 6. **Prize Pool**: Must be > 0 and host_fee + prize_pool ≤ 4000 bps (40%)
//! 7. **Prize Distribution**: first + second + third = 100 exactly
//! 8. **Charity Minimum**: charity_bps >= 4000 (40%, enforced by GlobalConfig.min_charity_bps)
//!
//! ## Error Conditions
//!
//! This instruction fails if:
//! - Room with same (host, room_id) already exists
//! - Host fee exceeds 5% (HostFeeTooHigh)
//! - Prize pool is 0 (PrizePoolTooLow)
//! - Host fee + prize pool exceeds 40% (PrizePoolTooHigh)
//! - Charity would be below 40% (CharityBelowMinimum)
//! - Prize distribution doesn't sum to 100 (InvalidPrizeDistribution)
//! - Invalid room_id length (InvalidRoomId)
//! - Invalid entry_fee (InvalidEntryFee)
//! - Invalid max_players (InvalidMaxPlayers)
//! - Emergency pause is active (EmergencyPause)
//! - Insufficient lamports for rent
//!
//! ## On-Chain Logs
//!
//! Successful execution emits:
//! ```text
//! Pool room created: <room_id>
//!    Entry fee: <entry_fee> lamports
//!    Max players: <max_players>
//!    Host fee: <host_fee_bps>bps, Prize pool: <prize_pool_bps>bps, Charity: <charity_bps>bps
//! ```
//!
//! ## Events
//!
//! Emits `RoomCreated` event containing:
//! - room: Room PDA address
//! - room_id: Human-readable identifier
//! - host: Host's pubkey
//! - entry_fee, max_players: Configuration
//! - expiration_slot: When room expires (0 = never)
//! - timestamp: Unix timestamp of creation
//!
//! ## Related Files
//!
//! - **state/room.rs**: Defines the Room struct and its data layout
//! - **state/global_config.rs**: Defines economic constraint limits
//! - **join_room.rs**: Handles player entry after room creation
//! - **end_room.rs**: Handles room closure and fund distribution
//! - **events.rs**: Defines the RoomCreated event structure
//! - **lib.rs**: Entry point that routes to this handler
//!
//! ## Security Features
//!
//! - **PDA Authority**: Room vault can only be controlled by Room PDA (program authority)
//! - **Economic Constraints**: Impossible to create room violating charity minimum
//! - **Capacity Limits**: max_players capped at 1000 prevents storage DoS
//! - **Input Validation**: All parameters validated before state changes
//! - **Deterministic Addressing**: Room addresses derived from (host + room_id) prevent collisions

use crate::errors::BingoError;
use crate::events::RoomCreated;
use crate::state::{PrizeMode, RoomStatus};
use crate::validation::{AmountValidator, EmergencyGuard, InputValidator};
use anchor_lang::prelude::*;

/// Create a pool-based room where prizes come from entry fee pool
pub fn handler(
    ctx: Context<InitPoolRoom>,
    room_id: String,
    charity_wallet: Pubkey,
    entry_fee: u64,
    max_players: u32,
    host_fee_bps: u16,
    prize_pool_bps: u16,
    first_place_pct: u16,
    second_place_pct: Option<u16>,
    third_place_pct: Option<u16>,
    charity_memo: String,
    expiration_slots: Option<u64>,
) -> Result<()> {
    // Security checks
    EmergencyGuard::check_not_paused(ctx.accounts.global_config.emergency_pause)?;
    InputValidator::validate_room_id(&room_id)?;
    InputValidator::validate_charity_memo(&charity_memo)?;
    InputValidator::validate_max_players(max_players)?;
    AmountValidator::validate_entry_fee(entry_fee)?;

    // Initialize vault if it doesn't exist
    if ctx.accounts.room_vault.data_is_empty() {
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
        let init_account_ix = anchor_spl::token::spl_token::instruction::initialize_account3(
            &anchor_spl::token::ID,
            &ctx.accounts.room_vault.key(),
            &ctx.accounts.fee_token_mint.key(),
            &ctx.accounts.room.key(), // owner is the room PDA
        )?;

        anchor_lang::solana_program::program::invoke(
            &init_account_ix,
            &[
                ctx.accounts.room_vault.to_account_info(),
                ctx.accounts.fee_token_mint.to_account_info(),
                ctx.accounts.rent.to_account_info(),
            ],
        )?;

        msg!("Room vault created successfully");
    } else {
        // SECURITY: Validate existing vault is a proper TokenAccount
        use anchor_spl::token::TokenAccount;

        let vault_data = ctx
            .accounts
            .room_vault
            .try_borrow_data()
            .map_err(|_| BingoError::InvalidVaultAccount)?;

        let vault_account = TokenAccount::try_deserialize(&mut vault_data.as_ref())
            .map_err(|_| BingoError::InvalidVaultAccount)?;

        // Verify vault has correct mint
        require!(
            vault_account.mint == ctx.accounts.fee_token_mint.key(),
            BingoError::InvalidTokenMint
        );

        // Verify vault authority is the room PDA
        require!(
            vault_account.owner == ctx.accounts.room.key(),
            BingoError::InvalidVaultAuthority
        );
    }

    // Validate token is approved in registry
    require!(
        ctx.accounts
            .token_registry
            .is_token_approved(&ctx.accounts.fee_token_mint.key()),
        BingoError::TokenNotApproved
    );

    // Validate host fee (max 5%)
    require!(
        host_fee_bps <= ctx.accounts.global_config.max_host_fee_bps,
        BingoError::HostFeeTooHigh
    );

    // Validate prize pool must be greater than 0
    require!(prize_pool_bps > 0, BingoError::PrizePoolTooLow);

    // Validate combined host + prizes does not exceed 40%
    const MAX_COMBINED_BPS: u16 = 4000; // 40% max for host + prizes combined
    require!(
        host_fee_bps.saturating_add(prize_pool_bps) <= MAX_COMBINED_BPS,
        BingoError::PrizePoolTooHigh
    );

    // Validate prize distribution sums to 100
    let total_prize_pct =
        first_place_pct + second_place_pct.unwrap_or(0) + third_place_pct.unwrap_or(0);
    require!(total_prize_pct == 100, BingoError::InvalidPrizeDistribution);

    // Initialize room
    let room = &mut ctx.accounts.room;
    room.room_id = room_id.clone();
    room.host = ctx.accounts.host.key();
    room.charity_wallet = charity_wallet;
    room.fee_token_mint = ctx.accounts.fee_token_mint.key();
    room.entry_fee = entry_fee;
    room.host_fee_bps = host_fee_bps;
    room.prize_pool_bps = prize_pool_bps;

    // Calculate charity percentage (remainder after platform + host + prizes)
    let platform_bps = ctx.accounts.global_config.platform_fee_bps;
    room.charity_bps = 10000_u16
        .saturating_sub(platform_bps)
        .saturating_sub(host_fee_bps)
        .saturating_sub(prize_pool_bps);

    // Enforce minimum charity allocation (40%)
    require!(
        room.charity_bps >= ctx.accounts.global_config.min_charity_bps,
        BingoError::CharityBelowMinimum
    );

    room.prize_mode = PrizeMode::PoolSplit;
    room.prize_distribution = vec![
        first_place_pct,
        second_place_pct.unwrap_or(0),
        third_place_pct.unwrap_or(0),
    ];
    room.status = RoomStatus::Ready;
    room.player_count = 0;
    room.max_players = max_players;
    room.total_collected = 0;
    room.total_entry_fees = 0;
    room.total_extras_fees = 0;
    room.joining_closed = false;
    room.ended = false;
    room.winners = Vec::new(); // Winners not yet declared
    room.prize_assets = [None, None, None]; // No asset prizes for pool-based rooms

    let current_slot = Clock::get()?.slot;
    room.creation_slot = current_slot;

    // Set expiration slot if specified
    room.expiration_slot = if let Some(slots) = expiration_slots {
        current_slot.checked_add(slots).unwrap_or(0)
    } else {
        0 // No expiration
    };

    room.charity_memo = charity_memo;
    room.bump = ctx.bumps.room;

    msg!("Pool room created: {}", room_id);
    msg!("   Entry fee: {} lamports", entry_fee);
    msg!("   Max players: {}", max_players);
    msg!(
        "   Host fee: {}bps, Prize pool: {}bps, Charity: {}bps",
        host_fee_bps,
        prize_pool_bps,
        room.charity_bps
    );

    // Emit event for off-chain indexers and frontend
    emit!(RoomCreated {
        room: room.key(),
        room_id: room_id.clone(),
        host: ctx.accounts.host.key(),
        entry_fee,
        max_players,
        expiration_slot: room.expiration_slot,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

/// Context for initializing a pool-based room
#[derive(Accounts)]
#[instruction(room_id: String)]
pub struct InitPoolRoom<'info> {
    /// Room PDA account
    #[account(
        init,
        payer = host,
        space = Room::LEN,
        seeds = [b"room", host.key().as_ref(), room_id.as_bytes()],
        bump
    )]
    pub room: Account<'info, Room>,

    /// CHECK: Room vault PDA - using AccountInfo because vault is created in same instruction as room. Handler validates TokenAccount structure manually.
    #[account(
        mut,
        seeds = [b"room-vault", room.key().as_ref()],
        bump
    )]
    pub room_vault: AccountInfo<'info>,

    /// Token mint for entry fees
    pub fee_token_mint: Account<'info, anchor_spl::token::Mint>,

    /// Token registry PDA
    #[account(
        seeds = [b"token-registry-v4"],
        bump = token_registry.bump
    )]
    pub token_registry: Account<'info, TokenRegistry>,

    /// Global configuration PDA
    #[account(
        seeds = [b"global-config"],
        bump = global_config.bump
    )]
    pub global_config: Account<'info, GlobalConfig>,

    /// Host account creating the room
    #[account(mut)]
    pub host: Signer<'info>,

    /// System program for account creation
    pub system_program: Program<'info, System>,

    /// Token program for token account initialization
    pub token_program: Program<'info, anchor_spl::token::Token>,

    /// Rent sysvar for account sizing
    pub rent: Sysvar<'info, Rent>,
}
