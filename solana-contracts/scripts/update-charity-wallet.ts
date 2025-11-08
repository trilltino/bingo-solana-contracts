import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { Bingo } from "../target/types/bingo";

/**
 * Script to update the charity wallet in GlobalConfig
 *
 * Usage: anchor run update-charity
 */

async function main() {
  // Configure the client to use devnet
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Bingo as Program<Bingo>;

  console.log("═══════════════════════════════════════════════════════");
  console.log("UPDATE CHARITY WALLET");
  console.log("═══════════════════════════════════════════════════════");
  console.log("Program ID:", program.programId.toBase58());
  console.log("Wallet:", provider.wallet.publicKey.toBase58());
  console.log("");

  // Derive GlobalConfig PDA
  const [globalConfigPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("global-config")],
    program.programId
  );

  console.log("GlobalConfig PDA:", globalConfigPDA.toBase58());
  console.log("");

  // Fetch current config
  try {
    const config = await program.account.globalConfig.fetch(globalConfigPDA);
    console.log("Current Configuration:");
    console.log("  Admin:", config.admin.toBase58());
    console.log("  Platform Wallet:", config.platformWallet.toBase58());
    console.log("  Charity Wallet:", config.charityWallet.toBase58());
    console.log("  Platform Fee:", config.platformFeeBps, "bps (", config.platformFeeBps / 100, "%)");
    console.log("  Max Host Fee:", config.maxHostFeeBps, "bps (", config.maxHostFeeBps / 100, "%)");
    console.log("  Max Prize Pool:", config.maxPrizePoolBps, "bps (", config.maxPrizePoolBps / 100, "%)");
    console.log("  Min Charity:", config.minCharityBps, "bps (", config.minCharityBps / 100, "%)");
    console.log("");

    // New charity wallet
    const newCharityWallet = new PublicKey("Ma6H8rHHk3WPSEymdofzGKnB9j2LAzu1LkCPeJnz2NV");

    console.log("───────────────────────────────────────────────────────");
    console.log("Updating charity wallet to:", newCharityWallet.toBase58());
    console.log("───────────────────────────────────────────────────────");
    console.log("");

    // Call update_global_config
    const tx = await program.methods
      .updateGlobalConfig(
        null,              // platform_wallet - no change
        newCharityWallet,  // charity_wallet - UPDATE THIS
        null,              // platform_fee_bps - no change
        null,              // max_host_fee_bps - no change
        null,              // max_prize_pool_bps - no change
        null               // min_charity_bps - no change
      )
      .accounts({
        globalConfig: globalConfigPDA,
        admin: provider.wallet.publicKey,
      })
      .rpc();

    console.log("✅ Transaction successful!");
    console.log("Signature:", tx);
    console.log("");

    // Fetch updated config
    const updatedConfig = await program.account.globalConfig.fetch(globalConfigPDA);
    console.log("Updated Configuration:");
    console.log("  Admin:", updatedConfig.admin.toBase58());
    console.log("  Platform Wallet:", updatedConfig.platformWallet.toBase58());
    console.log("  Charity Wallet:", updatedConfig.charityWallet.toBase58());
    console.log("");

    if (updatedConfig.charityWallet.toBase58() === newCharityWallet.toBase58()) {
      console.log("✅ Charity wallet successfully updated!");
    } else {
      console.log("❌ Charity wallet update verification failed");
    }

  } catch (error) {
    console.error("❌ Error:", error);
    throw error;
  }

  console.log("═══════════════════════════════════════════════════════");
}

main()
  .then(() => {
    console.log("✅ Script completed successfully");
    process.exit(0);
  })
  .catch((error) => {
    console.error("❌ Script failed:", error);
    process.exit(1);
  });
