import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { BondingCurve } from "../target/types/bonding_curve";
import { PublicKey, Keypair } from "@solana/web3.js";
import { createMint } from "@solana/spl-token";
import { expect } from "chai";

describe("bonding-curve", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.BondingCurve as Program<BondingCurve>;
  const provider = anchor.AnchorProvider.env();

  // Test actors
  let admin: Keypair;
  let nonAdmin: Keypair;
  let feeRecipient: Keypair;

  // A mint we’ll use for curve init
  let tokenMint: PublicKey;

  // Convenience: confirm airdrop
  const airdrop = async (kp: Keypair, lamports: number) => {
    const sig = await provider.connection.requestAirdrop(
      kp.publicKey,
      lamports
    );
    await provider.connection.confirmTransaction(sig, "confirmed");
  };

  before(async () => {
    admin = Keypair.generate();
    nonAdmin = Keypair.generate();
    feeRecipient = Keypair.generate();

    await airdrop(admin, 10 * anchor.web3.LAMPORTS_PER_SOL);
    await airdrop(nonAdmin, 2 * anchor.web3.LAMPORTS_PER_SOL);
    await airdrop(feeRecipient, 1 * anchor.web3.LAMPORTS_PER_SOL);

    tokenMint = await createMint(
      provider.connection,
      admin,
      admin.publicKey, // mint authority
      null, // freeze authority
      6 // decimals
    );
  });

  describe("init_config", () => {
    it("fails with invalid bps (> 10000)", async () => {
      try {
        await program.methods
          .initConfig({
            feeRecipient: feeRecipient.publicKey,
            buyFeeBps: 10001, // invalid
            sellFeeBps: 300,
            allowSellPreGrad: true,
          })
          .accounts({ admin: admin.publicKey }) // omit auto PDAs/programs
          .signers([admin])
          .rpc();
        expect.fail("Expected BadFee");
      } catch (e: any) {
        expect(String(e.message)).to.include("BadFee");
      }
    });

    it("initializes config", async () => {
      const tx = await program.methods
        .initConfig({
          feeRecipient: feeRecipient.publicKey,
          buyFeeBps: 250,
          sellFeeBps: 300,
          allowSellPreGrad: true,
        })
        .accounts({ admin: admin.publicKey })
        .signers([admin])
        .rpc();

      console.log("initConfig tx:", tx);

      const [configPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("config")],
        program.programId
      );
      const cfg = await program.account.config.fetch(configPda);

      expect(cfg.admin.toString()).to.eq(admin.publicKey.toString());
      expect(cfg.feeRecipient.toString()).to.eq(
        feeRecipient.publicKey.toString()
      );
      expect(cfg.buyFeeBps).to.eq(250);
      expect(cfg.sellFeeBps).to.eq(300);
      expect(cfg.allowSellPreGrad).to.eq(true);
    });
  });

  describe("init_curve", () => {
    it("initializes curve + transfers mint authority when requested", async () => {
      const X_V0 = new BN(anchor.web3.LAMPORTS_PER_SOL); // 1 SOL
      const Y_V0 = new BN(1_000_000); // 1e6 tokens (with 6 d.p.)
      const SUPPLY_CAP = new BN(10_000_000);

      const tx = await program.methods
        .initCurve({
          xV0Lamports: X_V0,
          yV0Tokens: Y_V0,
          curveSupplyCap: SUPPLY_CAP,
          takeMintAuthority: true,
        })
        .accounts({
          admin: admin.publicKey,
          tokenMint,
          currentMintAuthority: admin.publicKey, // required when takeMintAuthority = true
          // DO NOT pass config/curve/solVault/tokenProgram/systemProgram
        })
        .signers([admin])
        .rpc();

      console.log("initCurve tx:", tx);

      // PDAs we expect
      const [curvePda] = PublicKey.findProgramAddressSync(
        [Buffer.from("curve"), tokenMint.toBuffer()],
        program.programId
      );
      const [solVaultPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), tokenMint.toBuffer()],
        program.programId
      );
      const [mintAuthPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("mint_auth"), tokenMint.toBuffer()],
        program.programId
      );

      const curve = await program.account.curve.fetch(curvePda);
      expect(curve.tokenMint.toString()).to.eq(tokenMint.toString());
      expect(curve.solVault.toString()).to.eq(solVaultPda.toString());
      expect(curve.mintAuth.toString()).to.eq(mintAuthPda.toString());

      // Verify math/scaling
      const scale = BigInt(curve.scale.toString());
      const x = BigInt(curve.xVScaled.toString());
      const y = BigInt(curve.yVScaled.toString());
      const k = BigInt(curve.kScaled.toString());

      expect(x).to.equal(BigInt(anchor.web3.LAMPORTS_PER_SOL) * scale);
      expect(scale).to.equal(BigInt(10000));
      expect(y).to.equal(BigInt(1_000_000) * scale);
      expect(x * y).to.equal(k);

      // Mint authority actually moved to PDA (parsed check)
      const mintInfo = await provider.connection.getParsedAccountInfo(
        tokenMint
      );
      const parsed = (mintInfo.value?.data as any)?.parsed?.info;
      if (parsed) {
        expect(parsed.mintAuthority).to.eq(mintAuthPda.toString());
      }
    });

    it("fails when supply cap is zero", async () => {
      const newMint = await createMint(
        provider.connection,
        admin,
        admin.publicKey,
        null,
        6
      );

      try {
        await program.methods
          .initCurve({
            xV0Lamports: new BN(anchor.web3.LAMPORTS_PER_SOL),
            yV0Tokens: new BN(1_000_000),
            curveSupplyCap: new BN(0),
            takeMintAuthority: false,
          })
          .accounts({
            admin: admin.publicKey,
            tokenMint: newMint,
            // omit currentMintAuthority + auto PDAs
          })
          .signers([admin])
          .rpc();
        expect.fail("Expected InsufficientInventory");
      } catch (e: any) {
        expect(String(e.message)).to.include("InsufficientInventory");
      }
    });

    it("fails when non-admin attempts init_curve", async () => {
      const theirMint = await createMint(
        provider.connection,
        nonAdmin,
        nonAdmin.publicKey,
        null,
        6
      );

      try {
        await program.methods
          .initCurve({
            xV0Lamports: new BN(anchor.web3.LAMPORTS_PER_SOL),
            yV0Tokens: new BN(1_000_000),
            curveSupplyCap: new BN(10_000_000),
            takeMintAuthority: false,
          })
          .accounts({
            admin: nonAdmin.publicKey, // not the config.admin
            tokenMint: theirMint,
          })
          .signers([nonAdmin])
          .rpc();
        expect.fail("Expected BadAccount");
      } catch (e: any) {
        expect(String(e.message)).to.include("BadAccount");
      }
    });
  });

  describe("buy", () => {
    it("mints tokens to buyer and transfers SOL + fees", async () => {
      // fresh buyer
      const buyer = Keypair.generate();
      await airdrop(buyer, 5 * anchor.web3.LAMPORTS_PER_SOL);

      // PDAs
      const [configPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("config")],
        program.programId
      );
      const [curvePda] = PublicKey.findProgramAddressSync(
        [Buffer.from("curve"), tokenMint.toBuffer()],
        program.programId
      );
      const [vaultPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), tokenMint.toBuffer()],
        program.programId
      );

      // Buyer ATA
      const buyerAta = await anchor.utils.token.associatedAddress({
        mint: tokenMint,
        owner: buyer.publicKey,
      });

      // Run buy
      const tx = await program.methods
        .buy({
          maxPayLamports: new BN(anchor.web3.LAMPORTS_PER_SOL),
          minTokensOut: new BN(1),
        })
        .accounts({
          buyer: buyer.publicKey,
          config: configPda,
          curve: curvePda,
          tokenMint,
          buyerTokenAccount: buyerAta,
          solVault: vaultPda,
          feeRecipient: feeRecipient.publicKey,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
          associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([buyer])
        .rpc();

      console.log("buy tx:", tx);

      // Fetch balances after
      const buyerTokenAcc =
        await program.provider.connection.getTokenAccountBalance(buyerAta);
      const buyerBal = buyerTokenAcc.value.uiAmount ?? 0;

      const vaultBalance = await program.provider.connection.getBalance(
        vaultPda
      );
      const feeBalance = await program.provider.connection.getBalance(
        feeRecipient.publicKey
      );

      expect(buyerBal).to.be.greaterThan(0); // tokens minted
      expect(vaultBalance).to.be.greaterThan(0); // SOL transferred to vault
      expect(feeBalance).to.be.greaterThan(0); // fee recipient got some SOL
    });
  });
});
