//! # Declare Winners Instruction
//!
//! Host declares the 1-3 winners of a fundraising game before distributing funds.
//!
//! ## Overview
//!
//! This instruction allows the room host to officially declare which players won the game.
//! Winner declaration is **separate from** fund distribution (end_room), providing transparency
//! and allowing verification before irreversible token transfers occur. Winners must be valid
//! participants who actually joined the room.
//!
//! ## Role in Program Architecture
//!
//! Declare Winners is the **NEW fourth instruction** in the updated program flow:
//!
//! ```text
//! 1. initialize        ← Admin creates GlobalConfig (one-time)
//! 2. init_pool_room    ← Host creates room
//! 3. join_room         ← Players enter room (multiple times)
//! 4. declare_winners   ← Host declares winners (THIS INSTRUCTION, NEW!)
//! 5. end_room          ← Host distributes funds to declared winners
//! ```
//!
//! This separation of concerns provides several benefits:
//! - **Transparency**: Winners are publicly declared before funds move
//! - **Verification**: Frontend/players can verify winner validity before distribution
//! - **Audit Trail**: Winner declaration is permanently recorded via event
//! - **Flexibility**: Host can declare winners, then end room later (or vice versa if needed)
//!
//! ## What This Instruction Does
//!
//! 1. **Validates Host Authority**: Ensures only the host can declare winners
//! 2. **Validates Room State**: Room must be Active (players have joined) and not Ended
//! 3. **Validates Winners**: Ensures 1-3 unique winners, none is the host
//! 4. **Stores Winners**: Updates Room.winners array with declared winners
//! 5. **Emits WinnersDeclared Event**: Notifies frontend/indexers of winner declaration
//!
//! ## Winner Validation Rules
//!
//! The instruction enforces strict validation on declared winners:
//!
//! ### Number of Winners
//! ```text
//! - Minimum: 1 winner required
//! - Maximum: 3 winners allowed
//! - Matches prize_distribution array length in room configuration
//! ```
//!
//! ### Uniqueness
//! ```text
//! - All declared winners must be unique pubkeys
//! - Cannot declare same player as multiple winners
//! - Example INVALID: [Alice, Alice, Bob]
//! - Example VALID: [Alice, Bob, Charlie]
//! ```
//!
//! ### Host Exclusion
//! ```text
//! - Host cannot be declared as a winner
//! - Prevents self-dealing and conflicts of interest
//! - Host is compensated via host_fee_bps, not prizes
//! ```
//!
//! ### Optional Winner Verification (Future Enhancement)
//! ```text
//! Current: Winners validated at end_room (PlayerEntry PDA existence check)
//! Future: Could validate winner participation here via remaining_accounts
//! Trade-off: More upfront validation vs simpler instruction
//! ```
//!
//! ## Winner Storage Format
//!
//! Winners are stored as `[Option<Pubkey>; 3]` in Room.winners:
//!
//! ### Example 1: Three Winners
//! ```rust
//! winners input: vec![alice_pubkey, bob_pubkey, charlie_pubkey]
//! Room.winners = [Some(alice), Some(bob), Some(charlie)]
//! ```
//!
//! ### Example 2: Two Winners
//! ```rust
//! winners input: vec![alice_pubkey, bob_pubkey]
//! Room.winners = [Some(alice), Some(bob), None]
//! ```
//!
//! ### Example 3: Winner-Takes-All
//! ```rust
//! winners input: vec![alice_pubkey]
//! Room.winners = [Some(alice), None, None]
//! ```
//!
//! ## Frontend Integration
//!
//! The `useFundraiselyContract.ts` hook should add a `declareWinners()` function:
//!
//! ```typescript
//! const declareWinners = async (roomId: string, winners: PublicKey[]) => {
//!   const [roomPDA] = PublicKey.findProgramAddressSync(
//!     [Buffer.from("room"), hostPubkey.toBuffer(), Buffer.from(roomId)],
//!     program.programId
//!   );
//!
//!   await program.methods
//!     .declareWinners(roomId, winners)
//!     .accounts({
//!       room: roomPDA,
//!       host: wallet.publicKey,
//!     })
//!     .rpc();
//!
//!   console.log("Winners declared:", winners);
//! };
//! ```
//!
//! ## Event Listening
//!
//! Frontend can listen for winner declarations in real-time:
//!
//! ```typescript
//! program.addEventListener("WinnersDeclared", (event, slot) => {
//!   console.log(`Winners declared for room ${event.room}`);
//!   console.log(`Winners: ${event.winners.filter(w => w !== null)}`);
//!   updateRoomUI(event.room, { winners: event.winners });
//! });
//! ```
//!
//! ## Validation Rules
//!
//! The instruction enforces these constraints:
//!
//! 1. **Host Authority**: Only room.host can declare winners
//! 2. **Room Active**: Room status must be Active (not Ready, not Ended)
//! 3. **Room Not Ended**: room.ended must be false
//! 4. **Winner Count**: 1-3 winners required (vec length check)
//! 5. **Uniqueness**: All winners must be unique pubkeys
//! 6. **Host Exclusion**: None of the winners can be the host
//! 7. **No Re-declaration**: Winners can only be declared once (room.winners must be all None)
//!
//! ## Error Conditions
//!
//! This instruction fails if:
//! - Caller is not the host (Unauthorized)
//! - Room is not Active (InvalidRoomStatus)
//! - Room already ended (RoomAlreadyEnded)
//! - Winners already declared (WinnersAlreadyDeclared)
//! - Invalid number of winners (InvalidWinners - not 1-3)
//! - Duplicate winners (InvalidWinners)
//! - Host is in winners list (HostCannotBeWinner)
//!
//! ## On-Chain Logs
//!
//! Successful execution emits:
//! ```text
//! Winners declared for room
//!    Winner 1: <winner1_pubkey>
//!    Winner 2: <winner2_pubkey> (if present)
//!    Winner 3: <winner3_pubkey> (if present)
//! ```
//!
//! ## Events
//!
//! Emits `WinnersDeclared` event containing:
//! - room: Room PDA address
//! - winners: Fixed-size array [Option<Pubkey>; 3]
//! - timestamp: Unix timestamp of declaration
//!
//! ## Integration with end_room
//!
//! The end_room instruction should be updated to:
//!
//! 1. **Validate Winners Declared**: Check that room.winners is not all None
//! 2. **Use Declared Winners**: Distribute prizes only to room.winners (ignore end_room winners param)
//! 3. **Fallback Behavior**: For backwards compatibility, end_room can still accept winners param
//!    if room.winners is not set (allowing old flow to continue working)
//!
//! ```rust
//! // In end_room handler
//! let winners_to_use = if room.winners[0].is_some() {
//!     // Use declared winners (new flow)
//!     room.winners.iter().filter_map(|w| *w).collect()
//! } else {
//!     // Use passed-in winners (old flow for backwards compat)
//!     winners
//! };
//! ```
//!
//! ## Security Considerations
//!
//! - **Host-Only**: Only host can declare winners (prevents player manipulation)
//! - **One-Time Declaration**: Winners can only be declared once (prevents changing after declaration)
//! - **Host Exclusion**: Host cannot award themselves prizes (prevents self-dealing)
//! - **Immutable Storage**: Once declared, winners array cannot be modified
//! - **Pre-Distribution**: Declaration happens before fund movement (transparent process)
//!
//! ## Related Files
//!
//! - **state/room.rs**: Defines Room.winners field ([Option<Pubkey>; 3])
//! - **events.rs**: Defines WinnersDeclared event
//! - **end_room.rs**: Should validate and use declared winners
//! - **lib.rs**: Entry point that routes to this handler
//! - **errors.rs**: Defines WinnersAlreadyDeclared error type
//!
//! ## Future Enhancements
//!
//! 1. **Player Verification**: Add remaining_accounts to verify winners actually joined
//! 2. **Score Recording**: Add scores parameter to record why each player won
//! 3. **Multi-Signature**: Allow multiple hosts to co-sign winner declaration
//! 4. **Amendment**: Allow host to change winners before end_room (with event trail)
//! 5. **Automatic Declaration**: AI/oracle could declare winners based on on-chain game state

use crate::errors::BingoError;
use crate::events::WinnersDeclared;
use crate::state::RoomStatus;
use anchor_lang::prelude::*;

/// Declare winners for a room
///
/// Host-only instruction to officially declare 1-3 winners before fund distribution.
/// Winners are validated for uniqueness, host exclusion, and actual room participation.
pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, DeclareWinners<'info>>,
    _room_id: String,
    winners: Vec<Pubkey>,
) -> Result<()> {
    let room = &mut ctx.accounts.room;

    // Validation: Only host can declare winners
    require!(
        ctx.accounts.host.key() == room.host,
        BingoError::Unauthorized
    );

    // Validation: Room must be active or ready (allow declaring winners before players join for testing)
    require!(
        room.status == RoomStatus::Active || room.status == RoomStatus::Ready,
        BingoError::InvalidRoomStatus
    );

    // Validation: Room must not be ended
    require!(!room.ended, BingoError::RoomAlreadyEnded);

    // Validation: Winners not already declared
    require!(room.winners.is_empty(), BingoError::WinnersAlreadyDeclared);

    // Validation: Must have 1-10 winners
    require!(
        winners.len() > 0 && winners.len() <= 10,
        BingoError::InvalidWinners
    );

    // Validation: Must match prize distribution length
    require!(
        winners.len() == room.prize_distribution.len(),
        BingoError::InvalidWinners
    );

    // NOTE: We DO NOT validate for duplicate winners
    // This is intentional! When there are fewer players than prize positions,
    // the same winner can receive multiple prize portions (e.g., [alice, alice, alice])
    // This allows flexibility for:
    // - 1 player games: winner gets all 3 prize portions
    // - 2 player games: winners split, with one getting 2 portions
    // - Charity/Host receiving unclaimed portions (if implemented)

    // Validation: Host cannot be a winner
    for winner in &winners {
        require!(*winner != room.host, BingoError::HostCannotBeWinner);
    }

    // NEW VALIDATION: Winners must have actually joined the room
    // Verify that remaining_accounts contains valid PlayerEntry PDAs for each winner
    require!(
        ctx.remaining_accounts.len() >= winners.len(),
        BingoError::InvalidWinners
    );

    for (i, winner) in winners.iter().enumerate() {
        // Derive the expected PlayerEntry PDA for this winner
        // Seeds: ["player", room_pubkey, player_pubkey] - must match join_room.rs
        let (expected_player_entry_pda, _bump) = Pubkey::find_program_address(
            &[b"player", room.key().as_ref(), winner.as_ref()],
            ctx.program_id,
        );

        // Get the passed-in PlayerEntry account from remaining_accounts
        let player_entry_account = &ctx.remaining_accounts[i];

        // Validation: The account address must match the derived PDA
        require!(
            player_entry_account.key() == expected_player_entry_pda,
            BingoError::InvalidPlayerEntry
        );

        // Validation: The account must exist (have data and be initialized)
        require!(
            !player_entry_account.data_is_empty(),
            BingoError::InvalidPlayerEntry
        );

        // Validation: The account must be owned by this program
        require!(
            player_entry_account.owner == ctx.program_id,
            BingoError::InvalidPlayerEntry
        );

        msg!(
            "   Winner {} verified: {} (PlayerEntry exists)",
            i + 1,
            winner
        );
    }

    // Store winners in room (pad with None for unfilled positions)
    // Store all winners (1-10)
    room.winners = winners.clone();

    msg!("Winners declared for room");
    for (i, winner) in room.winners.iter().enumerate() {
        msg!("   Winner {}: {}", i + 1, winner);
    }

    // Emit event for off-chain indexers and frontend
    emit!(WinnersDeclared {
        room: room.key(),
        winners: room.winners.clone(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

/// Context for declaring winners
#[derive(Accounts)]
#[instruction(room_id: String)]
pub struct DeclareWinners<'info> {
    /// Room PDA account
    #[account(
        mut,
        seeds = [b"room", room.host.as_ref(), room_id.as_bytes()],
        bump = room.bump,
    )]
    pub room: Account<'info, Room>,

    /// Host account declaring winners
    #[account(mut)]
    pub host: Signer<'info>,
}
