//! # Player Entry State
//!
//! Individual player participation records for the Fundraisely smart contract.
//!
//! ## Overview
//!
//! PlayerEntry is a Program Derived Address (PDA) that records a single player's participation
//! in a specific room. It serves as an immutable receipt of entry, tracking the exact amounts
//! paid and when the player joined. Each player can only have one entry per room, enforced by
//! the PDA derivation scheme.
//!
//! ## Architecture Role
//!
//! PlayerEntry accounts serve as:
//!
//! 1. **Participation Proof**: Immutable on-chain record that player joined a specific room
//! 2. **Payment Receipt**: Transparent accounting of entry fees and extras paid
//! 3. **Duplicate Prevention**: PDA derivation ensures one entry per player per room
//! 4. **Historical Record**: Permanent audit trail for all participants
//!
//! ## PDA Derivation
//!
//! ```text
//! Seeds: ["player", room_pubkey, player_pubkey]
//! Bump: Stored in PlayerEntry.bump
//! ```
//!
//! This derivation scheme guarantees:
//! - Unique entry per player per room (attempting duplicate join fails at PDA creation)
//! - Deterministic addresses for efficient lookups
//! - Association with specific room and player
//!
//! ## Economic Tracking
//!
//! PlayerEntry separates payments into two categories for transparent accounting:
//!
//! ### Entry Fee (entry_paid)
//! - The base entry fee required to join the room
//! - Subject to percentage split (platform, host, prize, charity)
//! - Fixed amount per room (room.entry_fee)
//!
//! ### Extras (extras_paid)
//! - Optional additional payment beyond entry fee
//! - Goes 100% to charity (no splits)
//! - Amount chosen by player (can be 0)
//! - Maximizes fundraising impact
//!
//! ### Total Payment (total_paid)
//! - Sum of entry_paid + extras_paid
//! - Actual amount transferred from player's token account
//! - Used for receipt and accounting purposes
//!
//! Example:
//! ```text
//! Room entry fee: 10 USDC
//! Player chooses to pay extra: 5 USDC
//!
//! PlayerEntry:
//!   entry_paid = 10 USDC
//!   extras_paid = 5 USDC
//!   total_paid = 15 USDC
//!
//! Distribution:
//!   From entry_paid (10 USDC):
//!     Platform: 2 USDC (20%)
//!     Host: 0.5 USDC (5%)
//!     Prize: 3.5 USDC (35%)
//!     Charity: 4 USDC (40%)
//!
//!   From extras_paid (5 USDC):
//!     Charity: 5 USDC (100%)
//!
//!   Total to charity: 9 USDC (60% of total payment)
//! ```
//!
//! ## Lifecycle
//!
//! ### Creation
//! - Player calls `join_room` instruction
//! - Program validates room status and player eligibility
//! - SPL tokens transferred from player to room vault
//! - PlayerEntry PDA created with payment details
//!
//! ### Immutability
//! - Once created, PlayerEntry never changes
//! - Serves as permanent receipt of participation
//! - Cannot be closed or modified (even after room ends)
//!
//! ### Purpose After Room End
//! - Historical record for players to prove participation
//! - Audit trail for charity transparency
//! - Data source for analytics and reporting
//!
//! ## Join Slot Tracking
//!
//! - **join_slot**: Solana slot when player joined
//! - Used for:
//!   - Ordering players (first-come-first-served logic)
//!   - Time-based analytics
//!   - Verification that player joined before room ended
//!
//! Solana slots increment approximately every 400ms, providing precise temporal ordering.
//!
//! ## Frontend Integration
//!
//! The `useFundraiselyContract.ts` hook interacts with PlayerEntry accounts:
//!
//! ### Checking Participation
//! ```typescript
//! const [playerEntryPDA] = PublicKey.findProgramAddressSync(
//!   [
//!     Buffer.from("player"),
//!     roomPDA.toBuffer(),
//!     playerWallet.toBuffer()
//!   ],
//!   program.programId
//! );
//!
//! try {
//!   const playerEntry = await program.account.playerEntry.fetch(playerEntryPDA);
//!   console.log("Already joined:", playerEntry);
//! } catch {
//!   console.log("Not yet joined");
//! }
//! ```
//!
//! ### Joining Room
//! ```typescript
//! const extrasAmount = new BN(5_000_000); // 5 USDC (6 decimals)
//!
//! await program.methods
//!   .joinRoom(roomId, extrasAmount)
//!   .accounts({
//!     room: roomPDA,
//!     playerEntry: playerEntryPDA,
//!     player: playerWallet.publicKey,
//!     playerTokenAccount: playerUsdcAccount,
//!     roomVault: roomVaultPDA,
//!     // ... other accounts
//!   })
//!   .rpc();
//! ```
//!
//! ### Fetching Player History
//! ```typescript
//! // Get all rooms a player has joined
//! const playerEntries = await program.account.playerEntry.all([
//!   {
//!     memcmp: {
//!       offset: 8, // After discriminator
//!       bytes: playerWallet.toBase58()
//!     }
//!   }
//! ]);
//!
//! console.log(`Player has joined ${playerEntries.length} rooms`);
//! ```
//!
//! ## Duplicate Prevention
//!
//! The PDA derivation automatically prevents duplicate entries:
//!
//! ```rust
//! #[account(
//!     init,
//!     payer = player,
//!     space = PlayerEntry::LEN,
//!     seeds = [b"player", room.key().as_ref(), player.key().as_ref()],
//!     bump
//! )]
//! pub player_entry: Account<'info, PlayerEntry>,
//! ```
//!
//! If a player attempts to join the same room twice:
//! 1. PDA derivation produces the same address
//! 2. `init` constraint sees account already exists
//! 3. Transaction fails with "Account already initialized"
//!
//! This provides trustless duplicate prevention without additional logic.
//!
//! ## Security Considerations
//!
//! - **Immutable Receipts**: Cannot be tampered with after creation
//! - **PDA Security**: Only program can create these accounts
//! - **Verified Payments**: SPL token transfer occurs atomically with PDA creation
//! - **One Entry Per Player**: Enforced by PDA derivation scheme
//!
//! ## Data Structure Layout
//!
//! Account size: 105 bytes
//! - Discriminator: 8 bytes
//! - Player pubkey: 32 bytes
//! - Room pubkey: 32 bytes
//! - Entry paid: 8 bytes
//! - Extras paid: 8 bytes
//! - Total paid: 8 bytes
//! - Join slot: 8 bytes
//! - Bump: 1 byte
//!
//! ## Use Cases
//!
//! 1. **Player Verification**: Frontend checks if wallet already joined room
//! 2. **Donation Receipts**: Players can prove their charitable contribution
//! 3. **Analytics**: Platform tracks total players, average extras, etc.
//! 4. **Compliance**: Transparent record for regulatory requirements
//! 5. **Leaderboards**: Historical participation data for gamification

use anchor_lang::prelude::*;

/// Individual player participation record
///
/// Immutable receipt of a player joining a specific room, tracking exact
/// payment amounts and timing. One per player per room.
#[account]
#[derive(Debug)]
pub struct PlayerEntry {
    /// Player's public key
    pub player: Pubkey,

    /// Room public key
    pub room: Pubkey,

    /// Entry fee paid
    pub entry_paid: u64,

    /// Extra amount paid
    pub extras_paid: u64,

    /// Total amount paid (entry + extras)
    pub total_paid: u64,

    /// Slot when player joined
    pub join_slot: u64,

    /// PDA bump seed
    pub bump: u8,
}

impl PlayerEntry {
    pub const LEN: usize = 8 + // discriminator
        32 + // player
        32 + // room
        8 + // entry_paid
        8 + // extras_paid
        8 + // total_paid
        8 + // join_slot
        1; // bump
}
