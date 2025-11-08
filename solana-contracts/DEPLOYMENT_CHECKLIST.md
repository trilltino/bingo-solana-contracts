# Mainnet Deployment Checklist

Use this checklist to ensure all steps are completed for mainnet deployment.

## Pre-Deployment

### Environment Setup
- [ ] Solana CLI installed and configured
- [ ] Anchor CLI v0.32.1 installed
- [ ] Node.js and dependencies installed (`yarn install`)
- [ ] Mainnet wallet configured and funded (minimum 5-10 SOL)
- [ ] Solana CLI set to mainnet: `solana config set --url https://api.mainnet-beta.solana.com`
- [ ] Wallet address verified: `solana address`
- [ ] Wallet balance checked: `solana balance` (should show sufficient SOL)

### Code Review
- [ ] All tests passing: `anchor test`
- [ ] Code review completed
- [ ] Security audit completed (if applicable)
- [ ] Economic model validated
- [ ] Access controls verified
- [ ] Emergency pause functionality tested
- [ ] No known security vulnerabilities

### Configuration
- [ ] Program ID verified in `programs/bingo/src/lib.rs`
- [ ] Anchor.toml mainnet configuration verified
- [ ] Platform wallet address prepared
- [ ] Charity wallet address prepared
- [ ] Admin wallet address prepared
- [ ] Token mints identified (USDC, USDT, etc.)

## Deployment

### Build and Deploy
- [ ] Previous builds cleaned: `anchor clean`
- [ ] Program built for production: `anchor build`
- [ ] Build artifacts verified in `target/deploy/`
- [ ] Program keypair exists: `target/deploy/bingo-keypair.json`
- [ ] Program ID matches expected: `solana-keygen pubkey target/deploy/bingo-keypair.json`
- [ ] Anchor.toml provider.cluster set to "mainnet"
- [ ] Program deployed: `anchor deploy --provider.cluster mainnet`
- [ ] Deployment transaction signature recorded
- [ ] Program verified on-chain: `solana program show <PROGRAM_ID>`

## Post-Deployment Initialization

### Global Configuration
- [ ] GlobalConfig initialized: `npx ts-node scripts/initialize.ts <PLATFORM_WALLET> <CHARITY_WALLET>`
- [ ] GlobalConfig PDA address recorded
- [ ] Configuration values verified:
  - [ ] Platform fee: 20% (2000 bps)
  - [ ] Max host fee: 5% (500 bps)
  - [ ] Max prize pool: 35% (3500 bps)
  - [ ] Min charity: 40% (4000 bps)
- [ ] Initialization transaction signature recorded

### Token Registry
- [ ] TokenRegistry initialized: `npx ts-node scripts/initialize-token-registry.ts`
- [ ] TokenRegistry PDA address recorded
- [ ] Initialization transaction signature recorded

### Token Approvals
- [ ] USDC added: `npx ts-node scripts/add-approved-token.ts EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`
- [ ] USDT added: `npx ts-node scripts/add-approved-token.ts Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB`
- [ ] Wrapped SOL added: `npx ts-node scripts/add-approved-token.ts So11111111111111111111111111111111111111112`
- [ ] All token approval transactions recorded
- [ ] Token registry verified (all tokens present)

## Verification

### Functional Testing
- [ ] Room creation tested with approved token
- [ ] Player joining flow tested
- [ ] Fee distribution verified
- [ ] Winner declaration tested
- [ ] Room ending and fund distribution tested
- [ ] Events emitted correctly

### Monitoring
- [ ] Transaction monitoring set up
- [ ] Error monitoring configured
- [ ] Explorer links bookmarked
- [ ] Alerts configured (if applicable)

## Documentation

### Addresses Recorded
- [ ] Program ID recorded in MAINNET.md
- [ ] GlobalConfig PDA recorded
- [ ] TokenRegistry PDA recorded
- [ ] Platform wallet address recorded
- [ ] Charity wallet address recorded
- [ ] Admin wallet address recorded
- [ ] All approved token mints recorded

### Transactions Recorded
- [ ] Deployment transaction signature
- [ ] GlobalConfig initialization transaction
- [ ] TokenRegistry initialization transaction
- [ ] All token approval transactions

### Documentation Updated
- [ ] README.md reviewed and updated
- [ ] MAINNET.md populated with addresses
- [ ] DEPLOYMENT_CHECKLIST.md completed
- [ ] All explorer links verified

## Security

### Key Management
- [ ] Admin keys secured (hardware wallet or multisig)
- [ ] Key backups created and stored securely
- [ ] Access to admin keys restricted
- [ ] Key rotation plan documented

### Post-Deployment Security
- [ ] Emergency pause procedure documented
- [ ] Incident response plan prepared
- [ ] Monitoring alerts configured
- [ ] Regular security review scheduled

## Final Verification

- [ ] All checklist items completed
- [ ] All addresses documented
- [ ] All transactions recorded
- [ ] Testing completed successfully
- [ ] Monitoring active
- [ ] Documentation complete
- [ ] Team notified of deployment

## Sign-Off

- **Deployed by**: _________________ Date: ___________
- **Verified by**: _________________ Date: ___________
- **Approved by**: _________________ Date: ___________

## Notes

Use this section to record any issues, observations, or important notes during deployment:

_________________________________________________________________________

_________________________________________________________________________

_________________________________________________________________________

