import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Bingo } from "../target/types/bingo";
import { PublicKey, SystemProgram } from "@solana/web3.js";

/**
 * Initialize the Bingo global configuration
 *
 * This script must be run once after deploying the program to set up:
 * - Platform wallet (receives 20% platform fee)
 * - Charity wallet (default charity recipient)
 * - Admin (can pause program in emergencies)
 *
 * Usage:
 *   npx ts-node scripts/initialize.ts <PLATFORM_WALLET> <CHARITY_WALLET>
 *
 * Example:
 *   npx ts-node scripts/initialize.ts \
 *     FunDPlatformWallet...xyz \
 *     FunDCharityWallet...abc
 */

async function main() {
  console.log("========================================");
  console.log("Bingo Global Config Initialization");
  console.log("========================================\n");

  // Parse command line arguments
  const args = process.argv.slice(2);

  if (args.length !== 2) {
    console.error("❌ Error: Invalid arguments\n");
    console.log("Usage:");
    console.log("  npx ts-node scripts/initialize.ts <PLATFORM_WALLET> <CHARITY_WALLET>\n");
    console.log("Example:");
    console.log("  npx ts-node scripts/initialize.ts \\");
    console.log("    FunDPlatformWallet...xyz \\");
    console.log("    FunDCharityWallet...abc\n");
    process.exit(1);
  }

  let platformWallet: PublicKey;
  let charityWallet: PublicKey;

  try {
    platformWallet = new PublicKey(args[0]);
    charityWallet = new PublicKey(args[1]);
  } catch (err) {
    console.error("❌ Error: Invalid public key format");
    console.error(err);
    process.exit(1);
  }

  // Configure the client
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Bingo as Program<Bingo>;

  console.log("Configuration:");
  console.log("  Program ID:", program.programId.toString());
  console.log("  Admin:", provider.wallet.publicKey.toString());
  console.log("  Platform wallet:", platformWallet.toString());
  console.log("  Charity wallet:", charityWallet.toString());
  console.log("  Cluster:", provider.connection.rpcEndpoint);
  console.log();

  // Derive global config PDA
  const [globalConfigPda, bump] = PublicKey.findProgramAddressSync(
    [Buffer.from("global-config")],
    program.programId
  );

  console.log("Global Config PDA:", globalConfigPda.toString());
  console.log("Bump:", bump);
  console.log();

  // Check if already initialized
  try {
    const existingConfig = await program.account.globalConfig.fetch(globalConfigPda);
    console.log("⚠️  Warning: Global config already initialized!");
    console.log("\nExisting configuration:");
    console.log("  Admin:", existingConfig.admin.toString());
    console.log("  Platform wallet:", existingConfig.platformWallet.toString());
    console.log("  Charity wallet:", existingConfig.charityWallet.toString());
    console.log("  Platform fee:", existingConfig.platformFeeBps, "bps (", existingConfig.platformFeeBps / 100, "%)");
    console.log("  Max host fee:", existingConfig.maxHostFeeBps, "bps (", existingConfig.maxHostFeeBps / 100, "%)");
    console.log("  Max prize pool:", existingConfig.maxPrizePoolBps, "bps (", existingConfig.maxPrizePoolBps / 100, "%)");
    console.log("  Min charity:", existingConfig.minCharityBps, "bps (", existingConfig.minCharityBps / 100, "%)");
    console.log("  Emergency pause:", existingConfig.emergencyPause);
    console.log("\nTo re-initialize, you must deploy a new program.");
    return;
  } catch (err) {
    // Not initialized yet - this is expected
    console.log("✓ Config not yet initialized (expected)");
  }

  console.log();
  console.log("Initializing global configuration...");
  console.log();

  try {
    const tx = await program.methods
      .initialize(platformWallet, charityWallet)
      .rpc();

    console.log("✓ Initialization successful!");
    console.log();
    console.log("Transaction signature:", tx);
    console.log();

    // Fetch and display the initialized config
    const config = await program.account.globalConfig.fetch(globalConfigPda);

    console.log("Initialized configuration:");
    console.log("  Admin:", config.admin.toString());
    console.log("  Platform wallet:", config.platformWallet.toString());
    console.log("  Charity wallet:", config.charityWallet.toString());
    console.log("  Platform fee:", config.platformFeeBps, "bps (", config.platformFeeBps / 100, "%)");
    console.log("  Max host fee:", config.maxHostFeeBps, "bps (", config.maxHostFeeBps / 100, "%)");
    console.log("  Max prize pool:", config.maxPrizePoolBps, "bps (", config.maxPrizePoolBps / 100, "%)");
    console.log("  Min charity:", config.minCharityBps, "bps (", config.minCharityBps / 100, "%)");
    console.log("  Emergency pause:", config.emergencyPause);
    console.log();

    // View on Explorer
    const cluster = provider.connection.rpcEndpoint.includes("devnet")
      ? "devnet"
      : provider.connection.rpcEndpoint.includes("testnet")
      ? "testnet"
      : "mainnet";

    if (cluster === "mainnet") {
      console.log("View transaction on Solana Explorer:");
      console.log(`https://explorer.solana.com/tx/${tx}`);
      console.log();
      console.log("View global config account:");
      console.log(`https://explorer.solana.com/address/${globalConfigPda}`);
    } else {
      console.log("View transaction on Solana Explorer:");
      console.log(`https://explorer.solana.com/tx/${tx}?cluster=${cluster}`);
      console.log();
      console.log("View global config account:");
      console.log(`https://explorer.solana.com/address/${globalConfigPda}?cluster=${cluster}`);
    }

    console.log();
    console.log("========================================");
    console.log("Next steps:");
    console.log("1. Run tests: anchor test");
    console.log("2. Test creating a room from the frontend");
    console.log("3. Monitor transactions on Solana Explorer");
    console.log("========================================");
  } catch (err: any) {
    console.error("❌ Initialization failed!");
    console.error();
    console.error("Error:", err.message);

    if (err.logs) {
      console.error();
      console.error("Program logs:");
      err.logs.forEach((log: string) => console.error("  ", log));
    }

    console.error();
    console.error("Troubleshooting:");
    console.error("1. Make sure you have enough SOL");
    console.error("2. Verify you're connected to the right cluster");
    console.error("3. Check that the program is deployed");
    console.error("4. Ensure you haven't already initialized");

    process.exit(1);
  }
}

main()
  .then(() => process.exit(0))
  .catch((err) => {
    console.error(err);
    process.exit(1);
  });
