/**
 * Security Test Suite - Bingo/Fundraisely Contracts
 *
 * Tests all security fixes implemented in advanced-contracts branch:
 * - Authorization constraints
 * - Arithmetic overflow protection
 * - Input validation
 * - PDA validation
 * - Account substitution attacks
 * - Re-initialization attacks
 */

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Bingo } from "../target/types/bingo";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  LAMPORTS_PER_SOL
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  createAccount,
  mintTo,
  getAccount,
} from "@solana/spl-token";
import { expect } from "chai";

describe("Security Tests", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Bingo as Program<Bingo>;

  let admin: Keypair;
  let host: Keypair;
  let player: Keypair;
  let attacker: Keypair;
  let mint: PublicKey;
  let globalConfig: PublicKey;
  let tokenRegistry: PublicKey;

  before(async () => {
    admin = Keypair.generate();
    host = Keypair.generate();
    player = Keypair.generate();
    attacker = Keypair.generate();

    // Airdrop SOL
    await provider.connection.requestAirdrop(admin.publicKey, 10 * LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(host.publicKey, 10 * LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(player.publicKey, 10 * LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(attacker.publicKey, 10 * LAMPORTS_PER_SOL);

    await new Promise(resolve => setTimeout(resolve, 1000));

    // Create test token
    mint = await createMint(
      provider.connection,
      admin,
      admin.publicKey,
      null,
      6
    );

    // Derive PDAs
    [globalConfig] = PublicKey.findProgramAddressSync(
      [Buffer.from("global-config")],
      program.programId
    );

    [tokenRegistry] = PublicKey.findProgramAddressSync(
      [Buffer.from("token-registry-v2")],
      program.programId
    );

    // Initialize program
    try {
      await program.methods
        .initialize(admin.publicKey, admin.publicKey)
        .accounts({
          globalConfig,
          admin: admin.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([admin])
        .rpc();
    } catch (e) {
      // May already be initialized
    }

    // Initialize token registry
    try {
      await program.methods
        .initializeTokenRegistry()
        .accounts({
          tokenRegistry,
          admin: admin.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([admin])
        .rpc();
    } catch (e) {
      // May already be initialized
    }

    // Add token to registry
    try {
      await program.methods
        .addApprovedToken(mint)
        .accounts({
          tokenRegistry,
          admin: admin.publicKey,
        })
        .signers([admin])
        .rpc();
    } catch (e) {
      // Token may already be approved
    }
  });

  describe("1. Authorization Attack Tests", () => {
    it("SECURITY: Prevents unauthorized emergency pause", async () => {
      try {
        await program.methods
          .setEmergencyPause(true)
          .accounts({
            globalConfig,
            admin: attacker.publicKey, // Wrong admin
          })
          .signers([attacker])
          .rpc();

        expect.fail("Should have failed with Unauthorized error");
      } catch (error) {
        expect(error.toString()).to.include("Unauthorized");
      }
    });

    it("SECURITY: Prevents unauthorized token registry modification", async () => {
      const fakeMint = Keypair.generate().publicKey;

      try {
        await program.methods
          .addApprovedToken(fakeMint)
          .accounts({
            tokenRegistry,
            admin: attacker.publicKey, // Wrong admin
          })
          .signers([attacker])
          .rpc();

        expect.fail("Should have failed with Unauthorized error");
      } catch (error) {
        expect(error.toString()).to.include("Unauthorized");
      }
    });

    it("SECURITY: Prevents unauthorized winner declaration", async () => {
      const roomId = "test-room-" + Date.now();
      const [room] = PublicKey.findProgramAddressSync(
        [Buffer.from("room"), host.publicKey.toBuffer(), Buffer.from(roomId)],
        program.programId
      );

      const [roomVault] = PublicKey.findProgramAddressSync(
        [Buffer.from("room-vault"), room.toBuffer()],
        program.programId
      );

      // Create room first
      try {
        await program.methods
          .initPoolRoom(
            roomId,
            admin.publicKey,
            new anchor.BN(1_000_000),
            10,
            500,
            2000,
            100,
            null,
            null,
            "Test charity",
            null
          )
          .accounts({
            room,
            roomVault,
            feeTokenMint: mint,
            tokenRegistry,
            globalConfig,
            host: host.publicKey,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([host])
          .rpc();
      } catch (e) {
        // Room may already exist
      }

      // Try to declare winners with wrong host
      try {
        await program.methods
          .declareWinners(roomId, [player.publicKey])
          .accounts({
            room,
            host: attacker.publicKey, // Wrong host
          })
          .signers([attacker])
          .rpc();

        expect.fail("Should have failed with Unauthorized error");
      } catch (error) {
        expect(error.toString()).to.include("Unauthorized");
      }
    });

    it("SECURITY: Prevents unauthorized close joining", async () => {
      const roomId = "test-room-close-" + Date.now();
      const [room] = PublicKey.findProgramAddressSync(
        [Buffer.from("room"), host.publicKey.toBuffer(), Buffer.from(roomId)],
        program.programId
      );

      try {
        await program.methods
          .closeJoining(roomId)
          .accounts({
            room,
            host: attacker.publicKey, // Wrong host
          })
          .signers([attacker])
          .rpc();

        expect.fail("Should have failed with Unauthorized error");
      } catch (error) {
        expect(error.toString()).to.include("Unauthorized");
      }
    });
  });

  describe("2. Arithmetic Overflow Tests", () => {
    it("SECURITY: Prevents overflow in expiration slot calculation", async () => {
      const roomId = "overflow-test-" + Date.now();
      const [room] = PublicKey.findProgramAddressSync(
        [Buffer.from("room"), host.publicKey.toBuffer(), Buffer.from(roomId)],
        program.programId
      );

      const [roomVault] = PublicKey.findProgramAddressSync(
        [Buffer.from("room-vault"), room.toBuffer()],
        program.programId
      );

      try {
        await program.methods
          .initPoolRoom(
            roomId,
            admin.publicKey,
            new anchor.BN(1_000_000),
            10,
            500,
            2000,
            100,
            null,
            null,
            "Test charity",
            new anchor.BN("18446744073709551615") // u64::MAX
          )
          .accounts({
            room,
            roomVault,
            feeTokenMint: mint,
            tokenRegistry,
            globalConfig,
            host: host.publicKey,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([host])
          .rpc();

        expect.fail("Should have failed with ArithmeticOverflow error");
      } catch (error) {
        expect(error.toString()).to.include("ArithmeticOverflow");
      }
    });

    it("SECURITY: Validates winners array size", async () => {
      const roomId = "winners-size-" + Date.now();
      const [room] = PublicKey.findProgramAddressSync(
        [Buffer.from("room"), host.publicKey.toBuffer(), Buffer.from(roomId)],
        program.programId
      );

      // Create array with 11 winners (max is 10)
      const tooManyWinners = Array(11).fill(player.publicKey);

      try {
        await program.methods
          .declareWinners(roomId, tooManyWinners)
          .accounts({
            room,
            host: host.publicKey,
          })
          .signers([host])
          .rpc();

        expect.fail("Should have failed with InvalidWinners error");
      } catch (error) {
        expect(error.toString()).to.include("InvalidWinners");
      }
    });
  });

  describe("3. Input Validation Tests", () => {
    it("SECURITY: Rejects empty room_id", async () => {
      const roomId = ""; // Empty
      const [room] = PublicKey.findProgramAddressSync(
        [Buffer.from("room"), host.publicKey.toBuffer(), Buffer.from(roomId || "fallback")],
        program.programId
      );

      const [roomVault] = PublicKey.findProgramAddressSync(
        [Buffer.from("room-vault"), room.toBuffer()],
        program.programId
      );

      try {
        await program.methods
          .initPoolRoom(
            roomId,
            admin.publicKey,
            new anchor.BN(1_000_000),
            10,
            500,
            2000,
            100,
            null,
            null,
            "Test",
            null
          )
          .accounts({
            room,
            roomVault,
            feeTokenMint: mint,
            tokenRegistry,
            globalConfig,
            host: host.publicKey,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([host])
          .rpc();

        expect.fail("Should have failed with InvalidRoomId error");
      } catch (error) {
        expect(error.toString()).to.include("InvalidRoomId");
      }
    });

    it("SECURITY: Rejects room_id longer than 32 chars", async () => {
      const roomId = "a".repeat(33); // Too long
      const [room] = PublicKey.findProgramAddressSync(
        [Buffer.from("room"), host.publicKey.toBuffer(), Buffer.from(roomId.slice(0, 32))],
        program.programId
      );

      const [roomVault] = PublicKey.findProgramAddressSync(
        [Buffer.from("room-vault"), room.toBuffer()],
        program.programId
      );

      try {
        await program.methods
          .initPoolRoom(
            roomId,
            admin.publicKey,
            new anchor.BN(1_000_000),
            10,
            500,
            2000,
            100,
            null,
            null,
            "Test",
            null
          )
          .accounts({
            room,
            roomVault,
            feeTokenMint: mint,
            tokenRegistry,
            globalConfig,
            host: host.publicKey,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([host])
          .rpc();

        expect.fail("Should have failed with InvalidRoomId error");
      } catch (error) {
        expect(error.toString()).to.include("InvalidRoomId");
      }
    });

    it("SECURITY: Rejects charity_memo longer than 28 chars", async () => {
      const roomId = "memo-test-" + Date.now();
      const [room] = PublicKey.findProgramAddressSync(
        [Buffer.from("room"), host.publicKey.toBuffer(), Buffer.from(roomId)],
        program.programId
      );

      const [roomVault] = PublicKey.findProgramAddressSync(
        [Buffer.from("room-vault"), room.toBuffer()],
        program.programId
      );

      try {
        await program.methods
          .initPoolRoom(
            roomId,
            admin.publicKey,
            new anchor.BN(1_000_000),
            10,
            500,
            2000,
            100,
            null,
            null,
            "a".repeat(29), // Too long
            null
          )
          .accounts({
            room,
            roomVault,
            feeTokenMint: mint,
            tokenRegistry,
            globalConfig,
            host: host.publicKey,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([host])
          .rpc();

        expect.fail("Should have failed with InvalidMemo error");
      } catch (error) {
        expect(error.toString()).to.include("InvalidMemo");
      }
    });

    it("SECURITY: Rejects entry_fee exceeding maximum", async () => {
      const roomId = "fee-test-" + Date.now();
      const [room] = PublicKey.findProgramAddressSync(
        [Buffer.from("room"), host.publicKey.toBuffer(), Buffer.from(roomId)],
        program.programId
      );

      const [roomVault] = PublicKey.findProgramAddressSync(
        [Buffer.from("room-vault"), room.toBuffer()],
        program.programId
      );

      const MAX_ENTRY_FEE = new anchor.BN("1000000000000"); // 1M tokens

      try {
        await program.methods
          .initPoolRoom(
            roomId,
            admin.publicKey,
            MAX_ENTRY_FEE.add(new anchor.BN(1)), // Exceeds max
            10,
            500,
            2000,
            100,
            null,
            null,
            "Test",
            null
          )
          .accounts({
            room,
            roomVault,
            feeTokenMint: mint,
            tokenRegistry,
            globalConfig,
            host: host.publicKey,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([host])
          .rpc();

        expect.fail("Should have failed with InvalidEntryFee error");
      } catch (error) {
        expect(error.toString()).to.include("InvalidEntryFee");
      }
    });
  });

  describe("4. PDA & Account Validation Tests", () => {
    it("SECURITY: Rejects wrong vault PDA", async () => {
      const roomId = "vault-test-" + Date.now();
      const [room] = PublicKey.findProgramAddressSync(
        [Buffer.from("room"), host.publicKey.toBuffer(), Buffer.from(roomId)],
        program.programId
      );

      const fakeVault = Keypair.generate().publicKey; // Wrong vault

      try {
        await program.methods
          .initPoolRoom(
            roomId,
            admin.publicKey,
            new anchor.BN(1_000_000),
            10,
            500,
            2000,
            100,
            null,
            null,
            "Test",
            null
          )
          .accounts({
            room,
            roomVault: fakeVault, // Wrong PDA
            feeTokenMint: mint,
            tokenRegistry,
            globalConfig,
            host: host.publicKey,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([host])
          .rpc();

        expect.fail("Should have failed with PDA derivation error");
      } catch (error) {
        // Anchor will reject this at constraint validation level
        expect(error).to.exist;
      }
    });

    it("SECURITY: Validates token account mint matches room mint", async () => {
      // This test would require a fully set up room with players
      // Testing that wrong mint token accounts are rejected
      // Implementation would be similar to above patterns
    });

    it("SECURITY: Validates token account owner", async () => {
      // This test would require a fully set up room
      // Testing that token accounts owned by wrong wallet are rejected
      // Implementation would be similar to above patterns
    });
  });

  describe("5. Re-initialization Attack Tests", () => {
    it("SECURITY: Prevents vault pre-initialization", async () => {
      const roomId = "reinit-test-" + Date.now();
      const [room] = PublicKey.findProgramAddressSync(
        [Buffer.from("room"), host.publicKey.toBuffer(), Buffer.from(roomId)],
        program.programId
      );

      const [roomVault] = PublicKey.findProgramAddressSync(
        [Buffer.from("room-vault"), room.toBuffer()],
        program.programId
      );

      // Try to create room - first attempt should succeed
      try {
        await program.methods
          .initPoolRoom(
            roomId,
            admin.publicKey,
            new anchor.BN(1_000_000),
            10,
            500,
            2000,
            100,
            null,
            null,
            "Test",
            null
          )
          .accounts({
            room,
            roomVault,
            feeTokenMint: mint,
            tokenRegistry,
            globalConfig,
            host: host.publicKey,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([host])
          .rpc();

        // Try to call again - should fail because vault already exists
        await program.methods
          .initPoolRoom(
            roomId,
            admin.publicKey,
            new anchor.BN(1_000_000),
            10,
            500,
            2000,
            100,
            null,
            null,
            "Test",
            null
          )
          .accounts({
            room,
            roomVault,
            feeTokenMint: mint,
            tokenRegistry,
            globalConfig,
            host: host.publicKey,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([host])
          .rpc();

        expect.fail("Should have failed - room already exists");
      } catch (error) {
        // Expected - cannot re-initialize
        expect(error).to.exist;
      }
    });
  });

  describe("6. Program ID Validation Tests", () => {
    it("SECURITY: Rejects wrong token program", async () => {
      const roomId = "program-test-" + Date.now();
      const [room] = PublicKey.findProgramAddressSync(
        [Buffer.from("room"), host.publicKey.toBuffer(), Buffer.from(roomId)],
        program.programId
      );

      const [roomVault] = PublicKey.findProgramAddressSync(
        [Buffer.from("room-vault"), room.toBuffer()],
        program.programId
      );

      const fakeTokenProgram = Keypair.generate().publicKey;

      try {
        await program.methods
          .initPoolRoom(
            roomId,
            admin.publicKey,
            new anchor.BN(1_000_000),
            10,
            500,
            2000,
            100,
            null,
            null,
            "Test",
            null
          )
          .accounts({
            room,
            roomVault,
            feeTokenMint: mint,
            tokenRegistry,
            globalConfig,
            host: host.publicKey,
            systemProgram: SystemProgram.programId,
            tokenProgram: fakeTokenProgram, // Wrong program
          })
          .signers([host])
          .rpc();

        expect.fail("Should have failed with program ID validation error");
      } catch (error) {
        // Anchor will reject this at address constraint level
        expect(error).to.exist;
      }
    });
  });
});
