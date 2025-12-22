import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  createAccount,
  createMint,
  getAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { expect } from "chai";

import { MagicTrade } from "../target/types/magic_trade";
import { token } from "@coral-xyz/anchor/dist/cjs/utils";
import { sendMagicTransaction } from "magic-router-sdk";
import { MAGIC_CONTEXT_ID, MAGIC_PROGRAM_ID } from "@magicblock-labs/ephemeral-rollups-sdk";

const PLATFORM_SEED = "platform";
const TOKEN_AUTHORITY_SEED = "authority";
const LAMPORT_BANK_SEED = "lamport_bank";
const POOL_SEED = "pool";
const LP_MINT_SEED = "lp_token";
const CUSTODY_SEED = "custody";
const TOKEN_ACCOUNT_SEED = "token_account";
const MARKET_SEED = "market";
const BASKET_SEED = "basket";

const { PublicKey, SystemProgram, Keypair, Transaction } = anchor.web3;

const derivePda = (seeds: (Buffer | Uint8Array)[], programId: anchor.web3.PublicKey) =>
  PublicKey.findProgramAddressSync(seeds, programId)[0];

describe("magic-trade account initialization", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.magicTrade as Program<MagicTrade>;
  const admin = (provider.wallet as anchor.Wallet).payer;

  const oracle0Pubkey = new PublicKey("Dpw1EAVrSB1ibxiDQyTAW6Zip3J4Btk2x4SgApQCeFbX");
  const oracle1Pubkey = new PublicKey("4cSM2e6rvbGQUFiJbqytoVMi5GgghSMr8LwVrT9VPSPo");

  const routerConnection = new anchor.web3.Connection(
    process.env.ROUTER_ENDPOINT || "https://devnet-router.magicblock.app",
    {
      wsEndpoint: process.env.ROUTER_WS_ENDPOINT || "wss://devnet-router.magicblock.app",
    }
  );

  const platformPda = derivePda([Buffer.from(PLATFORM_SEED)], program.programId);
  const transferAuthorityPda = derivePda(
    [Buffer.from(TOKEN_AUTHORITY_SEED)],
    program.programId
  );
  const lamportBankPda = derivePda(
    [Buffer.from(LAMPORT_BANK_SEED)],
    program.programId
  );

  const poolId = 0;
  const poolPda = derivePda(
    [Buffer.from(POOL_SEED), Buffer.from([poolId])],
    program.programId
  );
  const lpMintPda = derivePda(
    [Buffer.from(LP_MINT_SEED), Buffer.from([poolId])],
    program.programId
  );

  const custody0Id = 0;
  const custody0Pda = derivePda(
    [Buffer.from(CUSTODY_SEED), poolPda.toBuffer(), Buffer.from([custody0Id])],
    program.programId
  );
  const custody0TokenPda = derivePda(
    [Buffer.from(TOKEN_ACCOUNT_SEED), custody0Pda.toBuffer()],
    program.programId
  );

  const custody1Id = 1;
  const custody1Pda = derivePda(
    [Buffer.from(CUSTODY_SEED), poolPda.toBuffer(), Buffer.from([custody1Id])],
    program.programId
  );
  const custody1TokenPda = derivePda(
    [Buffer.from(TOKEN_ACCOUNT_SEED), custody1Pda.toBuffer()],
    program.programId
  );

  const market0Id = 0;
  const market0Side = 1; // Long
  const market0Pda = derivePda(
    [
      Buffer.from(MARKET_SEED),
      custody1Pda.toBuffer(),
      custody1Pda.toBuffer(),
      Buffer.from([market0Side]),
    ],
    program.programId
  );

  const market1Id = 1;
  const market1Side = 2; // Short
  const market1Pda = derivePda(
    [
      Buffer.from(MARKET_SEED),
      custody1Pda.toBuffer(),
      custody0Pda.toBuffer(),
      Buffer.from([market1Side]),
    ],
    program.programId
  );

  const basketPda = derivePda(
    [Buffer.from(BASKET_SEED), admin.publicKey.toBuffer()],
    program.programId
  );

  console.log("PDAs", {
    platform: platformPda.toBase58(),
    transferAuthority: transferAuthorityPda.toBase58(),
    lamportBank: lamportBankPda.toBase58(),
    pool: poolPda.toBase58(),
    lpMint: lpMintPda.toBase58(),
    custody0: custody0Pda.toBase58(),
    custody0Token: custody0TokenPda.toBase58(),
    custody1: custody1Pda.toBase58(),
    custody1Token: custody1TokenPda.toBase58(),
    market0: market0Pda.toBase58(),
    market1: market1Pda.toBase58(),
    basket: basketPda.toBase58(),
  });

  let token0Mint: anchor.web3.PublicKey;
  let token1Mint: anchor.web3.PublicKey;
  let ownerToken0Account: anchor.web3.PublicKey;
  let ownerToken1Account: anchor.web3.PublicKey;
  let ownerLpTokenAccount: anchor.web3.PublicKey;
  it("initializes tokens, platform, pool, custody, market, and basket", async () => {
    token0Mint = await createMint(
      provider.connection,
      admin,
      admin.publicKey,
      null,
      6,
      undefined,
      undefined,
      TOKEN_PROGRAM_ID
    );
    console.log("token0 mint", token0Mint.toBase58());

    token1Mint = await createMint(
      provider.connection,
      admin,
      admin.publicKey,
      null,
      8,
      undefined,
      undefined,
      TOKEN_PROGRAM_ID
    );
    console.log("token1 mint", token1Mint.toBase58());

    ownerToken0Account = await createAccount(
      provider.connection,
      admin,
      token0Mint,
      admin.publicKey
    );

    console.log("owner token0 account", ownerToken0Account.toBase58());

    ownerToken1Account = await createAccount(
      provider.connection,
      admin,
      token1Mint,
      admin.publicKey
    );

    console.log("owner token1 account", ownerToken1Account.toBase58());

    await mintTo(
      provider.connection,
      admin,
      token0Mint,
      ownerToken0Account,
      admin.publicKey,
      1_000_000_000
    );

    await mintTo(
      provider.connection,
      admin,
      token1Mint,
      ownerToken1Account,
      admin.publicKey,
      1_000_000_000
    );

    const permissions = {
      liquidityAdd: true,
      liquidityRemove: true,
      tradeInit: true,
      tradeMaint: true,
      tradeLiquidation: true,
      padding: [0, 0, 0],
    };

    const initTxn = await program.methods
      .initialize(1, permissions)
      .accountsStrict({
        platform: platformPda,
        transferAuthority: transferAuthorityPda,
        lamportBank: lamportBankPda,
        admin: admin.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("initialize platform tx", initTxn);

    const initPoolTxn = await program.methods
      .initializePool(poolId, new anchor.BN(1_000_000), new anchor.BN(500_000), oracle0Pubkey)
      .accountsStrict({
        platform: platformPda,
        transferAuthority: transferAuthorityPda,
        lpMint: lpMintPda,
        pool: poolPda,
        admin: admin.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    console.log("initialize pool tx", initPoolTxn);

    const marginParams = {
      tradeFeeBps: 10,
      tradeSpreadMin: new anchor.BN(0),
      tradeSpreadMax: new anchor.BN(0),
      minInitLeverage: 1_0000,
      maxInitLeverage: 100_0000,
      maxLeverage: 200_0000,
      maxUtilization: 10_000,
      minCollateralUsd: 100_000,
      padding: [0, 0, 0, 0],
      virtualDelay: new anchor.BN(0),
      maxPositionSizeUsd: new anchor.BN(0),
      maxExposureUsd: new anchor.BN(0),
    };

    const initCustody0Txn = await program.methods
      .initializeCustody(
        custody0Id,
        6,
        false,
        false,
        permissions,
        new anchor.BN(1_000_000),
        marginParams
      )
      .accountsStrict({
        platform: platformPda,
        transferAuthority: transferAuthorityPda,
        pool: poolPda,
        custody: custody0Pda,
        tokenMint: token0Mint,
        tokenAccount: custody0TokenPda,
        oracle: oracle0Pubkey,
        admin: admin.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    console.log("initialize custody0 tx", initCustody0Txn);

    const initCustody1Txn = await program.methods
      .initializeCustody(
        custody1Id,
        6,
        false,
        false,
        permissions,
        new anchor.BN(1_000_000),
        marginParams
      )
      .accountsStrict({
        platform: platformPda,
        transferAuthority: transferAuthorityPda,
        pool: poolPda,
        custody: custody1Pda,
        tokenMint: token1Mint,
        tokenAccount: custody1TokenPda,
        oracle: oracle1Pubkey,
        admin: admin.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    console.log("initialize custody1 tx", initCustody1Txn);

    const initMarket0Txn = await program.methods
      .initializeMarket(market0Id, { long: {} } as any, false, permissions)
      .accountsStrict({
        platform: platformPda,
        pool: poolPda,
        targetCustody: custody1Pda,
        lockCustody: custody1Pda,
        market: market0Pda,
        admin: admin.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("initialize market0 tx", initMarket0Txn);

    const initMarket1Txn = await program.methods
      .initializeMarket(market1Id, { short: {} } as any, false, permissions)
      .accountsStrict({
        platform: platformPda,
        pool: poolPda,
        targetCustody: custody1Pda,
        lockCustody: custody0Pda,
        market: market1Pda,
        admin: admin.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const initBasketTxn = await program.methods
      .initializeBasket()
      .accountsStrict({
        basket: basketPda,
        owner: admin.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("initialize basket tx", initBasketTxn);

    ownerLpTokenAccount = await createAccount(
      provider.connection,
      admin,
      lpMintPda,
      admin.publicKey
    );

    console.log("owner lp account", ownerLpTokenAccount.toBase58());

    const platformAccount = await program.account.platform.fetch(platformPda);
    expect(platformAccount.poolCount).to.equal(1);
    expect(platformAccount.admin.toBase58()).to.equal(admin.publicKey.toBase58());

    const poolAccount = await program.account.pool.fetch(poolPda);
    expect(poolAccount.custodies).to.have.length(2);
    expect(poolAccount.markets).to.have.length(2);

    const custody0Account = await program.account.custody.fetch(custody0Pda);
    expect(custody0Account.tokenMint.toBase58()).to.equal(token0Mint.toBase58());
    expect(custody0Account.tokenAccount.toBase58()).to.equal(
      custody0TokenPda.toBase58()
    );

    const market0Account = await program.account.market.fetch(market0Pda);
    expect(market0Account.targetCustody.toBase58()).to.equal(custody1Pda.toBase58());
    expect(market0Account.lockCustody.toBase58()).to.equal(custody1Pda.toBase58());

    const market1Account = await program.account.market.fetch(market1Pda);
    expect(market1Account.targetCustody.toBase58()).to.equal(custody1Pda.toBase58());
    expect(market1Account.lockCustody.toBase58()).to.equal(custody0Pda.toBase58());

    const basketAccount = await program.account.basket.fetch(basketPda);
    expect(basketAccount.basketBump).to.be.a("number");
  }).timeout(120_000);

  it("deposits into and withdraws from basket custody", async () => {
    // Ensure previous init ran
    expect(token0Mint).to.exist;

    const depositAmount = 100_000_000;

    const depositTxn = await program.methods
      .depositCollateral(new anchor.BN(depositAmount))
      .accountsStrict({
        owner: admin.publicKey,
        ownerTokenAccount: ownerToken0Account,
        pool: poolPda,
        custody: custody0Pda,
        custodyTokenAccount: custody0TokenPda,
        basket: basketPda,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    console.log("deposit funds tx", depositTxn);

    const basketAfterDeposit = await program.account.basket.fetch(basketPda);
    expect(Number(basketAfterDeposit.deposits[0].amount)).to.equal(depositAmount);

    const custodyAfterDeposit = await program.account.custody.fetch(custody0Pda);
    expect(Number(custodyAfterDeposit.assets.reserved)).to.equal(depositAmount);

    const withdrawAmount = 50_000_000;
    const withdrawTxn = await program.methods
      .withdrawCollateral(new anchor.BN(withdrawAmount))
      .accountsStrict({
        owner: admin.publicKey,
        ownerTokenAccount: ownerToken0Account,
        pool: poolPda,
        custody: custody0Pda,
        custodyTokenAccount: custody0TokenPda,
        basket: basketPda,
        tokenAuthority: transferAuthorityPda,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    console.log("withdraw funds tx", withdrawTxn);

    const basketAfterWithdraw = await program.account.basket.fetch(basketPda);
    expect(Number(basketAfterWithdraw.deposits[0].amount)).to.equal(
      depositAmount - withdrawAmount
    );

    const ownerBalance = Number(
      (await getAccount(provider.connection, ownerToken0Account)).amount
    );
    expect(ownerBalance).to.be.greaterThan(0);
  }).timeout(120_000);

  it("adds liquidity", async () => {

    const add0Txn = await program.methods
      .addLiquidity(new anchor.BN(250_000_000))
      .accountsPartial({
        owner: admin.publicKey,
        ownerTokenAccount: ownerToken0Account,
        ownerLpAccount: ownerLpTokenAccount,
        pool: poolPda,
        custody: custody0Pda,
        oracle: oracle0Pubkey,
        custodyTokenAccount: custody0TokenPda,
        lpTokenMint: lpMintPda,
        tokenAuthority: transferAuthorityPda,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .remainingAccounts([
        { pubkey: custody0Pda, isWritable: false, isSigner: false },
        { pubkey: custody1Pda, isWritable: false, isSigner: false },
        { pubkey: oracle0Pubkey, isWritable: false, isSigner: false },
        { pubkey: oracle1Pubkey, isWritable: false, isSigner: false },
        { pubkey: market0Pda, isWritable: false, isSigner: false },
        { pubkey: market1Pda, isWritable: false, isSigner: false },
      ])
      .rpc();

    console.log("add liquidity0 tx", add0Txn);

    const add1Txn = await program.methods
      .addLiquidity(new anchor.BN(250_000_000))
      .accountsPartial({
        owner: admin.publicKey,
        ownerTokenAccount: ownerToken1Account,
        ownerLpAccount: ownerLpTokenAccount,
        pool: poolPda,
        custody: custody1Pda,
        oracle: oracle1Pubkey,
        custodyTokenAccount: custody1TokenPda,
        lpTokenMint: lpMintPda,
        tokenAuthority: transferAuthorityPda,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .remainingAccounts([
        { pubkey: custody0Pda, isWritable: false, isSigner: false },
        { pubkey: custody1Pda, isWritable: false, isSigner: false },
        { pubkey: oracle0Pubkey, isWritable: false, isSigner: false },
        { pubkey: oracle1Pubkey, isWritable: false, isSigner: false },
        { pubkey: market0Pda, isWritable: false, isSigner: false },
        { pubkey: market1Pda, isWritable: false, isSigner: false },
      ])
      .rpc();

    console.log("add liquidity1 tx", add1Txn);
  }).timeout(120_000);

  it("opens a position", async () => {

    const openTxn = await program.methods
      .openPosition(new anchor.BN(5_000_000), new anchor.BN(1_000))
      .accountsPartial({
        owner: admin.publicKey,
        basket: basketPda,
        pool: poolPda,
        market: market0Pda,
        targetCustody: custody1Pda,
        lockCustody: custody1Pda,
        collateralCustody: custody0Pda,
        targetOracle: oracle1Pubkey, 
        lockOracle: oracle1Pubkey,
        collateralOracle: oracle0Pubkey,
      })
      .rpc();

    console.log("open position tx", openTxn);
  }).timeout(120_000);

  it("add collateral to position", async () => {
    const addCollateralAmount = new anchor.BN(2_000_000);  
    const sizeAmount = new anchor.BN(500);                 

    const addCollateralTxn = await program.methods
      .processAddCollateralToPosition(addCollateralAmount, sizeAmount)
      .accountsPartial({
        owner: admin.publicKey,
        basket: basketPda,
        market: market0Pda,
        pool: poolPda,
        targetCustody: custody1Pda,
        collateralCustody: custody0Pda,
        lockCustody: custody1Pda,
        targetOracle: oracle1Pubkey, 
        collateralOracle: oracle0Pubkey,
        lockOracle: oracle1Pubkey,
      })
      .rpc();

      console.log("Added collateral to position tx: ", addCollateralTxn);
  });

  it.skip("remove collateral from position", async () => {
    const removeCollateralAmount = new anchor.BN(1_000_000);  
    const sizeAmount = new anchor.BN(250);    

    const removeCollateralTxn = await program.methods.removeCollateralFromPosition(removeCollateralAmount, sizeAmount).accountsPartial({
      owner: admin.publicKey,
      basket: basketPda,
      market: market0Pda,
      pool: poolPda,
      targetCustody: custody1Pda,
      collateralCustody: custody0Pda,
      lockCustody: custody1Pda,
      targetOracle: oracle1Pubkey, 
      collateralOracle: oracle0Pubkey,
      lockOracle: oracle1Pubkey,
    }).rpc();

    console.log("remove collateral from position tx: ", removeCollateralTxn);
  })

  it.skip("deposits into basket custody", async () => {
    const depositAmount = 100_000_000;
  
    const depositTxn = await program.methods
      .depositCollateral(new anchor.BN(depositAmount))
      .accountsStrict({
        owner: admin.publicKey,
        ownerTokenAccount: ownerToken0Account,
        pool: poolPda,
        custody: custody0Pda,
        custodyTokenAccount: custody0TokenPda,
        basket: basketPda,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();
  
    console.log("deposit funds tx", depositTxn);
  }).timeout(120_000);

  it.skip("closes a position", async () => {
    const closeTxn = await program.methods
      .closePosition()
      .accountsPartial({
        owner: admin.publicKey,
        basket: basketPda,
        pool: poolPda,
        market: market0Pda,
        targetCustody: custody1Pda,
        lockCustody: custody1Pda,
        collateralCustody: custody0Pda,
        targetOracle: oracle1Pubkey, 
        collateralOracle: oracle0Pubkey,
      })
      .rpc();

    console.log("close position tx", closeTxn);
  }).timeout(120_000);

  it.skip("removes liquidity", async () => {

    const custody0Oracle = (await program.account.custody.fetch(custody0Pda)).oracle;

    const removeTxn = await program.methods
      .removeLiquidity(new anchor.BN(10_000))
      .accountsPartial({
        owner: admin.publicKey,
        ownerLpAccount: ownerLpTokenAccount,
        ownerTokenAccount: custody0TokenPda,
        pool: poolPda,
        custody: custody0Pda,
        oracle: custody0Oracle,
        custodyTokenAccount: custody0TokenPda,
        lpTokenMint: lpMintPda,
        tokenAuthority: transferAuthorityPda,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .remainingAccounts([
        { pubkey: custody0Pda, isWritable: false, isSigner: false },
        { pubkey: custody1Pda, isWritable: false, isSigner: false },
        { pubkey: custody0Oracle, isWritable: false, isSigner: false },
        { pubkey: oracle1Pubkey, isWritable: false, isSigner: false },
        { pubkey: market0Pda, isWritable: false, isSigner: false },
        { pubkey: market1Pda, isWritable: false, isSigner: false },
      ])
      .rpc();

    console.log("remove liquidity tx", removeTxn);
  }).timeout(120_000);

  it("delegates pool, custody, and basket", async () => {
    const commitFrequency = 10_000;
    const validatorKey = new PublicKey("MAS1Dt9qreoRMQ14YQuhg8UTZMMzDdKhmkZMECCzk57");

    const delegatePoolTxn = await program.methods
      .delegatePool(commitFrequency, validatorKey)
      .accountsPartial({
        payer: admin.publicKey,
        pool: poolPda,
      })
      .rpc();

    console.log("delegate pool tx", delegatePoolTxn);


    const delegateCustody0Txn = await program.methods
      .delegateCustody(commitFrequency, validatorKey)
      .accountsPartial({
        payer: admin.publicKey,
        pool: poolPda,
        custody: custody0Pda,
      })
      .rpc();

    console.log("delegate custody0 tx", delegateCustody0Txn);

    const delegateCustody1Txn = await program.methods
      .delegateCustody(commitFrequency, validatorKey)
      .accountsPartial({
        payer: admin.publicKey,
        pool: poolPda,
        custody: custody1Pda,
      })
      .rpc();

    console.log("delegate custody1 tx", delegateCustody1Txn);

    const delegateMarketTxn = await program.methods
      .delegateMarket(commitFrequency, validatorKey)
      .accountsPartial({
        payer: admin.publicKey,
        targetCustody: custody1Pda,
        lockCustody: custody1Pda,
        market: market0Pda,
      })
      .rpc();

    console.log("delegate market tx", delegateMarketTxn);

    const delegateBasketTxn = await program.methods
      .delegateBasket(commitFrequency, validatorKey)
      .accountsPartial({
        payer: admin.publicKey,
        owner: admin.publicKey,
        basket: basketPda,
      })
      .rpc();

    console.log("delegate basket tx", delegateBasketTxn);
  }).timeout(60_000);

  it.skip("Open position ER", async () => {
    const transaction = await program.methods
      .openPosition(new anchor.BN(5_000_000), new anchor.BN(1_000))
      .accountsPartial({
        owner: admin.publicKey,
        basket: basketPda,
        pool: poolPda,
        market: market0Pda,
        targetCustody: custody1Pda,
        lockCustody: custody1Pda,
        collateralCustody: custody0Pda,
        targetOracle: oracle1Pubkey,
        lockOracle: oracle1Pubkey,
        collateralOracle: oracle0Pubkey,
      })
      .transaction();

    const { blockhash } = await routerConnection.getLatestBlockhash();
    transaction.recentBlockhash = blockhash;
    transaction.feePayer = admin.publicKey;

    const signature = await anchor.web3.sendAndConfirmTransaction(
      routerConnection,
      transaction,
      [admin],
      { skipPreflight: false } 
    );

    console.log(`Open position ER: ${signature}`);
  }).timeout(120_000);

  // it("Withdraw from basked custody", async () => {
  //   const withdrawAmount = 50_000_000;

  //   const withdrawTxn = await program.methods
  //     .withdrawCollateral(new anchor.BN(withdrawAmount))
  //     .accountsStrict({
  //       owner: admin.publicKey,
  //       ownerTokenAccount: ownerToken0Account,
  //       pool: poolPda,
  //       custody: custody0Pda,
  //       custodyTokenAccount: custody0TokenPda,
  //       basket: basketPda,
  //       tokenAuthority: transferAuthorityPda,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //     })
  //     .rpc();

  //   console.log("withdraw funds tx", withdrawTxn);
  // })

  it.skip("Close position ER", async () => {
    const transaction = await program.methods.closePosition().accountsPartial({
      owner: admin.publicKey,
      basket: basketPda,
      pool: poolPda,
      market: market0Pda,
      targetCustody: custody1Pda,
      lockCustody: custody1Pda,
      collateralCustody: custody0Pda,
      targetOracle: oracle1Pubkey,
      collateralOracle: oracle0Pubkey,
    })
    .transaction();

    const { blockhash } = await routerConnection.getLatestBlockhash();
    transaction.recentBlockhash = blockhash;
    transaction.feePayer = admin.publicKey;

    const signature = await anchor.web3.sendAndConfirmTransaction(
      routerConnection,
      transaction,
      [admin],
      { skipPreflight: false } 
    );

    console.log(`Close position ER: ${signature}`);
  });

  // it("add collateral after delegation", async () => {
  //   const addCollateralAmount = new anchor.BN(2_000_000);  
  //   const sizeAmount = new anchor.BN(500);                 

  //   const addCollateralTxn = await program.methods
  //     .addCollateralToPosition(addCollateralAmount, sizeAmount)
  //     .accountsPartial({
  //       owner: admin.publicKey,
  //       basket: basketPda,
  //       market: market0Pda,
  //       pool: poolPda,
  //       targetCustody: custody1Pda,
  //       collateralCustody: custody0Pda,
  //       lockCustody: custody1Pda,
  //       targetOracle: oracle1Pubkey, 
  //       collateralOracle: oracle0Pubkey,
  //       lockOracle: oracle1Pubkey,
  //     })
  //     .rpc();

  //     console.log("Added collateral to position tx: ", addCollateralTxn);
  // })

  // it("Commit and add collateral to position", async () => {
  //   const addCollateralAmount = new anchor.BN(2_000_000);  
  //   const sizeAmount = new anchor.BN(500);           

  //   const tx = await program.methods.commitAndAddCollateralToPosition(addCollateralAmount, sizeAmount).accountsPartial({
  //     owner: admin.publicKey,
  //     basket: basketPda,
  //     market: market0Pda,
  //     pool: poolPda,
  //     targetCustody: custody1Pda,
  //     collateralCustody: custody0Pda,
  //     lockCustody: custody1Pda,
  //     targetOracle: oracle1Pubkey,
  //     collateralOracle: oracle0Pubkey,
  //     lockOracle: oracle1Pubkey,
  //     magicContext: MAGIC_CONTEXT_ID,
  //     magicProgram: MAGIC_PROGRAM_ID,
  //   }).transaction();

  //   const signature = await sendMagicTransaction(
  //     routerConnection,
  //     tx,
  //     [admin]
  //   );

  //   await sleepWithAnimation(15);
  //   console.log(`Transaction Signature: ${signature}`);
  // })

  it("Commit, undelegate, add collateral to position and re-delegate", async () => {
    let tx = await program.methods.processCommitAndUndelegateAccounts().accountsPartial({
      owner: admin.publicKey,
      basket: basketPda,
      market: market0Pda,  
      pool: poolPda,
      targetCustody: custody1Pda,
      collateralCustody: custody0Pda,
      lockCustody: custody1Pda,
      targetOracle: oracle1Pubkey,
      collateralOracle: oracle0Pubkey,
      lockOracle: oracle1Pubkey,
    }).transaction();

    let signature = await sendMagicTransaction(
      routerConnection,
      tx,
      [admin]
    );

    await sleepWithAnimation(30);
    console.log(`Transaction Signature: ${signature}`);

    const addCollateralAmount = new anchor.BN(2_000_000);  
    const sizeAmount = new anchor.BN(500);                 

    const addCollateralTxn = await program.methods
      .processAddCollateralToPosition(addCollateralAmount, sizeAmount)
      .accountsPartial({
        owner: admin.publicKey,
        basket: basketPda,
        market: market0Pda,
        pool: poolPda,
        targetCustody: custody1Pda,
        collateralCustody: custody0Pda,
        lockCustody: custody1Pda,
        targetOracle: oracle1Pubkey, 
        collateralOracle: oracle0Pubkey,
        lockOracle: oracle1Pubkey,
      }).signers([admin]).rpc();

    console.log(`Add collateral to position: ${addCollateralTxn}`);

    //Delegate back the accounts
    await sleepWithAnimation(10);

    const commitFrequency = 10_000;
    const validatorKey = new PublicKey("MAS1Dt9qreoRMQ14YQuhg8UTZMMzDdKhmkZMECCzk57");

    const delegatePoolTxn = await program.methods
      .delegatePool(commitFrequency, validatorKey)
      .accountsPartial({
        payer: admin.publicKey,
        pool: poolPda,
      })
      .rpc();

    console.log("delegate pool tx", delegatePoolTxn);

    const delegateCustody0Txn = await program.methods
      .delegateCustody(commitFrequency, validatorKey)
      .accountsPartial({
        payer: admin.publicKey,
        pool: poolPda,
        custody: custody0Pda,
      })
      .rpc();

    console.log("delegate custody0 tx", delegateCustody0Txn);

    const delegateCustody1Txn = await program.methods
      .delegateCustody(commitFrequency, validatorKey)
      .accountsPartial({
        payer: admin.publicKey,
        pool: poolPda,
        custody: custody1Pda,
      })
      .rpc();

    console.log("delegate custody1 tx", delegateCustody1Txn);

    const delegateMarketTxn = await program.methods
      .delegateMarket(commitFrequency, validatorKey)
      .accountsPartial({
        payer: admin.publicKey,
        targetCustody: custody1Pda,
        lockCustody: custody1Pda,
        market: market0Pda,
      })
      .rpc();

    console.log("delegate market tx", delegateMarketTxn);

    const delegateBasketTxn = await program.methods
      .delegateBasket(commitFrequency, validatorKey)
      .accountsPartial({
        payer: admin.publicKey,
        owner: admin.publicKey,
        basket: basketPda,
      })
      .rpc();

    console.log("delegate basket tx", delegateBasketTxn);

    await sleepWithAnimation(15);
  });

  it("Commit, undelegate, remove collateral from position and re-delegate", async () => {
    let tx = await program.methods.processCommitAndUndelegateAccounts().accountsPartial({
      owner: admin.publicKey,
      basket: basketPda,
      market: market0Pda,
      pool: poolPda,
      targetCustody: custody1Pda,
      collateralCustody: custody0Pda,
      lockCustody: custody1Pda,
      targetOracle: oracle1Pubkey,
      collateralOracle: oracle0Pubkey,
      lockOracle: oracle1Pubkey,
    }).transaction();

    let signature = await sendMagicTransaction(
      routerConnection,
      tx,
      [admin]
    );

    await sleepWithAnimation(30);
    console.log(`Transaction Signature: ${signature}`);

    const removeCollateralAmount = new anchor.BN(1_000_00);  
    const sizeAmount = new anchor.BN(250);    

    const removeCollateralTxn = await program.methods.removeCollateralFromPosition(removeCollateralAmount, sizeAmount).accountsPartial({
      owner: admin.publicKey,
      basket: basketPda,
      market: market0Pda,
      pool: poolPda,
      targetCustody: custody1Pda,
      collateralCustody: custody0Pda,
      lockCustody: custody1Pda,
      targetOracle: oracle1Pubkey, 
      collateralOracle: oracle0Pubkey,
      lockOracle: oracle1Pubkey,
    }).rpc();

    console.log("remove collateral from position tx: ", removeCollateralTxn);

    //Delegate back the accounts
    await sleepWithAnimation(10);

    const commitFrequency = 10_000;
    const validatorKey = new PublicKey("MAS1Dt9qreoRMQ14YQuhg8UTZMMzDdKhmkZMECCzk57");

    const delegatePoolTxn = await program.methods
      .delegatePool(commitFrequency, validatorKey)
      .accountsPartial({
        payer: admin.publicKey,
        pool: poolPda,
      })
      .rpc();

    console.log("delegate pool tx", delegatePoolTxn);

    const delegateCustody0Txn = await program.methods
      .delegateCustody(commitFrequency, validatorKey)
      .accountsPartial({
        payer: admin.publicKey,
        pool: poolPda,
        custody: custody0Pda,
      })
      .rpc();

    console.log("delegate custody0 tx", delegateCustody0Txn);

    const delegateCustody1Txn = await program.methods
      .delegateCustody(commitFrequency, validatorKey)
      .accountsPartial({
        payer: admin.publicKey,
        pool: poolPda,
        custody: custody1Pda,
      })
      .rpc();

    console.log("delegate custody1 tx", delegateCustody1Txn);

    const delegateMarketTxn = await program.methods
      .delegateMarket(commitFrequency, validatorKey)
      .accountsPartial({
        payer: admin.publicKey,
        targetCustody: custody1Pda,
        lockCustody: custody1Pda,
        market: market0Pda,
      })
      .rpc();

    console.log("delegate market tx", delegateMarketTxn);

    const delegateBasketTxn = await program.methods
      .delegateBasket(commitFrequency, validatorKey)
      .accountsPartial({
        payer: admin.publicKey,
        owner: admin.publicKey,
        basket: basketPda,
      })
      .rpc();

    console.log("delegate basket tx", delegateBasketTxn);
  })
});

async function sleepWithAnimation(seconds: number): Promise<void> {
  const totalMs = seconds * 1000;
  const interval = 500; // Update every 500ms
  const iterations = Math.floor(totalMs / interval);

  for (let i = 0; i < iterations; i++) {
    const dots = '.'.repeat((i % 3) + 1);
    process.stdout.write(`\rWaiting${dots}   `);
    await new Promise(resolve => setTimeout(resolve, interval));
  }

  // Clear the line
  process.stdout.write('\r\x1b[K');
}