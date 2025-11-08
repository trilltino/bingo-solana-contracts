//! # Join Room Instruction
//!
//! Player entry into an active fundraising room with automatic SPL token transfer and transparent
//! accounting of entry fees vs. charitable extras.
//!
//! ## Overview
//!
//! This instruction allows players to participate in a fundraising room by paying the entry fee
//! plus optional extras. Upon execution, tokens are transferred from the player's wallet to the
//! room's vault, a PlayerEntry receipt PDA is created, and the room's financial counters are updated.
//! This is the core participation mechanism that funds both prizes and charitable donations.
//!
//! ## Role in Program Architecture
//!
//! Join Room is the **third instruction** in the typical game lifecycle:
//!
//! ```text
//! 1. initialize        ← Admin creates GlobalConfig (one-time)
//! 2. init_pool_room    ← Host creates room
//! 3. join_room         ← Players enter room (THIS INSTRUCTION, can happen many times per room)
//! 4. end_room          ← Host distributes funds to winners/charity/platform
//! ```
//!
//! This instruction can be called multiple times (once per player) until the room reaches max_players
//! or expires. Each successful join_room call adds funds to the vault and moves the room one step
//! closer to its fundraising goal.
//!
//! ## What This Instruction Does
//!
//! 1. **Validates Room State**: Ensures room is Ready/Active, not expired, not full, not ended
//! 2. **Calculates Payment**: entry_fee + extras_amount = total_payment
//! 3. **Transfers Tokens**: Moves SPL tokens from player's wallet to room vault via CPI
//! 4. **Creates PlayerEntry PDA**: Immutable receipt using seeds ["player", room, player]
//! 5. **Updates Room Counters**: Increments player_count, total_collected, total_entry_fees, total_extras_fees
//! 6. **Activates Room**: Changes status from Ready → Active on first player join
//! 7. **Emits PlayerJoined Event**: Notifies frontend/indexers of new participant
//!
//! ## Entry Fee vs. Extras
//!
//! Players pay two distinct amounts with different distribution rules:
//!
//! ### Entry Fee (Required)
//! ```text
//! - Fixed amount set by room.entry_fee (e.g., 10 USDC)
//! - Subject to percentage splits:
//!   * Platform: 20%
//!   * Host: 0-5%
//!   * Prizes: 0-35%
//!   * Charity: 40%+ (remainder)
//! - Recorded in Room.total_entry_fees
//! ```
//!
//! ### Extras (Optional)
//! ```text
//! - Voluntary additional amount chosen by player (can be 0)
//! - Goes 100% to charity (no splits)
//! - Maximizes fundraising impact
//! - Recorded in Room.total_extras_fees
//! - Examples:
//!   * extras_amount = 0: Pay exactly entry fee
//!   * extras_amount = 5_000_000: Pay entry + 5 USDC extra to charity
//! ```
//!
//! ### Example Payment Flow
//! ```text
//! Room: entry_fee = 10 USDC, host_fee = 5%, prize_pool = 35%
//! Player A: joins with extras_amount = 0
//!   - Pays: 10 USDC total
//!   - Distribution: Platform 2, Host 0.5, Prizes 3.5, Charity 4
//!
//! Player B: joins with extras_amount = 10 USDC
//!   - Pays: 20 USDC total (10 entry + 10 extras)
//!   - Distribution:
//!     From entry (10): Platform 2, Host 0.5, Prizes 3.5, Charity 4
//!     From extras (10): Charity 10
//!     Total to charity: 14 USDC (70% of Player B's payment!)
//! ```
//!
//! ## State Transitions
//!
//! ### First Player Join
//! ```text
//! Room.status: Ready → Active
//! Room.player_count: 0 → 1
//! Room becomes eligible for end_room
//! ```
//!
//! ### Subsequent Joins
//! ```text
//! Room.status: Active (unchanged)
//! Room.player_count: increments by 1
//! Room.total_collected: increases by (entry_fee + extras_amount)
//! ```
//!
//! ### Room Full
//! ```text
//! When Room.player_count reaches Room.max_players:
//!   - No more joins accepted (MaxPlayersReached error)
//!   - Host can immediately call end_room
//! ```
//!
//! ## Duplicate Prevention
//!
//! PlayerEntry PDA uses deterministic addressing to prevent double-joins:
//!
//! ```text
//! PlayerEntry seeds: ["player", room_pubkey, player_pubkey]
//!
//! If same player attempts to join same room twice:
//!   1. PDA derivation produces identical address
//!   2. Account init fails (account already exists)
//!   3. Transaction reverts automatically
//!
//! This provides trustless duplicate prevention without additional logic.
//! ```
//!
//! ## Token Transfer Security
//!
//! The SPL token transfer uses Cross-Program Invocation (CPI):
//!
//! ```rust
//! anchor_spl::token::transfer(
//!     CpiContext::new(
//!         token_program.to_account_info(),
//!         Transfer {
//!             from: player_token_account,  // Player's SPL token account
//!             to: room_vault,              // Room's vault (PDA-controlled)
//!             authority: player,           // Player signs
//!         }
//!     ),
//!     total_payment
//! )?;
//! ```
//!
//! Critical security properties:
//! - Player must sign the transaction (implicit authorization)
//! - Player's token account must have sufficient balance (enforced by SPL Token program)
//! - Tokens go directly to room_vault (PDA controlled by program, not host)
//! - Transfer and PlayerEntry creation are atomic (both succeed or both fail)
//!
//! ## Frontend Integration
//!
//! The `useFundraiselyContract.ts` hook's `joinRoom()` function calls this instruction:
//!
//! ```typescript
//! const joinRoom = async (roomId: string, extras: number) => {
//!   // Derive PDAs
//!   const [roomPDA] = PublicKey.findProgramAddressSync(
//!     [Buffer.from("room"), hostPubkey.toBuffer(), Buffer.from(roomId)],
//!     program.programId
//!   );
//!
//!   const [playerEntryPDA] = PublicKey.findProgramAddressSync(
//!     [Buffer.from("player"), roomPDA.toBuffer(), wallet.publicKey.toBuffer()],
//!     program.programId
//!   );
//!
//!   const [vaultPDA] = PublicKey.findProgramAddressSync(
//!     [Buffer.from("room-vault"), roomPDA.toBuffer()],
//!     program.programId
//!   );
//!
//!   // Fetch room to get token mint and entry fee
//!   const room = await program.account.room.fetch(roomPDA);
//!
//!   // Get player's associated token account for the room's token mint
//!   const playerTokenAccount = await getAssociatedTokenAddress(
//!     room.feeTokenMint,
//!     wallet.publicKey
//!   );
//!
//!   // Convert extras from UI units to base units (e.g., 5 USDC → 5_000_000)
//!   const extrasAmount = new BN(extras * (10 ** tokenDecimals));
//!
//!   await program.methods
//!     .joinRoom(roomId, extrasAmount)
//!     .accounts({
//!       room: roomPDA,
//!       playerEntry: playerEntryPDA,
//!       roomVault: vaultPDA,
//!       playerTokenAccount,
//!       globalConfig: globalConfigPDA,
//!       player: wallet.publicKey,
//!       tokenProgram: TOKEN_PROGRAM_ID,
//!       systemProgram: SystemProgram.programId,
//!     })
//!     .rpc();
//! };
//! ```
//!
//! ## Validation Rules
//!
//! The instruction enforces these constraints:
//!
//! 1. **Emergency Pause**: Fails if GlobalConfig.emergency_pause is true
//! 2. **Room Status**: Room must be in Ready status (Active also accepted)
//! 3. **Room Expiration**: Current slot must be < expiration_slot (if set)
//! 4. **Room Not Ended**: room.ended must be false
//! 5. **Room Not Full**: player_count must be < max_players
//! 6. **Player Not Joined**: PlayerEntry PDA must not already exist (enforced by init constraint)
//! 7. **Token Mint Match**: player_token_account.mint must equal room.fee_token_mint
//! 8. **Sufficient Balance**: Player must have balance >= (entry_fee + extras_amount)
//!
//! ## Error Conditions
//!
//! This instruction fails if:
//! - Player already joined (PlayerEntry PDA exists)
//! - Room is full (MaxPlayersReached)
//! - Room has expired (RoomExpired)
//! - Room already ended (RoomAlreadyEnded)
//! - Room status is not Ready/Active (RoomNotReady)
//! - Emergency pause active (EmergencyPause)
//! - Insufficient token balance (InsufficientBalance - from SPL Token program)
//! - Wrong token mint (constraint violation)
//! - Arithmetic overflow (ArithmeticOverflow)
//! - Insufficient lamports for PlayerEntry rent
//!
//! ## On-Chain Logs
//!
//! Successful execution emits:
//! ```text
//! Player joined room
//!    Player: <player_pubkey>
//!    Payment: <total_payment> lamports
//!    Room total: <room.total_collected> lamports
//! ```
//!
//! ## Events
//!
//! Emits `PlayerJoined` event containing:
//! - room: Room PDA address
//! - player: Player's pubkey
//! - amount_paid: Total payment (entry + extras)
//! - extras_paid: Just the extras portion
//! - player_count: New player count after this join
//! - timestamp: Unix timestamp of join
//!
//! ## Economic Tracking
//!
//! Room maintains separate counters for transparent accounting:
//!
//! ```rust
//! room.total_entry_fees += room.entry_fee;      // Subject to splits
//! room.total_extras_fees += extras_amount;      // 100% to charity
//! room.total_collected += total_payment;        // Grand total
//! ```
//!
//! This separation enables:
//! - Accurate distribution calculations in end_room
//! - Transparent reporting of charitable impact
//! - Verification that extras go 100% to charity
//!
//! ## Related Files
//!
//! - **state/room.rs**: Room struct that tracks collected funds
//! - **state/player_entry.rs**: PlayerEntry receipt PDA definition
//! - **init_pool_room.rs**: Creates the room players join
//! - **end_room.rs**: Distributes the collected funds
//! - **events.rs**: Defines the PlayerJoined event structure
//! - **lib.rs**: Entry point that routes to this handler
//!
//! ## Security Features
//!
//! - **Atomic Operations**: Token transfer and PlayerEntry creation are atomic (both or neither)
//! - **PDA Duplicate Prevention**: Same player can't join same room twice (enforced by PDA derivation)
//! - **PDA Custody**: Tokens go to program-controlled vault, not host's personal account
//! - **Token Validation**: Mint must match room's fee_token_mint
//! - **Checked Arithmetic**: All additions use checked_add to prevent overflow
//! - **Immutable Receipts**: PlayerEntry PDAs are permanent proof of participation

use anchor_lang::prelude::*;
use crate::state::RoomStatus;
use crate::errors::BingoError;
use crate::events::PlayerJoined;

/// Join a room by paying entry fee
pub fn handler(
    ctx: Context<crate::JoinRoom>,
    _room_id: String,
    extras_amount: u64,
) -> Result<()> {
    let room = &mut ctx.accounts.room;
    let current_slot = Clock::get()?.slot;

    // Check emergency pause
    require!(
        !ctx.accounts.global_config.emergency_pause,
        BingoError::EmergencyPause
    );

    // Check room not ended
    require!(!room.ended, BingoError::RoomAlreadyEnded);

    // Validate amounts
    let total_payment = room.entry_fee.checked_add(extras_amount)
        .ok_or(BingoError::InvalidEntryFee)?;

    // Check if room has expired
    require!(
        room.expiration_slot == 0 || current_slot < room.expiration_slot,
        BingoError::RoomExpired
    );

    require!(
        room.status == RoomStatus::Ready || room.status == RoomStatus::Active,
        BingoError::RoomNotReady
    );

    require!(
        !room.ended,
        BingoError::RoomAlreadyEnded
    );

    // Check if joining is closed
    require!(
        !room.joining_closed,
        BingoError::JoiningClosed
    );

    // Check max players limit
    require!(
        room.player_count < room.max_players,
        BingoError::MaxPlayersReached
    );

    // Transfer tokens from player to room vault
    anchor_spl::token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: ctx.accounts.player_token_account.to_account_info(),
                to: ctx.accounts.room_vault.to_account_info(),
                authority: ctx.accounts.player.to_account_info(),
            },
        ),
        total_payment,
    )?;

    // Create player entry
    let player_entry = &mut ctx.accounts.player_entry;
    player_entry.player = ctx.accounts.player.key();
    player_entry.room = room.key();
    player_entry.entry_paid = room.entry_fee;
    player_entry.extras_paid = extras_amount;
    player_entry.total_paid = total_payment;
    player_entry.join_slot = Clock::get()?.slot;
    player_entry.bump = ctx.bumps.player_entry;

    // Update room state
    room.player_count = room.player_count
        .checked_add(1)
        .ok_or(BingoError::ArithmeticOverflow)?;

    room.total_collected = room.total_collected
        .checked_add(total_payment)
        .ok_or(BingoError::ArithmeticOverflow)?;

    room.total_entry_fees = room.total_entry_fees
        .checked_add(room.entry_fee)
        .ok_or(BingoError::ArithmeticOverflow)?;

    room.total_extras_fees = room.total_extras_fees
        .checked_add(extras_amount)
        .ok_or(BingoError::ArithmeticOverflow)?;

    // Change status to Active when first player joins
    if room.player_count == 1 {
        room.status = RoomStatus::Active;
    }

    msg!("Player joined room");
    msg!("   Player: {}", ctx.accounts.player.key());
    msg!("   Payment: {} lamports", total_payment);
    msg!("   Room total: {} lamports", room.total_collected);

    // Emit event for off-chain indexers and frontend
    emit!(PlayerJoined {
        room: room.key(),
        player: ctx.accounts.player.key(),
        amount_paid: total_payment,
        extras_paid: extras_amount,
        player_count: room.player_count,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

// Note: JoinRoom struct moved to lib.rs for Anchor macro compatibility
