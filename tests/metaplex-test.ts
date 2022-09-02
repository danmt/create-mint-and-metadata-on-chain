import {
  bundlrStorage,
  isMintWithMetadata,
  Metaplex,
  walletAdapterIdentity,
} from "@metaplex-foundation/js";
import * as anchor from "@project-serum/anchor";
import { AnchorError, Program } from "@project-serum/anchor";
import {
  createAssociatedTokenAccountInstruction,
  getAccount,
  getAssociatedTokenAddress,
  getMint,
} from "@solana/spl-token";
import { BN } from "bn.js";
import { assert } from "chai";
import { Disco } from "../target/types/disco";
import {
  createFundedWallet,
  createMint,
  createUserAndAssociatedWallet,
} from "./utils";

describe("disco", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Disco as Program<Disco>;
  const eventBaseKeypair = anchor.web3.Keypair.generate();
  const eventGeneralTicketBaseKeypair = anchor.web3.Keypair.generate();
  const eventVipTicketBaseKeypair = anchor.web3.Keypair.generate();
  const collaborator1Keypair = anchor.web3.Keypair.generate();
  const metadataProgramPublicKey = new anchor.web3.PublicKey(
    "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
  );
  const metaplex = Metaplex.make(provider.connection)
    .use(walletAdapterIdentity(provider.wallet))
    .use(bundlrStorage());
  const aliceBalance = 5000;
  const eventFeeVaultBalance = anchor.web3.LAMPORTS_PER_SOL;
  let aliceKeypair: anchor.web3.Keypair;
  let aliceAssociatedWalletPublicKey: anchor.web3.PublicKey;
  let eventPublicKey: anchor.web3.PublicKey;
  let eventVaultPublicKey: anchor.web3.PublicKey;
  let feeVaultPublicKey: anchor.web3.PublicKey;
  let aliceGeneralTicketAssociatedTokenPublicKey: anchor.web3.PublicKey;
  let aliceVipTicketAssociatedTokenPublicKey: anchor.web3.PublicKey;
  let acceptedMintPublicKey: anchor.web3.PublicKey;
  let eventGeneralTicketPublicKey: anchor.web3.PublicKey;
  let eventVipTicketPublicKey: anchor.web3.PublicKey;
  let eventGeneralTicketMintPublicKey: anchor.web3.PublicKey;
  let eventVipTicketMintPublicKey: anchor.web3.PublicKey;
  let feeVaultRentExemption: number;
  let collaborator1PublicKey: anchor.web3.PublicKey;

  before(async () => {
    [eventPublicKey] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("event", "utf-8"), eventBaseKeypair.publicKey.toBuffer()],
      program.programId
    );
    [eventGeneralTicketPublicKey] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from("event_ticket", "utf-8"),
          eventPublicKey.toBuffer(),
          eventGeneralTicketBaseKeypair.publicKey.toBuffer(),
        ],
        program.programId
      );
    [eventVipTicketPublicKey] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("event_ticket", "utf-8"),
        eventPublicKey.toBuffer(),
        eventVipTicketBaseKeypair.publicKey.toBuffer(),
      ],
      program.programId
    );
    [eventGeneralTicketMintPublicKey] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from("ticket_mint", "utf-8"),
          eventPublicKey.toBuffer(),
          eventGeneralTicketPublicKey.toBuffer(),
        ],
        program.programId
      );
    [eventVipTicketMintPublicKey] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from("ticket_mint", "utf-8"),
          eventPublicKey.toBuffer(),
          eventVipTicketPublicKey.toBuffer(),
        ],
        program.programId
      );
    [eventVaultPublicKey] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("event_vault", "utf-8"), eventPublicKey.toBuffer()],
      program.programId
    );
    [feeVaultPublicKey] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("fee_vault", "utf-8"), eventPublicKey.toBuffer()],
      program.programId
    );
    [collaborator1PublicKey] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("collaborator", "utf-8"),
        eventPublicKey.toBuffer(),
        collaborator1Keypair.publicKey.toBuffer(),
      ],
      program.programId
    );

    aliceKeypair = await createFundedWallet(provider);
    acceptedMintPublicKey = await createMint(provider);
    aliceAssociatedWalletPublicKey = await createUserAndAssociatedWallet(
      provider,
      acceptedMintPublicKey,
      aliceBalance,
      aliceKeypair
    );
    aliceGeneralTicketAssociatedTokenPublicKey =
      await getAssociatedTokenAddress(
        eventGeneralTicketMintPublicKey,
        aliceKeypair.publicKey
      );
    aliceVipTicketAssociatedTokenPublicKey = await getAssociatedTokenAddress(
      eventVipTicketMintPublicKey,
      aliceKeypair.publicKey
    );
    feeVaultRentExemption =
      await provider.connection.getMinimumBalanceForRentExemption(0);
  });

  it("should create Tomorrowland 2022 event", async () => {
    // arrange
    const eventTitle = "Tomorrowland 2022";
    const feeVaultAmount = eventFeeVaultBalance + feeVaultRentExemption;
    // act
    await program.methods
      .createEvent(eventTitle, new BN(feeVaultAmount))
      .accounts({
        authority: provider.wallet.publicKey,
        eventBase: eventBaseKeypair.publicKey,
        acceptedMint: acceptedMintPublicKey,
      })
      .rpc();
    // assert
    const eventAccount = await program.account.event.fetch(eventPublicKey);
    const eventVaultAccount = await getAccount(
      provider.connection,
      eventVaultPublicKey
    );
    const feeVaultAccountInfo = await provider.connection.getAccountInfo(
      feeVaultPublicKey
    );
    assert.isDefined(eventAccount);
    assert.equal(eventAccount.eventTitle, eventTitle);
    assert.isTrue(eventAccount.authority.equals(provider.wallet.publicKey));
    assert.isTrue(
      eventAccount.totalFeeVaultValueLocked.eq(new BN(feeVaultAmount))
    );
    assert.isTrue(
      eventAccount.totalFeeVaultDeposited.eq(new BN(feeVaultAmount))
    );
    assert.isDefined(eventVaultAccount);
    assert.equal(eventVaultAccount.amount, BigInt(0));
    assert.isTrue(eventVaultAccount.mint.equals(acceptedMintPublicKey));
    assert.isDefined(feeVaultAccountInfo);
    assert.equal(feeVaultAccountInfo.lamports, feeVaultAmount);
  });

  it("should deposit and withdraw from fee vault", async () => {
    // arrange
    const depositAmount = 5000;
    const withdrawAmount = 2000;
    const eventBaseKeypair = anchor.web3.Keypair.generate();
    const [eventPublicKey] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("event", "utf-8"), eventBaseKeypair.publicKey.toBuffer()],
      program.programId
    );
    const [feeVaultPublicKey] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("fee_vault", "utf-8"), eventPublicKey.toBuffer()],
      program.programId
    );
    // act
    await program.methods
      .createEvent("fakeEvent", new BN(feeVaultRentExemption))
      .accounts({
        authority: provider.wallet.publicKey,
        eventBase: eventBaseKeypair.publicKey,
        acceptedMint: acceptedMintPublicKey,
      })
      .rpc();
    await program.methods
      .depositInFeeVault(new BN(depositAmount))
      .accounts({
        authority: provider.wallet.publicKey,
        eventBase: eventBaseKeypair.publicKey,
      })
      .rpc();
    await program.methods
      .withdrawFromFeeVault(new BN(withdrawAmount))
      .accounts({
        authority: provider.wallet.publicKey,
        eventBase: eventBaseKeypair.publicKey,
      })
      .rpc();
    // assert
    const eventAccount = await program.account.event.fetch(eventPublicKey);
    const feeVaultAccountInfo = await provider.connection.getAccountInfo(
      feeVaultPublicKey
    );
    assert.isDefined(eventAccount);
    assert.isTrue(
      eventAccount.totalFeeVaultValueLocked.eq(
        new BN(depositAmount + feeVaultRentExemption - withdrawAmount)
      )
    );
    assert.isTrue(
      eventAccount.totalFeeVaultDeposited.eq(
        new BN(depositAmount + feeVaultRentExemption)
      )
    );
    assert.isDefined(feeVaultAccountInfo);
    assert.equal(
      feeVaultAccountInfo.lamports,
      depositAmount + feeVaultRentExemption - withdrawAmount
    );
  });

  it("should fail on unauthorized withdraw from fee vault", async () => {
    // arrange
    const eventBaseKeypair = anchor.web3.Keypair.generate();
    let error: AnchorError;
    // act
    await program.methods
      .createEvent("fakeEvent", new BN(feeVaultRentExemption))
      .accounts({
        authority: provider.wallet.publicKey,
        eventBase: eventBaseKeypair.publicKey,
        acceptedMint: acceptedMintPublicKey,
      })
      .rpc();
    await program.methods
      .depositInFeeVault(new BN(5000))
      .accounts({
        authority: provider.wallet.publicKey,
        eventBase: eventBaseKeypair.publicKey,
      })
      .rpc();
    try {
      await program.methods
        .withdrawFromFeeVault(new BN(2000))
        .accounts({
          authority: aliceKeypair.publicKey,
          eventBase: eventBaseKeypair.publicKey,
        })
        .signers([aliceKeypair])
        .rpc();
    } catch (err) {
      error = err;
    }
    // assert
    assert.isDefined(error);
    assert.equal(
      error.error.errorCode.code,
      "OnlyEventAuthorityCanWithdrawFromFeeVault"
    );
  });

  it("should create and delete collaborators", async () => {
    // arrange
    const collaborator2Keypair = anchor.web3.Keypair.generate();
    const [collaborator2PublicKey] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from("collaborator", "utf-8"),
          eventPublicKey.toBuffer(),
          collaborator2Keypair.publicKey.toBuffer(),
        ],
        program.programId
      );
    // act
    await Promise.all([
      program.methods
        .createCollaborator()
        .accounts({
          authority: provider.wallet.publicKey,
          eventBase: eventBaseKeypair.publicKey,
          collaboratorBase: collaborator1Keypair.publicKey,
        })
        .rpc(),
      program.methods
        .createCollaborator()
        .accounts({
          authority: provider.wallet.publicKey,
          eventBase: eventBaseKeypair.publicKey,
          collaboratorBase: collaborator2Keypair.publicKey,
        })
        .rpc(),
    ]);
    await program.methods
      .deleteCollaborator()
      .accounts({
        authority: provider.wallet.publicKey,
        eventBase: eventBaseKeypair.publicKey,
        collaboratorBase: collaborator2Keypair.publicKey,
      })
      .rpc();
    // assert
    const collaborator1Account = await program.account.collaborator.fetch(
      collaborator1PublicKey
    );
    const collaborator2Account =
      await program.account.collaborator.fetchNullable(collaborator2PublicKey);
    assert.isDefined(collaborator1Account);
    assert.isNull(collaborator2Account);
  });

  it("should fail on unauthorized create collaborator", async () => {
    // arrange
    const eventBaseKeypair = anchor.web3.Keypair.generate();
    const collaboratorKeypair = anchor.web3.Keypair.generate();
    let error: AnchorError;
    // act
    await program.methods
      .createEvent("fakeEvent", new BN(feeVaultRentExemption))
      .accounts({
        authority: provider.wallet.publicKey,
        eventBase: eventBaseKeypair.publicKey,
        acceptedMint: acceptedMintPublicKey,
      })
      .rpc();
    try {
      await program.methods
        .createCollaborator()
        .accounts({
          authority: aliceKeypair.publicKey,
          eventBase: eventBaseKeypair.publicKey,
          collaboratorBase: collaboratorKeypair.publicKey,
        })
        .signers([aliceKeypair])
        .rpc();
    } catch (err) {
      error = err;
    }
    // assert
    assert.isDefined(error);
    assert.equal(
      error.error.errorCode.code,
      "OnlyEventAuthorityCanCreateCollaborators"
    );
  });

  it("should fail on unauthorized delete collaborator", async () => {
    // arrange
    const eventBaseKeypair = anchor.web3.Keypair.generate();
    const collaboratorKeypair = anchor.web3.Keypair.generate();
    let error: AnchorError;
    // act
    await program.methods
      .createEvent(
        "fakeEvent",
        new BN(feeVaultRentExemption + anchor.web3.LAMPORTS_PER_SOL)
      )
      .accounts({
        authority: provider.wallet.publicKey,
        eventBase: eventBaseKeypair.publicKey,
        acceptedMint: acceptedMintPublicKey,
      })
      .rpc();
    await program.methods
      .createCollaborator()
      .accounts({
        authority: provider.wallet.publicKey,
        eventBase: eventBaseKeypair.publicKey,
        collaboratorBase: collaboratorKeypair.publicKey,
      })
      .rpc();
    try {
      await program.methods
        .deleteCollaborator()
        .accounts({
          authority: aliceKeypair.publicKey,
          eventBase: eventBaseKeypair.publicKey,
          collaboratorBase: collaboratorKeypair.publicKey,
        })
        .signers([aliceKeypair])
        .rpc();
    } catch (err) {
      error = err;
    }
    // assert
    assert.isDefined(error);
    assert.equal(
      error.error.errorCode.code,
      "OnlyEventAuthorityCanDeleteCollaborators"
    );
  });

  it("should create general tickets", async () => {
    // arrange
    const ticketName = "Tomorrowland 2022 - General";
    const ticketSymbol = "TMRLND2022";
    const ticketURI = "https://www.gooogle.com";
    const ticketPrice = 5;
    const ticketQuantity = 30;
    // act
    await program.methods
      .createEventTicket(
        ticketName,
        ticketSymbol,
        ticketURI,
        ticketPrice,
        ticketQuantity
      )
      .accounts({
        authority: provider.wallet.publicKey,
        eventBase: eventBaseKeypair.publicKey,
        eventTicketBase: eventGeneralTicketBaseKeypair.publicKey,
        metadataProgram: metadataProgramPublicKey,
      })
      .rpc();
    // assert
    const eventGeneralTicketAccount = await program.account.eventTicket.fetch(
      eventGeneralTicketPublicKey
    );
    const ticketMintAccount = await getMint(
      provider.connection,
      eventGeneralTicketMintPublicKey
    );
    const metaplexNft = await metaplex
      .nfts()
      .findMintWithMetadataByAddress(eventGeneralTicketMintPublicKey)
      .run();
    assert.equal(eventGeneralTicketAccount.price, ticketPrice);
    assert.equal(eventGeneralTicketAccount.quantity, ticketQuantity);
    assert.equal(eventGeneralTicketAccount.sold, 0);
    assert.isDefined(ticketMintAccount);
    assert.equal(ticketMintAccount.decimals, 0);
    assert.equal(ticketMintAccount.supply, BigInt(0));
    assert.isDefined(metaplexNft);
    assert.equal(metaplexNft.decimals, 0);
    assert.isTrue(metaplexNft.supply.basisPoints.eq(new anchor.BN(0)));
    assert.isTrue(isMintWithMetadata(metaplexNft));
    if (isMintWithMetadata(metaplexNft)) {
      assert.equal(metaplexNft.metadata.name, ticketName);
      assert.equal(metaplexNft.metadata.symbol, ticketSymbol);
      assert.equal(metaplexNft.metadata.uri, ticketURI);
    }
  });

  it("should create vip tickets", async () => {
    // arrange
    const ticketName = "Tomorrowland 2022 - VIP";
    const ticketSymbol = "TMRLND2022";
    const ticketURI = "https://www.gooogle.com";
    const ticketPrice = 20;
    const ticketQuantity = 15;
    // act
    await program.methods
      .createEventTicket(
        ticketName,
        ticketSymbol,
        ticketURI,
        ticketPrice,
        ticketQuantity
      )
      .accounts({
        authority: provider.wallet.publicKey,
        eventBase: eventBaseKeypair.publicKey,
        eventTicketBase: eventVipTicketBaseKeypair.publicKey,
        metadataProgram: metadataProgramPublicKey,
      })
      .rpc();
    // assert
    const eventVipTicketAccount = await program.account.eventTicket.fetch(
      eventVipTicketPublicKey
    );
    const ticketMintAccount = await getMint(
      provider.connection,
      eventVipTicketMintPublicKey
    );
    const metaplexNft = await metaplex
      .nfts()
      .findMintWithMetadataByAddress(eventVipTicketMintPublicKey)
      .run();
    assert.equal(eventVipTicketAccount.price, ticketPrice);
    assert.equal(eventVipTicketAccount.quantity, ticketQuantity);
    assert.equal(eventVipTicketAccount.sold, 0);
    assert.equal(eventVipTicketAccount.used, 0);
    assert.isDefined(ticketMintAccount);
    assert.equal(ticketMintAccount.decimals, 0);
    assert.equal(ticketMintAccount.supply, BigInt(0));
    assert.isDefined(metaplexNft);
    assert.equal(metaplexNft.decimals, 0);
    assert.isTrue(metaplexNft.supply.basisPoints.eq(new anchor.BN(0)));
    assert.isTrue(isMintWithMetadata(metaplexNft));
    if (isMintWithMetadata(metaplexNft)) {
      assert.equal(metaplexNft.metadata.name, ticketName);
      assert.equal(metaplexNft.metadata.symbol, ticketSymbol);
      assert.equal(metaplexNft.metadata.uri, ticketURI);
    }
  });

  it("should buy 3 general tickets and 1 vip", async () => {
    // arrange
    const generalTicketQuantity = 3;
    const vipTicketQuantity = 1;
    const beforeAliceAccount = await getAccount(
      provider.connection,
      aliceAssociatedWalletPublicKey
    );
    const beforeEventVaultAccount = await getAccount(
      provider.connection,
      eventVaultPublicKey
    );
    const beforeGeneralTicketMintAccount = await getMint(
      provider.connection,
      eventGeneralTicketMintPublicKey
    );
    const beforeVipTicketMintAccount = await getMint(
      provider.connection,
      eventVipTicketMintPublicKey
    );
    const beforeEventGeneralTicketAccount =
      await program.account.eventTicket.fetch(eventGeneralTicketPublicKey);
    const beforeEventVipTicketAccount = await program.account.eventTicket.fetch(
      eventVipTicketPublicKey
    );
    // act
    await provider.sendAndConfirm(
      new anchor.web3.Transaction().add(
        createAssociatedTokenAccountInstruction(
          provider.wallet.publicKey,
          aliceGeneralTicketAssociatedTokenPublicKey,
          aliceKeypair.publicKey,
          eventGeneralTicketMintPublicKey
        ),
        createAssociatedTokenAccountInstruction(
          provider.wallet.publicKey,
          aliceVipTicketAssociatedTokenPublicKey,
          aliceKeypair.publicKey,
          eventVipTicketMintPublicKey
        )
      )
    );
    await Promise.all([
      program.methods
        .buyTickets(generalTicketQuantity)
        .accounts({
          authority: aliceKeypair.publicKey,
          eventBase: eventBaseKeypair.publicKey,
          eventTicketBase: eventGeneralTicketBaseKeypair.publicKey,
          buyerVault: aliceAssociatedWalletPublicKey,
          ticketVault: aliceGeneralTicketAssociatedTokenPublicKey,
        })
        .signers([aliceKeypair])
        .rpc(),
      program.methods
        .buyTickets(vipTicketQuantity)
        .accounts({
          authority: aliceKeypair.publicKey,
          eventBase: eventBaseKeypair.publicKey,
          eventTicketBase: eventVipTicketBaseKeypair.publicKey,
          buyerVault: aliceAssociatedWalletPublicKey,
          ticketVault: aliceVipTicketAssociatedTokenPublicKey,
        })
        .signers([aliceKeypair])
        .rpc(),
    ]);
    // assert
    const afterEventGeneralTicketAccount =
      await program.account.eventTicket.fetch(eventGeneralTicketPublicKey);
    const afterEventVipTicketAccount = await program.account.eventTicket.fetch(
      eventVipTicketPublicKey
    );
    const afterAliceAccount = await getAccount(
      provider.connection,
      aliceAssociatedWalletPublicKey
    );
    const afterEventVaultAccount = await getAccount(
      provider.connection,
      eventVaultPublicKey
    );
    const afterGeneralTicketMintAccount = await getMint(
      provider.connection,
      eventGeneralTicketMintPublicKey
    );
    const afterVipTicketMintAccount = await getMint(
      provider.connection,
      eventVipTicketMintPublicKey
    );
    const aliceGeneralTicketVaultAccount = await getAccount(
      provider.connection,
      aliceGeneralTicketAssociatedTokenPublicKey
    );
    const aliceVipTicketVaultAccount = await getAccount(
      provider.connection,
      aliceVipTicketAssociatedTokenPublicKey
    );

    // Assert alice vault changed
    assert.isDefined(beforeAliceAccount);
    assert.isDefined(afterAliceAccount);
    assert.equal(
      afterAliceAccount.amount,
      beforeAliceAccount.amount -
        (BigInt(generalTicketQuantity * afterEventGeneralTicketAccount.price) +
          BigInt(vipTicketQuantity * afterEventVipTicketAccount.price))
    );

    // Assert event vault changed
    assert.isDefined(beforeEventVaultAccount);
    assert.isDefined(afterEventVaultAccount);
    assert.equal(
      afterEventVaultAccount.amount,
      beforeEventVaultAccount.amount +
        (BigInt(generalTicketQuantity * afterEventGeneralTicketAccount.price) +
          BigInt(vipTicketQuantity * afterEventVipTicketAccount.price))
    );

    // Assert general ticket values changed
    assert.isDefined(aliceGeneralTicketVaultAccount);
    assert.equal(
      aliceGeneralTicketVaultAccount.amount,
      BigInt(generalTicketQuantity)
    );
    assert.isDefined(beforeGeneralTicketMintAccount);
    assert.isDefined(afterGeneralTicketMintAccount);
    assert.equal(
      afterGeneralTicketMintAccount.supply,
      beforeGeneralTicketMintAccount.supply + BigInt(generalTicketQuantity)
    );
    assert.equal(
      afterEventGeneralTicketAccount.sold,
      beforeEventGeneralTicketAccount.sold + generalTicketQuantity
    );

    // Assert VIP ticket values changed
    assert.isDefined(aliceVipTicketVaultAccount);
    assert.equal(aliceVipTicketVaultAccount.amount, BigInt(vipTicketQuantity));
    assert.isDefined(beforeVipTicketMintAccount);
    assert.isDefined(afterVipTicketMintAccount);
    assert.equal(
      afterVipTicketMintAccount.supply,
      beforeVipTicketMintAccount.supply + BigInt(vipTicketQuantity)
    );
    assert.equal(
      afterEventVipTicketAccount.sold,
      beforeEventVipTicketAccount.sold + vipTicketQuantity
    );
  });

  it("should check-in 2 general tickets", async () => {
    // arrange
    const generalTicketQuantity = 2;
    const beforeEventGeneralTicketAccount =
      await program.account.eventTicket.fetch(eventGeneralTicketPublicKey);
    const beforeAliceGeneralTicketVaultAccount = await getAccount(
      provider.connection,
      aliceGeneralTicketAssociatedTokenPublicKey
    );
    // act
    await program.methods
      .checkIn(generalTicketQuantity)
      .accounts({
        attendee: aliceKeypair.publicKey,
        collaboratorBase: collaborator1Keypair.publicKey,
        eventBase: eventBaseKeypair.publicKey,
        eventTicketBase: eventGeneralTicketBaseKeypair.publicKey,
        ticketVault: aliceGeneralTicketAssociatedTokenPublicKey,
      })
      .signers([aliceKeypair, collaborator1Keypair])
      .rpc();
    // assert
    const afterEventGeneralTicketAccount =
      await program.account.eventTicket.fetch(eventGeneralTicketPublicKey);
    const afterAliceGeneralTicketVaultAccount = await getAccount(
      provider.connection,
      aliceGeneralTicketAssociatedTokenPublicKey
    );
    assert.isDefined(beforeEventGeneralTicketAccount);
    assert.isDefined(afterEventGeneralTicketAccount);
    assert.equal(
      beforeEventGeneralTicketAccount.used + generalTicketQuantity,
      afterEventGeneralTicketAccount.used
    );
    assert.isDefined(beforeAliceGeneralTicketVaultAccount);
    assert.isDefined(afterAliceGeneralTicketVaultAccount);
    assert.equal(
      beforeAliceGeneralTicketVaultAccount.amount,
      afterAliceGeneralTicketVaultAccount.amount + BigInt(generalTicketQuantity)
    );
  });

  it("should fail when there are not enough tickets available", async () => {
    // arrange
    let error: AnchorError;
    const ticketName = "Tomorrowland 2022 - Ultra VIP";
    const ticketSymbol = "TMRLND2022";
    const ticketURI = "https://www.gooogle.com";
    const ticketPrice = 5;
    const ticketQuantity = 1;
    const ticketToBuy = 2;
    const eventUltraVipTicketBaseKeypair = anchor.web3.Keypair.generate();
    const [eventUltraVipTicketPublicKey] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from("event_ticket", "utf-8"),
          eventPublicKey.toBuffer(),
          eventUltraVipTicketBaseKeypair.publicKey.toBuffer(),
        ],
        program.programId
      );
    const [eventUltraVipTicketMintPublicKey] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from("ticket_mint", "utf-8"),
          eventPublicKey.toBuffer(),
          eventUltraVipTicketPublicKey.toBuffer(),
        ],
        program.programId
      );
    const aliceUltraVipTicketAssociatedTokenPublicKey =
      await getAssociatedTokenAddress(
        eventUltraVipTicketMintPublicKey,
        aliceKeypair.publicKey
      );
    // act
    try {
      await program.methods
        .buyTickets(ticketToBuy)
        .accounts({
          authority: aliceKeypair.publicKey,
          eventBase: eventBaseKeypair.publicKey,
          eventTicketBase: eventUltraVipTicketBaseKeypair.publicKey,
          buyerVault: aliceAssociatedWalletPublicKey,
          ticketVault: aliceUltraVipTicketAssociatedTokenPublicKey,
        })
        .preInstructions([
          await program.methods
            .createEventTicket(
              ticketName,
              ticketSymbol,
              ticketURI,
              ticketPrice,
              ticketQuantity
            )
            .accounts({
              authority: provider.wallet.publicKey,
              eventBase: eventBaseKeypair.publicKey,
              eventTicketBase: eventUltraVipTicketBaseKeypair.publicKey,
              metadataProgram: metadataProgramPublicKey,
            })
            .instruction(),
          createAssociatedTokenAccountInstruction(
            provider.wallet.publicKey,
            aliceUltraVipTicketAssociatedTokenPublicKey,
            aliceKeypair.publicKey,
            eventUltraVipTicketMintPublicKey
          ),
        ])
        .signers([aliceKeypair])
        .rpc();
    } catch (err) {
      error = err;
    }
    // assert
    assert.isDefined(error);
    assert.equal(error.error.errorCode.code, "NotEnoughTicketsAvailable");
  });
});
