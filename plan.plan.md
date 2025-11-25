<!-- 25d35a66-25b7-4ce0-8eb8-60149512985a 548a3f34-747d-4612-bc9c-e4437a53cf05 -->
# Fix Three Critical Contract Bugs

## Issues to Fix

1. **CleanupRoom token authority error**: Missing `token::authority = room` constraint causing "owner does not match" when closing vault
2. **Prize distribution calculation**: Fees calculated on `entry_fees_total` instead of `total_pool` (entry_fees + extras), causing 100% of extras to go to charity
3. **InitAssetRoom mut constraint error**: Using `AccountInfo` with `mut` constraint causes ConstraintMut violation; should use `UncheckedAccount` like InitPoolRoom

## Files to Modify

### 1. `bingo/programs/bingo/src/lib.rs`

- **Line ~940-946**: Add `token::authority = room` and `token::mint = room.fee_token_mint` constraints to `CleanupRoom.room_vault`
- **Line ~789-795**: Change `InitAssetRoom.room_vault` from `AccountInfo<'info>` to `UncheckedAccount<'info>`

### 2. `bingo/programs/bingo/src/instructions/game/end_room.rs` 

- **Line ~85-99**: Replace fee calculation block to use `total_pool` (entry_fees_total + extras_total) instead of just `entry_fees_total` for platform_fee, host_fee, and prize_amount calculations

## Implementation Details

### Fix 1: CleanupRoom token authority

```rust
#[account(
    mut,
    seeds = [b"room-vault", room.key().as_ref()],
    bump,
    token::authority = room,           // ADD THIS
    token::mint = room.fee_token_mint   // ADD THIS
)]
pub room_vault: Account<'info, anchor_spl::token::TokenAccount>,
```

### Fix 2: Prize distribution calculation

Replace the calculation block with:

```rust
// Calculate total pool (entry fees + extras)
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
```

### Fix 3: InitAssetRoom account type

Change from:

```rust
pub room_vault: AccountInfo<'info>,
```

To:

```rust
pub room_vault: UncheckedAccount<'info>,
```

## Testing Considerations

- Verify CleanupRoom can close empty vaults without authority errors
- Verify prize distribution splits total_pool correctly (not just entry fees)
- Verify InitAssetRoom can create asset-based rooms without ConstraintMut errors
- All changes maintain backward compatibility with existing accounts