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
} from "@solana/spl-token";
import { assert, expect } from "chai";

describe("fundraisely", () => {
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
  let player3Wallet: Keypair;

  // Token accounts
  let hostTokenAccount: PublicKey;
  let player1TokenAccount: PublicKey;
  let player2TokenAccount: PublicKey;
  let player3TokenAccount: PublicKey;
  let platformTokenAccount: PublicKey;
  let charityTokenAccount: PublicKey;

  // PDAs
  let globalConfigPda: PublicKey;
  let globalConfigBump: number;

  before(async () => {
    // Create wallets
    platformWallet = Keypair.generate();
    charityWallet = Keypair.generate();
    hostWallet = Keypair.generate();
    player1Wallet = Keypair.generate();
    player2Wallet = Keypair.generate();
    player3Wallet = Keypair.generate();

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
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(player3Wallet.publicKey, airdropAmount)
    );

    // Create token mint
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

    player3TokenAccount = await createAccount(
      provider.connection,
      admin.payer,
      tokenMint,
      player3Wallet.publicKey
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

    // Mint tokens to players (1000 tokens each)
    const mintAmount = 1000 * 1_000_000; // 1000 tokens with 6 decimals
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

    await mintTo(
      provider.connection,
      admin.payer,
      tokenMint,
      player3TokenAccount,
      admin.publicKey,
      mintAmount
    );

    // Derive global config PDA
    [globalConfigPda, globalConfigBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("global-config")],
      program.programId
    );
  });

  describe("Initialize", () => {
    it("Successfully initializes global config", async () => {
      await program.methods
        .initialize(platformWallet.publicKey, charityWallet.publicKey)
        .accounts({
          globalConfig: globalConfigPda,
          admin: admin.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      const config = await program.account.globalConfig.fetch(globalConfigPda);

      assert.equal(config.admin.toString(), admin.publicKey.toString());
      assert.equal(config.platformWallet.toString(), platformWallet.publicKey.toString());
      assert.equal(config.charityWallet.toString(), charityWallet.publicKey.toString());
      assert.equal(config.platformFeeBps, 2000); // 20%
      assert.equal(config.maxHostFeeBps, 500); // 5%
      assert.equal(config.maxPrizePoolBps, 3500); // 35%
      assert.equal(config.minCharityBps, 4000); // 40% minimum
      assert.equal(config.emergencyPause, false);
    });

    it("Fails to initialize twice", async () => {
      try {
        await program.methods
          .initialize(platformWallet.publicKey, charityWallet.publicKey)
          .accounts({
            globalConfig: globalConfigPda,
            admin: admin.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .rpc();
        assert.fail("Should have thrown error");
      } catch (err) {
        // Expected to fail - account already exists
        expect(err.message).to.include("already in use");
      }
    });
  });

  describe("Init Pool Room", () => {
    const roomId = "test-room-1";
    let roomPda: PublicKey;
    let roomVaultPda: PublicKey;

    before(() => {
      [roomPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("room"), hostWallet.publicKey.toBuffer(), Buffer.from(roomId)],
        program.programId
      );

      [roomVaultPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("room-vault"), roomPda.toBuffer()],
        program.programId
      );
    });

    it("Successfully creates a pool room", async () => {
      const entryFee = new anchor.BN(10 * 1_000_000); // 10 tokens
      const hostFeeBps = 300; // 3%
      const prizePoolBps = 2000; // 20%
      const firstPlacePct = 60;
      const secondPlacePct = 30;
      const thirdPlacePct = 10;
      const charityMemo = "Test charity campaign";

      await program.methods
        .initPoolRoom(
          roomId,
          entryFee,
          hostFeeBps,
          prizePoolBps,
          firstPlacePct,
          secondPlacePct,
          thirdPlacePct,
          charityMemo
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

      const room = await program.account.room.fetch(roomPda);

      assert.equal(room.roomId, roomId);
      assert.equal(room.host.toString(), hostWallet.publicKey.toString());
      assert.equal(room.entryFee.toNumber(), entryFee.toNumber());
      assert.equal(room.hostFeeBps, hostFeeBps);
      assert.equal(room.prizePoolBps, prizePoolBps);
      assert.equal(room.playerCount, 0);
      assert.equal(room.totalCollected.toNumber(), 0);
      assert.equal(room.ended, false);
    });

    it("Fails with host fee too high", async () => {
      const roomId2 = "test-room-2";
      const [roomPda2] = PublicKey.findProgramAddressSync(
        [Buffer.from("room"), hostWallet.publicKey.toBuffer(), Buffer.from(roomId2)],
        program.programId
      );
      const [roomVaultPda2] = PublicKey.findProgramAddressSync(
        [Buffer.from("room-vault"), roomPda2.toBuffer()],
        program.programId
      );

      try {
        await program.methods
          .initPoolRoom(
            roomId2,
            new anchor.BN(10 * 1_000_000),
            600, // 6% - exceeds max of 5%
            2000,
            100,
            null,
            null,
            "Test"
          )
          .accounts({
            room: roomPda2,
            roomVault: roomVaultPda2,
            globalConfig: globalConfigPda,
            host: hostWallet.publicKey,
            feeTokenMint: tokenMint,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([hostWallet])
          .rpc();
        assert.fail("Should have thrown error");
      } catch (err) {
        expect(err.toString()).to.include("HostFeeTooHigh");
      }
    });

    it("Fails with total allocation too high", async () => {
      const roomId3 = "test-room-3";
      const [roomPda3] = PublicKey.findProgramAddressSync(
        [Buffer.from("room"), hostWallet.publicKey.toBuffer(), Buffer.from(roomId3)],
        program.programId
      );
      const [roomVaultPda3] = PublicKey.findProgramAddressSync(
        [Buffer.from("room-vault"), roomPda3.toBuffer()],
        program.programId
      );

      try {
        await program.methods
          .initPoolRoom(
            roomId3,
            new anchor.BN(10 * 1_000_000),
            500, // 5% host
            3600, // 36% prize - total = 41% exceeds max 40%
            100,
            null,
            null,
            "Test"
          )
          .accounts({
            room: roomPda3,
            roomVault: roomVaultPda3,
            globalConfig: globalConfigPda,
            host: hostWallet.publicKey,
            feeTokenMint: tokenMint,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([hostWallet])
          .rpc();
        assert.fail("Should have thrown error");
      } catch (err) {
        expect(err.toString()).to.include("TotalAllocationTooHigh");
      }
    });

    it("Fails with invalid prize distribution", async () => {
      const roomId4 = "test-room-4";
      const [roomPda4] = PublicKey.findProgramAddressSync(
        [Buffer.from("room"), hostWallet.publicKey.toBuffer(), Buffer.from(roomId4)],
        program.programId
      );
      const [roomVaultPda4] = PublicKey.findProgramAddressSync(
        [Buffer.from("room-vault"), roomPda4.toBuffer()],
        program.programId
      );

      try {
        await program.methods
          .initPoolRoom(
            roomId4,
            new anchor.BN(10 * 1_000_000),
            300,
            2000,
            50, // Only 50% - doesn't sum to 100%
            30,
            null,
            "Test"
          )
          .accounts({
            room: roomPda4,
            roomVault: roomVaultPda4,
            globalConfig: globalConfigPda,
            host: hostWallet.publicKey,
            feeTokenMint: tokenMint,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([hostWallet])
          .rpc();
        assert.fail("Should have thrown error");
      } catch (err) {
        expect(err.toString()).to.include("InvalidPrizeDistribution");
      }
    });
  });

  describe("Join Room", () => {
    const roomId = "join-test-room";
    let roomPda: PublicKey;
    let roomVaultPda: PublicKey;
    let player1EntryPda: PublicKey;
    let player2EntryPda: PublicKey;

    before(async () => {
      [roomPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("room"), hostWallet.publicKey.toBuffer(), Buffer.from(roomId)],
        program.programId
      );

      [roomVaultPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("room-vault"), roomPda.toBuffer()],
        program.programId
      );

      // Create room first
      await program.methods
        .initPoolRoom(
          roomId,
          new anchor.BN(10 * 1_000_000),
          300,
          2000,
          60,
          30,
          10,
          "Join test"
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

      [player1EntryPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("player"), roomPda.toBuffer(), player1Wallet.publicKey.toBuffer()],
        program.programId
      );

      [player2EntryPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("player"), roomPda.toBuffer(), player2Wallet.publicKey.toBuffer()],
        program.programId
      );
    });

    it("Player 1 joins room successfully", async () => {
      const extrasAmount = new anchor.BN(5 * 1_000_000); // 5 tokens extra donation

      await program.methods
        .joinRoom(roomId, extrasAmount)
        .accounts({
          room: roomPda,
          roomVault: roomVaultPda,
          playerEntry: player1EntryPda,
          player: player1Wallet.publicKey,
          playerTokenAccount: player1TokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([player1Wallet])
        .rpc();

      const room = await program.account.room.fetch(roomPda);
      const entry = await program.account.playerEntry.fetch(player1EntryPda);

      assert.equal(room.playerCount, 1);
      assert.equal(room.totalCollected.toNumber(), 15 * 1_000_000); // 10 entry + 5 extras
      assert.equal(entry.player.toString(), player1Wallet.publicKey.toString());
      assert.equal(entry.entryPaid.toNumber(), 10 * 1_000_000);
      assert.equal(entry.extrasPaid.toNumber(), 5 * 1_000_000);
      assert.equal(entry.totalPaid.toNumber(), 15 * 1_000_000);

      // Verify tokens transferred to vault
      const vaultAccount = await getAccount(provider.connection, roomVaultPda);
      assert.equal(vaultAccount.amount.toString(), (15 * 1_000_000).toString());
    });

    it("Player 2 joins room successfully", async () => {
      await program.methods
        .joinRoom(roomId, new anchor.BN(0)) // No extras
        .accounts({
          room: roomPda,
          roomVault: roomVaultPda,
          playerEntry: player2EntryPda,
          player: player2Wallet.publicKey,
          playerTokenAccount: player2TokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([player2Wallet])
        .rpc();

      const room = await program.account.room.fetch(roomPda);
      assert.equal(room.playerCount, 2);
      assert.equal(room.totalCollected.toNumber(), 25 * 1_000_000); // 15 + 10
    });

    it("Fails when player tries to join twice", async () => {
      try {
        await program.methods
          .joinRoom(roomId, new anchor.BN(0))
          .accounts({
            room: roomPda,
            roomVault: roomVaultPda,
            playerEntry: player1EntryPda,
            player: player1Wallet.publicKey,
            playerTokenAccount: player1TokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([player1Wallet])
          .rpc();
        assert.fail("Should have thrown error");
      } catch (err) {
        // Expected - account already exists
        expect(err.message).to.include("already in use");
      }
    });
  });

  describe("End Room", () => {
    const roomId = "end-test-room";
    let roomPda: PublicKey;
    let roomVaultPda: PublicKey;

    before(async () => {
      [roomPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("room"), hostWallet.publicKey.toBuffer(), Buffer.from(roomId)],
        program.programId
      );

      [roomVaultPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("room-vault"), roomPda.toBuffer()],
        program.programId
      );

      // Create room
      await program.methods
        .initPoolRoom(
          roomId,
          new anchor.BN(100 * 1_000_000), // 100 tokens entry
          400, // 4% host
          3000, // 30% prize pool
          60,
          30,
          10,
          "End test"
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

      // Have 3 players join
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

      const [player3Entry] = PublicKey.findProgramAddressSync(
        [Buffer.from("player"), roomPda.toBuffer(), player3Wallet.publicKey.toBuffer()],
        program.programId
      );

      await program.methods
        .joinRoom(roomId, new anchor.BN(0))
        .accounts({
          room: roomPda,
          roomVault: roomVaultPda,
          playerEntry: player3Entry,
          player: player3Wallet.publicKey,
          playerTokenAccount: player3TokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([player3Wallet])
        .rpc();
    });

    it("Successfully ends room and distributes prizes", async () => {
      // Get initial balances
      const initialPlatformBalance = (await getAccount(provider.connection, platformTokenAccount)).amount;
      const initialCharityBalance = (await getAccount(provider.connection, charityTokenAccount)).amount;
      const initialHostBalance = (await getAccount(provider.connection, hostTokenAccount)).amount;

      const winners = [
        player1Wallet.publicKey, // 1st place
        player2Wallet.publicKey, // 2nd place
        player3Wallet.publicKey, // 3rd place
      ];

      await program.methods
        .endRoom(roomId, winners)
        .accounts({
          room: roomPda,
          roomVault: roomVaultPda,
          globalConfig: globalConfigPda,
          host: hostWallet.publicKey,
          platformWallet: platformWallet.publicKey,
          platformTokenAccount: platformTokenAccount,
          charityWallet: charityWallet.publicKey,
          charityTokenAccount: charityTokenAccount,
          hostTokenAccount: hostTokenAccount,
          firstPlaceWallet: player1Wallet.publicKey,
          firstPlaceTokenAccount: player1TokenAccount,
          secondPlaceWallet: player2Wallet.publicKey,
          secondPlaceTokenAccount: player2TokenAccount,
          thirdPlaceWallet: player3Wallet.publicKey,
          thirdPlaceTokenAccount: player3TokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([hostWallet])
        .rpc();

      const room = await program.account.room.fetch(roomPda);
      assert.equal(room.ended, true);

      // Verify fee distribution
      // Total collected: 300 tokens (3 players * 100 tokens)
      // Platform (20%): 60 tokens
      // Host (4%): 12 tokens
      // Prize pool (30%): 90 tokens
      // Charity (46%): 138 tokens

      const totalCollected = 300 * 1_000_000;
      const expectedPlatform = (totalCollected * 2000) / 10000; // 60 tokens
      const expectedHost = (totalCollected * 400) / 10000; // 12 tokens
      const expectedPrizePool = (totalCollected * 3000) / 10000; // 90 tokens
      const expectedCharity = totalCollected - expectedPlatform - expectedHost - expectedPrizePool; // 138 tokens

      const finalPlatformBalance = (await getAccount(provider.connection, platformTokenAccount)).amount;
      const finalCharityBalance = (await getAccount(provider.connection, charityTokenAccount)).amount;
      const finalHostBalance = (await getAccount(provider.connection, hostTokenAccount)).amount;

      assert.equal(
        Number(finalPlatformBalance - initialPlatformBalance),
        expectedPlatform
      );
      assert.equal(
        Number(finalCharityBalance - initialCharityBalance),
        expectedCharity
      );
      assert.equal(
        Number(finalHostBalance - initialHostBalance),
        expectedHost
      );

      // Verify prize distribution (60%, 30%, 10% of prize pool)
      const expectedFirstPrize = (expectedPrizePool * 60) / 100; // 54 tokens
      const expectedSecondPrize = (expectedPrizePool * 30) / 100; // 27 tokens
      const expectedThirdPrize = (expectedPrizePool * 10) / 100; // 9 tokens

      console.log("Total collected:", totalCollected / 1_000_000, "tokens");
      console.log("Platform fee:", expectedPlatform / 1_000_000, "tokens");
      console.log("Host fee:", expectedHost / 1_000_000, "tokens");
      console.log("Prize pool:", expectedPrizePool / 1_000_000, "tokens");
      console.log("Charity:", expectedCharity / 1_000_000, "tokens");
      console.log("1st prize:", expectedFirstPrize / 1_000_000, "tokens");
      console.log("2nd prize:", expectedSecondPrize / 1_000_000, "tokens");
      console.log("3rd prize:", expectedThirdPrize / 1_000_000, "tokens");
    });

    it("Fails when non-host tries to end room", async () => {
      const roomId2 = "end-test-room-2";
      const [roomPda2] = PublicKey.findProgramAddressSync(
        [Buffer.from("room"), hostWallet.publicKey.toBuffer(), Buffer.from(roomId2)],
        program.programId
      );
      const [roomVaultPda2] = PublicKey.findProgramAddressSync(
        [Buffer.from("room-vault"), roomPda2.toBuffer()],
        program.programId
      );

      // Create room
      await program.methods
        .initPoolRoom(
          roomId2,
          new anchor.BN(10 * 1_000_000),
          300,
          2000,
          100,
          null,
          null,
          "Test"
        )
        .accounts({
          room: roomPda2,
          roomVault: roomVaultPda2,
          globalConfig: globalConfigPda,
          host: hostWallet.publicKey,
          feeTokenMint: tokenMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([hostWallet])
        .rpc();

      try {
        await program.methods
          .endRoom(roomId2, [])
          .accounts({
            room: roomPda2,
            roomVault: roomVaultPda2,
            globalConfig: globalConfigPda,
            host: player1Wallet.publicKey, // Wrong host!
            platformWallet: platformWallet.publicKey,
            platformTokenAccount: platformTokenAccount,
            charityWallet: charityWallet.publicKey,
            charityTokenAccount: charityTokenAccount,
            hostTokenAccount: hostTokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([player1Wallet])
          .rpc();
        assert.fail("Should have thrown error");
      } catch (err) {
        expect(err.toString()).to.include("ConstraintHasOne");
      }
    });
  });

  describe("Edge Cases", () => {
    it("Cannot create room with empty room ID", async () => {
      const emptyRoomId = "";
      const [roomPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("room"), hostWallet.publicKey.toBuffer(), Buffer.from(emptyRoomId)],
        program.programId
      );
      const [roomVaultPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("room-vault"), roomPda.toBuffer()],
        program.programId
      );

      try {
        await program.methods
          .initPoolRoom(
            emptyRoomId,
            new anchor.BN(10 * 1_000_000),
            300,
            2000,
            100,
            null,
            null,
            "Test"
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
        assert.fail("Should have thrown error");
      } catch (err) {
        expect(err.toString()).to.include("InvalidRoomId");
      }
    });

    it("Cannot create room with zero entry fee", async () => {
      const roomId = "zero-fee-room";
      const [roomPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("room"), hostWallet.publicKey.toBuffer(), Buffer.from(roomId)],
        program.programId
      );
      const [roomVaultPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("room-vault"), roomPda.toBuffer()],
        program.programId
      );

      try {
        await program.methods
          .initPoolRoom(
            roomId,
            new anchor.BN(0), // Zero entry fee
            300,
            2000,
            100,
            null,
            null,
            "Test"
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
        assert.fail("Should have thrown error");
      } catch (err) {
        expect(err.toString()).to.include("InvalidEntryFee");
      }
    });
  });
});
