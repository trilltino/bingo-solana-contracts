import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Fundraisely } from "../target/types/fundraisely";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  createAccount,
  mintTo,
  getAccount,
  getAssociatedTokenAddress,
} from "@solana/spl-token";
import { assert, expect } from "chai";

describe("Prize Distribution Fix Test", () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Fundraisely as Program<Fundraisely>;
  const admin = provider.wallet as anchor.Wallet;

  // Test accounts
  let tokenMint: PublicKey;
  let platformWallet: Keypair;
  let charityWallet: Keypair;
  let hostWallet: Keypair;
  let player1Wallet: Keypair;
  let player2Wallet: Keypair;

  // Token accounts
  let hostTokenAccount: PublicKey;
  let player1TokenAccount: PublicKey;
  let player2TokenAccount: PublicKey;
  let platformTokenAccount: PublicKey;
  let charityTokenAccount: PublicKey;

  // PDAs
  let globalConfigPda: PublicKey;

  before(async () => {
    // Create wallets
    platformWallet = Keypair.generate();
    charityWallet = Keypair.generate();
    hostWallet = Keypair.generate();
    player1Wallet = Keypair.generate();
    player2Wallet = Keypair.generate();

    // Airdrop SOL to wallets
    const airdropAmount = 2 * LAMPORTS_PER_SOL;
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(hostWallet.publicKey, airdropAmount)
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(player1Wallet.publicKey, airdropAmount)
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(player2Wallet.publicKey, airdropAmount)
    );

    // Create token mint (USDC with 6 decimals)
    tokenMint = await createMint(
      provider.connection,
      admin.payer,
      admin.publicKey,
      null,
      6 // 6 decimals
    );

    // Create token accounts
    hostTokenAccount = await createAccount(
      provider.connection,
      admin.payer,
      tokenMint,
      hostWallet.publicKey
    );

    player1TokenAccount = await createAccount(
      provider.connection,
      admin.payer,
      tokenMint,
      player1Wallet.publicKey
    );

    player2TokenAccount = await createAccount(
      provider.connection,
      admin.payer,
      tokenMint,
      player2Wallet.publicKey
    );

    platformTokenAccount = await createAccount(
      provider.connection,
      admin.payer,
      tokenMint,
      platformWallet.publicKey
    );

    charityTokenAccount = await createAccount(
      provider.connection,
      admin.payer,
      tokenMint,
      charityWallet.publicKey
    );

    // Mint tokens to players (100 USDC each)
    const mintAmount = 100 * 1_000_000; // 100 USDC with 6 decimals
    await mintTo(
      provider.connection,
      admin.payer,
      tokenMint,
      player1TokenAccount,
      admin.publicKey,
      mintAmount
    );

    await mintTo(
      provider.connection,
      admin.payer,
      tokenMint,
      player2TokenAccount,
      admin.publicKey,
      mintAmount
    );

    // Derive global config PDA
    [globalConfigPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("global-config")],
      program.programId
    );

    // Initialize global config if not already initialized
    try {
      await program.methods
        .initialize(platformWallet.publicKey, charityWallet.publicKey)
        .accounts({
          globalConfig: globalConfigPda,
          admin: admin.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
    } catch (err) {
      // Already initialized, that's fine
      console.log("Global config already initialized");
    }
  });

  it("Should correctly distribute prizes: 6 USDC total, 5% host, 35% prize pool, 80%/20% split", async () => {
    const roomId = "prize-fix-test-room";
    const [roomPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("room"), hostWallet.publicKey.toBuffer(), Buffer.from(roomId)],
      program.programId
    );

    const [roomVaultPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("room-vault"), roomPda.toBuffer()],
      program.programId
    );

    // Configuration: 5% host, 35% prize pool, 40% charity (remaining)
    const entryFee = new anchor.BN(3 * 1_000_000); // 3 USDC per player
    const hostFeeBps = 500; // 5%
    const prizePoolBps = 3500; // 35%
    const firstPlacePct = 80; // 80% of prize pool
    const secondPlacePct = 20; // 20% of prize pool

    // Create room
    await program.methods
      .initPoolRoom(
        roomId,
        entryFee,
        hostFeeBps,
        prizePoolBps,
        firstPlacePct,
        secondPlacePct,
        null,
        "Prize fix test"
      )
      .accounts({
        room: roomPda,
        roomVault: roomVaultPda,
        globalConfig: globalConfigPda,
        host: hostWallet.publicKey,
        feeTokenMint: tokenMint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([hostWallet])
      .rpc();

    // Player 1 joins (3 USDC)
    const [player1Entry] = PublicKey.findProgramAddressSync(
      [Buffer.from("player"), roomPda.toBuffer(), player1Wallet.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .joinRoom(roomId, new anchor.BN(0))
      .accounts({
        room: roomPda,
        roomVault: roomVaultPda,
        playerEntry: player1Entry,
        player: player1Wallet.publicKey,
        playerTokenAccount: player1TokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([player1Wallet])
      .rpc();

    // Player 2 joins (3 USDC)
    const [player2Entry] = PublicKey.findProgramAddressSync(
      [Buffer.from("player"), roomPda.toBuffer(), player2Wallet.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .joinRoom(roomId, new anchor.BN(0))
      .accounts({
        room: roomPda,
        roomVault: roomVaultPda,
        playerEntry: player2Entry,
        player: player2Wallet.publicKey,
        playerTokenAccount: player2TokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([player2Wallet])
      .rpc();

    // Get initial balances
    const initialPlatformBalance = (await getAccount(provider.connection, platformTokenAccount)).amount;
    const initialCharityBalance = (await getAccount(provider.connection, charityTokenAccount)).amount;
    const initialHostBalance = (await getAccount(provider.connection, hostTokenAccount)).amount;
    const initialPlayer1Balance = (await getAccount(provider.connection, player1TokenAccount)).amount;
    const initialPlayer2Balance = (await getAccount(provider.connection, player2TokenAccount)).amount;

    // Expected calculations:
    // Total entry fees: 6 USDC (2 players × 3 USDC)
    // Platform (20%): 1.2 USDC
    // Host (5%): 0.3 USDC
    // Prize pool (35%): 2.1 USDC
    // Charity (40%): 2.4 USDC
    // 
    // Prize distribution from prize pool (2.1 USDC):
    // First place (80%): 2.1 × 0.80 = 1.68 USDC
    // Second place (20%): 2.1 × 0.20 = 0.42 USDC

    const totalEntryFees = 6 * 1_000_000; // 6 USDC
    const expectedPlatform = Math.floor((totalEntryFees * 2000) / 10000); // 1.2 USDC
    const expectedHost = Math.floor((totalEntryFees * 500) / 10000); // 0.3 USDC
    const expectedPrizePool = Math.floor((totalEntryFees * 3500) / 10000); // 2.1 USDC
    const expectedCharity = totalEntryFees - expectedPlatform - expectedHost - expectedPrizePool; // 2.4 USDC

    const expectedFirstPrize = Math.floor((expectedPrizePool * 80) / 100); // 1.68 USDC
    const expectedSecondPrize = Math.floor((expectedPrizePool * 20) / 100); // 0.42 USDC

    console.log("\n=== EXPECTED DISTRIBUTION ===");
    console.log("Total entry fees:", totalEntryFees / 1_000_000, "USDC");
    console.log("Platform (20%):", expectedPlatform / 1_000_000, "USDC");
    console.log("Host (5%):", expectedHost / 1_000_000, "USDC");
    console.log("Prize pool (35%):", expectedPrizePool / 1_000_000, "USDC");
    console.log("Charity (40%):", expectedCharity / 1_000_000, "USDC");
    console.log("\nPrize distribution:");
    console.log("1st place (80% of prize pool):", expectedFirstPrize / 1_000_000, "USDC");
    console.log("2nd place (20% of prize pool):", expectedSecondPrize / 1_000_000, "USDC");

    // Get winner token accounts
    const winner1TokenAccount = await getAssociatedTokenAddress(
      tokenMint,
      player1Wallet.publicKey
    );
    const winner2TokenAccount = await getAssociatedTokenAddress(
      tokenMint,
      player2Wallet.publicKey
    );

    // End room and distribute
    const winners = [player1Wallet.publicKey, player2Wallet.publicKey];

    await program.methods
      .endRoom(roomId, winners)
      .accounts({
        room: roomPda,
        roomVault: roomVaultPda,
        globalConfig: globalConfigPda,
        platformTokenAccount: platformTokenAccount,
        charityTokenAccount: charityTokenAccount,
        hostTokenAccount: hostTokenAccount,
        host: hostWallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .remainingAccounts([
        {
          pubkey: winner1TokenAccount,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: winner2TokenAccount,
          isSigner: false,
          isWritable: true,
        },
      ])
      .signers([hostWallet])
      .rpc();

    // Verify balances
    const finalPlatformBalance = (await getAccount(provider.connection, platformTokenAccount)).amount;
    const finalCharityBalance = (await getAccount(provider.connection, charityTokenAccount)).amount;
    const finalHostBalance = (await getAccount(provider.connection, hostTokenAccount)).amount;
    const finalPlayer1Balance = (await getAccount(provider.connection, player1TokenAccount)).amount;
    const finalPlayer2Balance = (await getAccount(provider.connection, player2TokenAccount)).amount;

    const actualPlatform = Number(finalPlatformBalance - initialPlatformBalance);
    const actualCharity = Number(finalCharityBalance - initialCharityBalance);
    const actualHost = Number(finalHostBalance - initialHostBalance);
    const actualFirstPrize = Number(finalPlayer1Balance - initialPlayer1Balance);
    const actualSecondPrize = Number(finalPlayer2Balance - initialPlayer2Balance);

    console.log("\n=== ACTUAL DISTRIBUTION ===");
    console.log("Platform received:", actualPlatform / 1_000_000, "USDC");
    console.log("Host received:", actualHost / 1_000_000, "USDC");
    console.log("Charity received:", actualCharity / 1_000_000, "USDC");
    console.log("1st place received:", actualFirstPrize / 1_000_000, "USDC");
    console.log("2nd place received:", actualSecondPrize / 1_000_000, "USDC");

    // Assertions
    assert.approximately(
      actualPlatform,
      expectedPlatform,
      1000, // Allow 0.001 USDC tolerance
      "Platform fee should be 1.2 USDC"
    );

    assert.approximately(
      actualHost,
      expectedHost,
      1000,
      "Host fee should be 0.3 USDC"
    );

    assert.approximately(
      actualCharity,
      expectedCharity,
      1000,
      "Charity should receive 2.4 USDC"
    );

    // THE CRITICAL TEST: First place should get 1.68 USDC (80% of prize pool), NOT 4.8 USDC (80% of total)
    assert.approximately(
      actualFirstPrize,
      expectedFirstPrize,
      1000,
      `First place should get ${expectedFirstPrize / 1_000_000} USDC (80% of prize pool), NOT ${totalEntryFees * 80 / 100 / 1_000_000} USDC (80% of total)`
    );

    assert.approximately(
      actualSecondPrize,
      expectedSecondPrize,
      1000,
      "Second place should get 0.42 USDC (20% of prize pool)"
    );

    console.log("\n✅ ALL TESTS PASSED! Prize distribution is correct.");
  });
});

