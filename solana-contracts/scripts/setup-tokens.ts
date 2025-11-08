/**
 * Setup Token Registry for Bingo Program
 *
 * This script:
 * 1. Initializes TokenRegistry (if not already done)
 * 2. Adds USDC devnet to allowlist
 * 3. Adds Wrapped SOL to allowlist
 */

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Bingo } from "../target/types/bingo";
import { PublicKey, SystemProgram } from "@solana/web3.js";

// Token mints on devnet
const USDC_DEVNET = new PublicKey("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU");
const WRAPPED_SOL = new PublicKey("So11111111111111111111111111111111111111112");

async function main() {
  console.log("========================================");
  console.log("Token Registry Setup");
  console.log("========================================\n");

  // Configure the client
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Bingo as Program<Bingo>;

  console.log("Configuration:");
  console.log("  Program ID:", program.programId.toString());
  console.log("  Admin:", provider.wallet.publicKey.toString());
  console.log("  Cluster:", provider.connection.rpcEndpoint);
  console.log();

  // Derive PDAs
  const [globalConfigPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("global-config")],
    program.programId
  );

  const [tokenRegistryPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("approved_tokens")],
    program.programId
  );

  console.log("Global Config PDA:", globalConfigPda.toString());
  console.log("Token Registry PDA:", tokenRegistryPda.toString());
  console.log();

  // ========== STEP 1: Initialize TokenRegistry ==========
  console.log("STEP 1: Initialize TokenRegistry");
  console.log("-".repeat(40));

  try {
    const existing = await program.account.tokenRegistry.fetch(tokenRegistryPda);
    console.log("✓ TokenRegistry already initialized");
    console.log("  Token count:", existing.approvedTokens.length);
  } catch (err) {
    console.log("  Initializing...");
    try {
      const tx = await program.methods
        .initializeTokenRegistry()
        .rpc();

      console.log("✓ TokenRegistry initialized!");
      console.log("  Transaction:", tx);
    } catch (error: any) {
      console.error("❌ Failed to initialize TokenRegistry:", error.message);
      process.exit(1);
    }
  }

  console.log();

  // ========== STEP 2: Add USDC ==========
  console.log("STEP 2: Add USDC to allowlist");
  console.log("-".repeat(40));
  console.log("  Token:", USDC_DEVNET.toString());

  try {
    const tx = await program.methods
      .addApprovedToken(USDC_DEVNET)
      .rpc();

    console.log("✓ USDC added!");
    console.log("  Transaction:", tx);
  } catch (error: any) {
    if (error.message?.includes("TokenAlreadyApproved") || error.message?.includes("0x1772")) {
      console.log("✓ USDC already in allowlist");
    } else {
      console.error("❌ Failed to add USDC:", error.message);
    }
  }

  console.log();

  // ========== STEP 3: Add Wrapped SOL ==========
  console.log("STEP 3: Add Wrapped SOL to allowlist");
  console.log("-".repeat(40));
  console.log("  Token:", WRAPPED_SOL.toString());

  try {
    const tx = await program.methods
      .addApprovedToken(WRAPPED_SOL)
      .rpc();

    console.log("✓ Wrapped SOL added!");
    console.log("  Transaction:", tx);
  } catch (error: any) {
    if (error.message?.includes("TokenAlreadyApproved") || error.message?.includes("0x1772")) {
      console.log("✓ Wrapped SOL already in allowlist");
    } else {
      console.error("❌ Failed to add Wrapped SOL:", error.message);
    }
  }

  console.log();

  // ========== Summary ==========
  console.log("========================================");
  console.log("Setup Complete!");
  console.log("========================================");

  // Fetch and display final state
  try {
    const registry = await program.account.tokenRegistry.fetch(tokenRegistryPda);
    console.log("\nApproved tokens:", registry.approvedTokens.length);
    registry.approvedTokens.forEach((token: any, idx: number) => {
      console.log(`  ${idx + 1}. ${token.toString()}`);
    });
  } catch (err) {
    console.log("\n(Could not fetch token registry)");
  }

  console.log("\n🚀 Ready to create rooms!");
  console.log("   Visit: http://localhost:5173");
  console.log("   Connect Phantom wallet (devnet)");
  console.log("   Select Solana chain and create a room\n");
}

main()
  .then(() => process.exit(0))
  .catch((err) => {
    console.error(err);
    process.exit(1);
  });
