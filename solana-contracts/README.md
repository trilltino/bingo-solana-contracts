# Fundraisely Solana Smart Contract

A trustless, on-chain fundraising platform built on Solana using the Anchor framework. Fundraisely enables transparent, verifiable charitable fundraising through competitive game rooms with automatic fee distribution.

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Economic Model](#economic-model)
- [Program Instructions](#program-instructions)
- [Development Setup](#development-setup)
- [Testing](#testing)
- [Deployment](#deployment)
- [Initialization](#initialization)
- [Security Considerations](#security-considerations)
- [Useful Commands](#useful-commands)

## Overview

Fundraisely is a decentralized fundraising platform that executes all fund distribution logic on-chain, ensuring trustless operation with zero possibility of fund misappropriation. Entry fees are automatically split between platform operations, hosts, prize pools, and charitable causes according to immutable smart contract rules.

### Key Features

- **Trustless Fund Distribution**: All fee splits execute automatically on-chain
- **Transparent Accounting**: Complete audit trail of all transactions
- **Flexible Room Configuration**: Hosts can configure prize pools and fees within platform limits
- **Multiple Prize Modes**: Pool-based (percentage splits) and asset-based (pre-deposited prizes)
- **Token Registry**: Admin-controlled allowlist of approved SPL tokens
- **Emergency Controls**: Admin can pause operations if critical issues are discovered
- **Room Expiration**: Prevents abandoned rooms from locking funds indefinitely

## Architecture

### Program Structure

The program is organized into modular components:

- **state/**: Account data structures (GlobalConfig, Room, PlayerEntry, TokenRegistry)
- **instructions/**: Instruction handlers organized by feature (admin, room, player, game, asset)
- **errors/**: Custom error definitions
- **events/**: Event definitions for off-chain indexing
- **constants/**: Centralized constants and validation limits

### Program Derived Addresses (PDAs)

All state accounts use PDAs for deterministic addressing and security:

- **GlobalConfig**: `["global-config"]`
- **Room**: `["room", host_pubkey, room_id]`
- **PlayerEntry**: `["player", room_pubkey, player_pubkey]`
- **RoomVault**: `["room-vault", room_pubkey]`
- **TokenRegistry**: `["token-registry-v2"]`

### Account Lifecycle

1. **Initialization**: Admin creates GlobalConfig and TokenRegistry
2. **Room Creation**: Host creates Room and RoomVault PDAs
3. **Player Entry**: Players join rooms, creating PlayerEntry PDAs and depositing tokens
4. **Game End**: Host declares winners and distributes funds automatically
5. **Cleanup**: Host or admin can close rooms and reclaim rent

## Economic Model

### Fee Distribution (Entry Fees)

Entry fees are automatically split via on-chain execution:

- **Platform Fee**: 20% (fixed) - Covers infrastructure and development
- **Host Fee**: 0-5% (configurable) - Incentivizes room creation
- **Prize Pool**: 0-35% (configurable) - Rewards to winners
- **Charity**: 40%+ minimum (calculated remainder) - Primary beneficiary

### Extras Allocation

All "extras" payments (tips beyond entry fee) go 100% to charity, maximizing fundraising impact while maintaining transparent accounting.

### Economic Constraints

The platform enforces these constraints at room creation:

```
Platform Fee: 20% (fixed)
Host Fee: 0-5% (host chooses, max 5%)
Prize Pool: 0-35% (host chooses, max 35%)
Charity: 40%+ (calculated remainder, minimum 40%)

Constraint: host_fee + prize_pool <= 40%
```

### Example Distribution

For a room with 100 USDC entry fee, 5% host fee, 35% prize pool:

- Platform: 20 USDC (20%)
- Host: 5 USDC (5%)
- Prize Pool: 35 USDC (35%)
- Charity: 40 USDC (40%)

If a player pays 10 USDC extra:
- Extra 10 USDC goes 100% to charity
- Total charity: 50 USDC (45.5% of total 110 USDC)

## Program Instructions

### Admin Instructions

#### `initialize`
One-time setup of global configuration. Must be called before any other operations.

**Parameters:**
- `platform_wallet`: Wallet that receives platform fees
- `charity_wallet`: Default charity wallet address

**Sets:**
- Platform fee: 20% (2000 bps)
- Max host fee: 5% (500 bps)
- Max prize pool: 35% (3500 bps)
- Min charity: 40% (4000 bps)

#### `initialize_token_registry`
Creates the token registry PDA for managing approved SPL tokens.

#### `add_approved_token`
Adds a token mint to the approved list. Token must not have freeze authority.

#### `remove_approved_token`
Removes a token from the approved list.

#### `update_global_config`
Updates global configuration parameters (admin only).

#### `set_emergency_pause`
Toggles emergency pause state to halt all operations (admin only).

#### `recover_room`
Recovers funds from abandoned rooms (admin only).

### Room Instructions

#### `init_pool_room`
Creates a new pool-based fundraising room with prize pool distribution.

**Parameters:**
- `room_id`: Unique room identifier (max 32 chars)
- `charity_wallet`: Charity wallet for this room
- `entry_fee`: Entry fee amount in token base units
- `max_players`: Maximum number of players
- `host_fee_bps`: Host fee in basis points (0-500)
- `prize_pool_bps`: Prize pool in basis points (0-3500)
- `first_place_pct`: First place prize percentage (1-100)
- `second_place_pct`: Second place prize percentage (optional)
- `third_place_pct`: Third place prize percentage (optional)
- `charity_memo`: Memo for charity transfers (max 28 chars)
- `expiration_slots`: Room expiration slot (0 = no expiration)

#### `init_asset_room`
Creates an asset-based room with pre-deposited prizes.

**Parameters:**
- Similar to `init_pool_room`, plus:
- `prize_1_mint`, `prize_1_amount`: First prize token and amount
- `prize_2_mint`, `prize_2_amount`: Second prize (optional)
- `prize_3_mint`, `prize_3_amount`: Third prize (optional)

#### `close_joining`
Closes a room to new players (host only).

#### `cleanup_room`
Closes room and reclaims rent (host or admin).

### Player Instructions

#### `join_room`
Joins a room by paying entry fee and optional extras.

**Parameters:**
- `room_id`: Room identifier
- `extras_amount`: Optional extra donation amount (100% to charity)

### Game Instructions

#### `declare_winners`
Declares winners for a room (must be called before `end_room`).

**Parameters:**
- `room_id`: Room identifier
- `winners`: List of winner pubkeys (1-10 winners)

#### `end_room`
Ends a room and distributes funds to platform, host, charity, and winners.

**Parameters:**
- `room_id`: Room identifier
- `winners`: List of winner pubkeys (if not previously declared)

### Asset Instructions

#### `add_prize_asset`
Deposits a prize asset into an asset-based room.

**Parameters:**
- `room_id`: Room identifier
- `prize_index`: Prize index (0 = 1st, 1 = 2nd, 2 = 3rd)

## Development Setup

### Prerequisites

- Rust (latest stable version)
- Solana CLI (v1.18 or later)
- Anchor CLI (v0.32.1)
- Node.js (v18 or later)
- Yarn or npm

### Installation

1. **Install Solana CLI:**
```bash
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
```

2. **Install Anchor:**
```bash
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install latest
avm use latest
```

3. **Clone the repository:**
```bash
git clone <repository-url>
cd solana-contracts
```

4. **Install dependencies:**
```bash
yarn install
```

5. **Build the program:**
```bash
anchor build
```

### Configuration

The project uses Anchor.toml for configuration:

```toml
[toolchain]
anchor_version = "0.32.1"

[provider]
cluster = "devnet"  # Change to "mainnet" for production deployment
wallet = "~/.config/solana/id.json"

[programs.devnet]
bingo = "8W83G9mSeoQ6Ljcz5QJHYPjH2vQgw94YeVCnpY6KFt7i"

[programs.mainnet]
bingo = "8W83G9mSeoQ6Ljcz5QJHYPjH2vQgw94YeVCnpY6KFt7i"
```

**Note for Mainnet Deployment:**
- Update `[provider].cluster = "mainnet"` in Anchor.toml before deploying
- Ensure Solana CLI is configured for mainnet: `solana config set --url https://api.mainnet-beta.solana.com`
- Verify wallet is funded with sufficient SOL (minimum 5-10 SOL recommended)

### Development Workflow

1. **Start local validator (optional):**
```bash
solana-test-validator
```

2. **Build and test:**
```bash
anchor build
anchor test
```

3. **Deploy to devnet:**
```bash
anchor deploy --provider.cluster devnet
```

## Testing

### Run All Tests

```bash
anchor test
```

### Run Specific Test File

```bash
anchor test tests/security.test.ts
```

### Test Coverage

The test suite includes:

- **Basic functionality**: Room creation, player joining, fee distribution
- **Edge cases**: Expired rooms, full rooms, invalid inputs
- **Security**: Access control, duplicate prevention, arithmetic safety
- **Prize distribution**: Multiple winner scenarios, percentage validation

### Test Files

- `tests/fundraisely.ts`: Basic functionality tests
- `tests/security.test.ts`: Security and access control tests
- `tests/edge-cases.test.ts`: Edge case scenarios
- `tests/prize-distribution-fix.test.ts`: Prize distribution validation

## Deployment

### Mainnet Deployment Checklist

#### Pre-Deployment

- [ ] Code review completed
- [ ] All tests passing
- [ ] Security audit completed (if applicable)
- [ ] Economic model validated
- [ ] Access controls verified
- [ ] Emergency pause functionality tested
- [ ] Wallet funded (minimum 5-10 SOL)

#### Deployment Steps

1. **Update configuration for mainnet:**
```bash
solana config set --url https://api.mainnet-beta.solana.com
```

2. **Update Anchor.toml:**
```toml
[provider]
cluster = "mainnet"
```

3. **Build for production:**
```bash
anchor clean
anchor build
```

4. **Deploy to mainnet:**
```bash
anchor deploy --provider.cluster mainnet
```

5. **Verify deployment:**
```bash
solana program show <PROGRAM_ID>
```

#### Post-Deployment

- [ ] Program deployed successfully
- [ ] GlobalConfig initialized
- [ ] TokenRegistry initialized
- [ ] Approved tokens added
- [ ] First test transactions successful
- [ ] Monitoring set up

### Deployment Scripts

The project includes deployment scripts in the `scripts/` directory:

- `scripts/initialize.ts`: Initialize GlobalConfig
- `scripts/setup-tokens.ts`: Initialize TokenRegistry and add tokens
- `scripts/deploy.sh`: Complete deployment workflow

## Initialization

### Step 1: Initialize Global Configuration

After deploying the program, initialize the global configuration:

```bash
npx ts-node scripts/initialize.ts <PLATFORM_WALLET> <CHARITY_WALLET>
```

Example:
```bash
npx ts-node scripts/initialize.ts \
  FunDPlatformWallet111111111111111111111111111 \
  FunDCharityWallet1111111111111111111111111111
```

### Step 2: Initialize Token Registry

Initialize the token registry:

```typescript
// Using Anchor CLI or custom script
anchor run initialize-token-registry
```

Or use the setup script:
```bash
npx ts-node scripts/setup-tokens.ts
```

### Step 3: Add Approved Tokens

Add tokens to the approved list. For mainnet, common tokens include:

- **USDC**: `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`
- **USDT**: `Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB`
- **SOL (Wrapped)**: `So11111111111111111111111111111111111111112`

Example script:
```typescript
await program.methods
  .addApprovedToken(new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"))
  .accounts({
    tokenRegistry: tokenRegistryPDA,
    tokenMintAccount: usdcMint,
    admin: adminWallet.publicKey,
  })
  .rpc();
```

### Verification

After initialization, verify the setup:

1. **Check GlobalConfig:**
```typescript
const [globalConfigPDA] = PublicKey.findProgramAddressSync(
  [Buffer.from("global-config")],
  program.programId
);
const config = await program.account.globalConfig.fetch(globalConfigPDA);
console.log("Platform fee:", config.platformFeeBps / 100, "%");
```

2. **Check TokenRegistry:**
```typescript
const [tokenRegistryPDA] = PublicKey.findProgramAddressSync(
  [Buffer.from("token-registry-v2")],
  program.programId
);
const registry = await program.account.tokenRegistry.fetch(tokenRegistryPDA);
console.log("Approved tokens:", registry.approvedTokens.length);
```

## Security Considerations

### Access Control

- **Admin Authority**: Only admin can modify GlobalConfig and TokenRegistry
- **Host Authority**: Only host can end their rooms (unless expired)
- **PDA Security**: All accounts use PDAs, preventing signature forgery

### Economic Security

- **Fee Validation**: All fee percentages validated at room creation
- **Arithmetic Safety**: All calculations use checked math to prevent overflow/underflow
- **Minimum Charity**: Enforced 40% minimum charity allocation

### Operational Security

- **Emergency Pause**: Admin can halt operations if critical issues discovered
- **Token Validation**: Only approved tokens without freeze authority allowed
- **Room Expiration**: Prevents abandoned rooms from locking funds
- **Host Restrictions**: Hosts cannot be winners, preventing self-dealing

### Best Practices

- **Key Management**: Secure admin keys using hardware wallets or multisig
- **Monitoring**: Monitor program usage and transactions for anomalies
- **Testing**: Thoroughly test all functionality on devnet before mainnet
- **Documentation**: Maintain clear documentation of all operations

## Useful Commands

### Development

```bash
# Build the program
anchor build

# Run tests
anchor test

# Clean build artifacts
anchor clean

# Generate IDL
anchor idl parse -f programs/bingo/src/lib.rs -o target/idl/bingo.json
```

### Deployment

```bash
# Deploy to devnet
anchor deploy --provider.cluster devnet

# Deploy to mainnet
anchor deploy --provider.cluster mainnet

# Verify program
solana program show <PROGRAM_ID>
```

### Account Management

```bash
# Check wallet balance
solana balance

# Check program account
solana program show <PROGRAM_ID>

# View account data
solana account <ACCOUNT_ADDRESS>
```

### Scripts

```bash
# Initialize global config
npx ts-node scripts/initialize.ts <PLATFORM_WALLET> <CHARITY_WALLET>

# Setup token registry
npx ts-node scripts/setup-tokens.ts

# Update charity wallet
npx ts-node scripts/update-charity-wallet.ts

# Update global config
npx ts-node scripts/update-config.ts
```

## Program ID

**Current Program ID:** `8W83G9mSeoQ6Ljcz5QJHYPjH2vQgw94YeVCnpY6KFt7i`

**Explorer Links:**
- Devnet: [View on Explorer](https://explorer.solana.com/address/8W83G9mSeoQ6Ljcz5QJHYPjH2vQgw94YeVCnpY6KFt7i?cluster=devnet)
- Mainnet: [View on Explorer](https://explorer.solana.com/address/8W83G9mSeoQ6Ljcz5QJHYPjH2vQgw94YeVCnpY6KFt7i)

## License

[Add license information]

## Support

For issues, questions, or contributions, please [open an issue]([repository-url]/issues) or contact the development team.

