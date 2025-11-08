import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Bingo } from "../target/types/bingo";

async function main() {
  // Configure the client
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.Bingo as Program<Bingo>;

  console.log("Program ID:", program.programId.toBase58());
  console.log("Provider wallet:", program.provider.publicKey.toBase58());

  // Derive GlobalConfig PDA
  const [globalConfigPDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("global-config")],
    program.programId
  );

  console.log("GlobalConfig PDA:", globalConfigPDA.toBase58());

  // Fetch current config
  console.log("\nFetching current GlobalConfig...");
  const configBefore = await program.account.globalConfig.fetch(globalConfigPDA);
  console.log("Current values:");
  console.log("  max_prize_pool_bps:", configBefore.maxPrizePoolBps);
  console.log("  min_charity_bps:", configBefore.minCharityBps);

  // Update the config
  console.log("\nUpdating GlobalConfig...");
  const tx = await program.methods
    .updateGlobalConfig(
      null, // platform_wallet - no change
      null, // charity_wallet - no change
      null, // platform_fee_bps - no change
      null, // max_host_fee_bps - no change
      3500, // max_prize_pool_bps - update to 35%
      4000  // min_charity_bps - update to 40%
    )
    .accounts({
      globalConfig: globalConfigPDA,
      admin: program.provider.publicKey,
    })
    .rpc();

  console.log("Transaction signature:", tx);

  // Fetch updated config
  console.log("\nFetching updated GlobalConfig...");
  const configAfter = await program.account.globalConfig.fetch(globalConfigPDA);
  console.log("Updated values:");
  console.log("  max_prize_pool_bps:", configAfter.maxPrizePoolBps);
  console.log("  min_charity_bps:", configAfter.minCharityBps);

  console.log("\n✅ GlobalConfig updated successfully!");
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
