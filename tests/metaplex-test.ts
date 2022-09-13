import {
  bundlrStorage, Metaplex,
  walletAdapterIdentity
} from "@metaplex-foundation/js";
import * as anchor from "@project-serum/anchor";
import { AnchorError, Program } from "@project-serum/anchor";
import {
  getAccount,
  getAssociatedTokenAddress
} from "@solana/spl-token";
import { BN } from "bn.js";
import { assert } from "chai";
import { Disco } from "../target/types/disco";
import {
  createFundedWallet,
  createMint,
  createNftWithVerifiedCollection,
  createUserAndAssociatedWallet
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
  const generalTicket1Keypair = anchor.web3.Keypair.generate();
  const metadataProgramPublicKey = new anchor.web3.PublicKey(
    "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
  );
  const metaplex = Metaplex.make(provider.connection)
    .use(walletAdapterIdentity(provider.wallet))
    .use(bundlrStorage({ address: "https://devnet.bundlr.network" }));
  const aliceBalance = 5000;
  let aliceKeypair: anchor.web3.Keypair;
  let aliceAssociatedWalletPublicKey: anchor.web3.PublicKey;
  let eventPublicKey: anchor.web3.PublicKey;
  let eventVaultPublicKey: anchor.web3.PublicKey;
  let aliceGeneralTicketAssociatedTokenPublicKey: anchor.web3.PublicKey;
  let aliceVipTicketAssociatedTokenPublicKey: anchor.web3.PublicKey;
  let acceptedMintPublicKey: anchor.web3.PublicKey;
  let eventGeneralTicketPublicKey: anchor.web3.PublicKey;
  let eventVipTicketPublicKey: anchor.web3.PublicKey;
  let eventGeneralTicketMintPublicKey: anchor.web3.PublicKey;
  let eventVipTicketMintPublicKey: anchor.web3.PublicKey;
  let vipAttendanceMintPublicKey: anchor.web3.PublicKey;
  let collaborator1PublicKey: anchor.web3.PublicKey;
  let aliceVipAttendanceAssociatedTokenPublicKey: anchor.web3.PublicKey;
  let collectionPublicKey: anchor.web3.PublicKey;
  let nftWithVerifiedCollectionPublicKey: anchor.web3.PublicKey;

  let eventMintPublicKey: anchor.web3.PublicKey;
  let eventMetadataPublicKey: anchor.web3.PublicKey;

  before(async () => {
    [eventPublicKey] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("event", "utf-8"), eventBaseKeypair.publicKey.toBuffer()],
      program.programId
    );
    [eventMintPublicKey] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("event_mint", "utf-8"), eventPublicKey.toBuffer()],
      program.programId
    );
    [eventMetadataPublicKey] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("metadata", "utf-8"),
        metadataProgramPublicKey.toBuffer(),
        eventMintPublicKey.toBuffer(),
      ],
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
    [vipAttendanceMintPublicKey] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from("attendance_mint", "utf-8"),
          eventPublicKey.toBuffer(),
          eventVipTicketPublicKey.toBuffer(),
        ],
        program.programId
      );
    [eventVaultPublicKey] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("event_vault", "utf-8"), eventPublicKey.toBuffer()],
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
    aliceVipAttendanceAssociatedTokenPublicKey =
      await getAssociatedTokenAddress(
        vipAttendanceMintPublicKey,
        aliceKeypair.publicKey
      );
    [collectionPublicKey, nftWithVerifiedCollectionPublicKey] =
      await createNftWithVerifiedCollection(metaplex, provider);
  });

  it("should create Tomorrowland 2022 event", async () => {
    // arrange
    const eventName = "Tomorrowland 2022";
    const eventSymbol = "TMRL2022";
    const eventUri = "www.google.com";
    // act
    await program.methods
      .createEvent(eventName, eventSymbol, eventUri)
      .accounts({
        authority: provider.wallet.publicKey,
        eventBase: eventBaseKeypair.publicKey,
        acceptedMint: acceptedMintPublicKey,
        metadataProgram: metadataProgramPublicKey,
      })
      .rpc();
    // assert
    const eventAccount = await program.account.event.fetch(eventPublicKey);
    const eventVaultAccount = await getAccount(
      provider.connection,
      eventVaultPublicKey
    );
    const eventCollectionNftAccount = await metaplex
      .nfts()
      .findByMint(eventMintPublicKey)
      .run();
    assert.isDefined(eventAccount);
    assert.isTrue(eventAccount.authority.equals(provider.wallet.publicKey));
    assert.isDefined(eventVaultAccount);
    assert.equal(eventVaultAccount.amount, BigInt(0));
    assert.isTrue(eventVaultAccount.mint.equals(acceptedMintPublicKey));
    assert.isDefined(eventCollectionNftAccount);
    assert.equal(eventCollectionNftAccount.name, eventName);
    assert.equal(eventCollectionNftAccount.symbol, eventSymbol);
    assert.equal(eventCollectionNftAccount.uri, eventUri);
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
      .createEvent("fakeEvent", "FAKE", "news.com")
      .accounts({
        authority: provider.wallet.publicKey,
        eventBase: eventBaseKeypair.publicKey,
        acceptedMint: acceptedMintPublicKey,
        metadataProgram: metadataProgramPublicKey,
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
      .createEvent("fakeEvent", "FAKE", "news.com")
      .accounts({
        authority: provider.wallet.publicKey,
        eventBase: eventBaseKeypair.publicKey,
        acceptedMint: acceptedMintPublicKey,
        metadataProgram: metadataProgramPublicKey,
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
        new BN(ticketPrice),
        new BN(ticketQuantity)
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
    assert.isTrue(eventGeneralTicketAccount.sold.eq(new anchor.BN(0)));
    assert.isTrue(eventGeneralTicketAccount.used.eq(new anchor.BN(0)));
    assert.isTrue(
      eventGeneralTicketAccount.price.eq(new anchor.BN(ticketPrice))
    );
    assert.isTrue(
      eventGeneralTicketAccount.quantity.eq(new anchor.BN(ticketQuantity))
    );
  });

  it("should create vip tickets with proof of attendance", async () => {
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
        new BN(ticketPrice),
        new BN(ticketQuantity)
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
    assert.isDefined(eventVipTicketAccount);
    assert.isTrue(eventVipTicketAccount.sold.eq(new anchor.BN(0)));
    assert.isTrue(eventVipTicketAccount.used.eq(new anchor.BN(0)));
    assert.isTrue(eventVipTicketAccount.price.eq(new anchor.BN(ticketPrice)));
    assert.isTrue(
      eventVipTicketAccount.quantity.eq(new anchor.BN(ticketQuantity))
    );
  });

  it("should buy 3 general ticket and 2 vip ticket", async () => {
    // arrange
    const generalTicketQuantity = 3;
    const vipTicketQuantity = 2;
    const [generalTicket1MintPublicKey] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from("ticket_mint", "utf-8"),
          eventPublicKey.toBuffer(),
          eventGeneralTicketPublicKey.toBuffer(),
          generalTicket1Keypair.publicKey.toBuffer(),
        ],
        program.programId
      );
    const aliceGeneralTicket1AssociatedTokenPublicKey =
      await getAssociatedTokenAddress(
        generalTicket1MintPublicKey,
        aliceKeypair.publicKey
      );
    const generalTicket2Keypair = anchor.web3.Keypair.generate();
    const [generalTicket2MintPublicKey] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from("ticket_mint", "utf-8"),
          eventPublicKey.toBuffer(),
          eventGeneralTicketPublicKey.toBuffer(),
          generalTicket2Keypair.publicKey.toBuffer(),
        ],
        program.programId
      );
    const aliceGeneralTicket2AssociatedTokenPublicKey =
      await getAssociatedTokenAddress(
        generalTicket2MintPublicKey,
        aliceKeypair.publicKey
      );
    const generalTicket3Keypair = anchor.web3.Keypair.generate();
    const [generalTicket3MintPublicKey] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from("ticket_mint", "utf-8"),
          eventPublicKey.toBuffer(),
          eventGeneralTicketPublicKey.toBuffer(),
          generalTicket3Keypair.publicKey.toBuffer(),
        ],
        program.programId
      );
    const aliceGeneralTicket3AssociatedTokenPublicKey =
      await getAssociatedTokenAddress(
        generalTicket3MintPublicKey,
        aliceKeypair.publicKey
      );
    const vipTicket1Keypair = anchor.web3.Keypair.generate();
    const [vipTicket1MintPublicKey] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from("ticket_mint", "utf-8"),
          eventPublicKey.toBuffer(),
          eventVipTicketPublicKey.toBuffer(),
          vipTicket1Keypair.publicKey.toBuffer(),
        ],
        program.programId
      );
    const aliceVipTicket1AssociatedTokenPublicKey =
      await getAssociatedTokenAddress(
        vipTicket1MintPublicKey,
        aliceKeypair.publicKey
      );
    const vipTicket2Keypair = anchor.web3.Keypair.generate();
    const [vipTicket2MintPublicKey] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from("ticket_mint", "utf-8"),
          eventPublicKey.toBuffer(),
          eventVipTicketPublicKey.toBuffer(),
          vipTicket2Keypair.publicKey.toBuffer(),
        ],
        program.programId
      );
    const aliceVipTicket2AssociatedTokenPublicKey =
      await getAssociatedTokenAddress(
        vipTicket2MintPublicKey,
        aliceKeypair.publicKey
      );
    const beforeAliceAccount = await getAccount(
      provider.connection,
      aliceAssociatedWalletPublicKey
    );
    const beforeEventVaultAccount = await getAccount(
      provider.connection,
      eventVaultPublicKey
    );
    // act
    await Promise.all([
      program.methods
        .buyTickets()
        .accounts({
          authority: aliceKeypair.publicKey,
          eventBase: eventBaseKeypair.publicKey,
          eventTicketBase: eventGeneralTicketBaseKeypair.publicKey,
          buyerVault: aliceAssociatedWalletPublicKey,
          ticketMintBase: generalTicket1Keypair.publicKey,
          ticketVault: aliceGeneralTicket1AssociatedTokenPublicKey,
          metadataProgram: metadataProgramPublicKey,
        })
        .signers([aliceKeypair])
        .rpc(),
      program.methods
        .buyTickets()
        .accounts({
          authority: aliceKeypair.publicKey,
          eventBase: eventBaseKeypair.publicKey,
          eventTicketBase: eventGeneralTicketBaseKeypair.publicKey,
          buyerVault: aliceAssociatedWalletPublicKey,
          ticketMintBase: generalTicket2Keypair.publicKey,
          ticketVault: aliceGeneralTicket2AssociatedTokenPublicKey,
          metadataProgram: metadataProgramPublicKey,
        })
        .signers([aliceKeypair])
        .rpc(),
      program.methods
        .buyTickets()
        .accounts({
          authority: aliceKeypair.publicKey,
          eventBase: eventBaseKeypair.publicKey,
          eventTicketBase: eventGeneralTicketBaseKeypair.publicKey,
          buyerVault: aliceAssociatedWalletPublicKey,
          ticketMintBase: generalTicket3Keypair.publicKey,
          ticketVault: aliceGeneralTicket3AssociatedTokenPublicKey,
          metadataProgram: metadataProgramPublicKey,
        })
        .signers([aliceKeypair])
        .rpc(),
      program.methods
        .buyTickets()
        .accounts({
          authority: aliceKeypair.publicKey,
          eventBase: eventBaseKeypair.publicKey,
          eventTicketBase: eventVipTicketBaseKeypair.publicKey,
          buyerVault: aliceAssociatedWalletPublicKey,
          ticketMintBase: vipTicket1Keypair.publicKey,
          ticketVault: aliceVipTicket1AssociatedTokenPublicKey,
          metadataProgram: metadataProgramPublicKey,
        })
        .signers([aliceKeypair])
        .rpc(),
      await program.methods
        .buyTickets()
        .accounts({
          authority: aliceKeypair.publicKey,
          eventBase: eventBaseKeypair.publicKey,
          eventTicketBase: eventVipTicketBaseKeypair.publicKey,
          buyerVault: aliceAssociatedWalletPublicKey,
          ticketMintBase: vipTicket2Keypair.publicKey,
          ticketVault: aliceVipTicket2AssociatedTokenPublicKey,
          metadataProgram: metadataProgramPublicKey,
        })
        .signers([aliceKeypair])
        .rpc(),
    ]);
    // assert
    const eventGeneralTicket1NftAccount = await metaplex
      .nfts()
      .findByMint(generalTicket1MintPublicKey)
      .run();
    const eventGeneralTicket2NftAccount = await metaplex
      .nfts()
      .findByMint(generalTicket2MintPublicKey)
      .run();
    const eventGeneralTicket3NftAccount = await metaplex
      .nfts()
      .findByMint(generalTicket3MintPublicKey)
      .run();
    const eventVipTicket1NftAccount = await metaplex
      .nfts()
      .findByMint(vipTicket1MintPublicKey)
      .run();
    const eventVipTicket2NftAccount = await metaplex
      .nfts()
      .findByMint(vipTicket2MintPublicKey)
      .run();

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
    const aliceGeneralTicket1VaultAccount = await getAccount(
      provider.connection,
      aliceGeneralTicket1AssociatedTokenPublicKey
    );
    const aliceGeneralTicket2VaultAccount = await getAccount(
      provider.connection,
      aliceGeneralTicket2AssociatedTokenPublicKey
    );
    const aliceGeneralTicket3VaultAccount = await getAccount(
      provider.connection,
      aliceGeneralTicket3AssociatedTokenPublicKey
    );
    const aliceVipTicket1VaultAccount = await getAccount(
      provider.connection,
      aliceVipTicket1AssociatedTokenPublicKey
    );
    const aliceVipTicket2VaultAccount = await getAccount(
      provider.connection,
      aliceVipTicket2AssociatedTokenPublicKey
    );

    // Tickets sold are updated
    assert.isTrue(
      afterEventGeneralTicketAccount.sold.eq(new BN(generalTicketQuantity))
    );
    assert.isTrue(
      afterEventVipTicketAccount.sold.eq(new BN(vipTicketQuantity))
    );

    // Nft was created
    assert.isDefined(eventGeneralTicket1NftAccount);
    assert.equal(
      eventGeneralTicket1NftAccount.name,
      afterEventGeneralTicketAccount.name
    );
    assert.equal(
      eventGeneralTicket1NftAccount.symbol,
      afterEventGeneralTicketAccount.symbol
    );
    assert.equal(
      eventGeneralTicket1NftAccount.uri,
      afterEventGeneralTicketAccount.uri
    );
    assert.equal(eventGeneralTicket1NftAccount.uses.useMethod, 2);
    assert.isTrue(eventGeneralTicket1NftAccount.uses.remaining.eq(new BN(1)));
    assert.isTrue(eventGeneralTicket1NftAccount.uses.total.eq(new BN(1)));
    assert.isTrue(eventGeneralTicket1NftAccount.collection.verified);
    assert.isTrue(
      eventGeneralTicket1NftAccount.collection.key.equals(eventMintPublicKey)
    );
    assert.isDefined(eventGeneralTicket2NftAccount);
    assert.equal(
      eventGeneralTicket2NftAccount.name,
      afterEventGeneralTicketAccount.name
    );
    assert.equal(
      eventGeneralTicket2NftAccount.symbol,
      afterEventGeneralTicketAccount.symbol
    );
    assert.equal(
      eventGeneralTicket2NftAccount.uri,
      afterEventGeneralTicketAccount.uri
    );
    assert.equal(eventGeneralTicket2NftAccount.uses.useMethod, 2);
    assert.isTrue(eventGeneralTicket2NftAccount.uses.remaining.eq(new BN(1)));
    assert.isTrue(eventGeneralTicket2NftAccount.uses.total.eq(new BN(1)));
    assert.isTrue(eventGeneralTicket2NftAccount.collection.verified);
    assert.isTrue(
      eventGeneralTicket2NftAccount.collection.key.equals(eventMintPublicKey)
    );
    assert.isDefined(eventGeneralTicket3NftAccount);
    assert.equal(
      eventGeneralTicket3NftAccount.name,
      afterEventGeneralTicketAccount.name
    );
    assert.equal(
      eventGeneralTicket3NftAccount.symbol,
      afterEventGeneralTicketAccount.symbol
    );
    assert.equal(
      eventGeneralTicket3NftAccount.uri,
      afterEventGeneralTicketAccount.uri
    );
    assert.equal(eventGeneralTicket3NftAccount.uses.useMethod, 2);
    assert.isTrue(eventGeneralTicket3NftAccount.uses.remaining.eq(new BN(1)));
    assert.isTrue(eventGeneralTicket3NftAccount.uses.total.eq(new BN(1)));
    assert.isTrue(eventGeneralTicket3NftAccount.collection.verified);
    assert.isTrue(
      eventGeneralTicket3NftAccount.collection.key.equals(eventMintPublicKey)
    );
    assert.isDefined(eventVipTicket1NftAccount);
    assert.equal(
      eventVipTicket1NftAccount.name,
      afterEventVipTicketAccount.name
    );
    assert.equal(
      eventVipTicket1NftAccount.symbol,
      afterEventVipTicketAccount.symbol
    );
    assert.equal(eventVipTicket1NftAccount.uri, afterEventVipTicketAccount.uri);
    assert.equal(eventVipTicket1NftAccount.uses.useMethod, 2);
    assert.isTrue(eventVipTicket1NftAccount.uses.remaining.eq(new BN(1)));
    assert.isTrue(eventVipTicket1NftAccount.uses.total.eq(new BN(1)));
    assert.isTrue(eventVipTicket1NftAccount.collection.verified);
    assert.isTrue(
      eventVipTicket1NftAccount.collection.key.equals(eventMintPublicKey)
    );
    assert.isDefined(eventVipTicket2NftAccount);
    assert.equal(
      eventVipTicket2NftAccount.name,
      afterEventVipTicketAccount.name
    );
    assert.equal(
      eventVipTicket2NftAccount.symbol,
      afterEventVipTicketAccount.symbol
    );
    assert.equal(eventVipTicket2NftAccount.uri, afterEventVipTicketAccount.uri);
    assert.equal(eventVipTicket2NftAccount.uses.useMethod, 2);
    assert.isTrue(eventVipTicket2NftAccount.uses.remaining.eq(new BN(1)));
    assert.isTrue(eventVipTicket2NftAccount.uses.total.eq(new BN(1)));
    assert.isTrue(eventVipTicket2NftAccount.collection.verified);
    assert.isTrue(
      eventVipTicket2NftAccount.collection.key.equals(eventMintPublicKey)
    );

    // Alice account changed
    assert.isDefined(beforeAliceAccount);
    assert.isDefined(afterAliceAccount);
    assert.isTrue(
      new BN(afterAliceAccount.amount.toString()).eq(
        new BN(beforeAliceAccount.amount.toString()).sub(
          afterEventGeneralTicketAccount.price
            .mul(new BN(generalTicketQuantity))
            .add(
              afterEventVipTicketAccount.price.mul(new BN(vipTicketQuantity))
            )
        )
      )
    );

    // Assert event vault changed
    assert.isDefined(beforeEventVaultAccount);
    assert.isDefined(afterEventVaultAccount);
    assert.isTrue(
      new BN(afterEventVaultAccount.amount.toString()).eq(
        new BN(beforeEventVaultAccount.amount.toString()).add(
          afterEventGeneralTicketAccount.price
            .mul(new BN(generalTicketQuantity))
            .add(
              afterEventVipTicketAccount.price.mul(new BN(vipTicketQuantity))
            )
        )
      )
    );

    // Assert general ticket values changed
    assert.isDefined(aliceGeneralTicket1VaultAccount);
    assert.equal(aliceGeneralTicket1VaultAccount.amount, BigInt(1));
    assert.isDefined(aliceGeneralTicket2VaultAccount);
    assert.equal(aliceGeneralTicket2VaultAccount.amount, BigInt(1));
    assert.isDefined(aliceGeneralTicket3VaultAccount);
    assert.equal(aliceGeneralTicket3VaultAccount.amount, BigInt(1));
    assert.isDefined(aliceVipTicket1VaultAccount);
    assert.equal(aliceVipTicket1VaultAccount.amount, BigInt(1));
    assert.isDefined(aliceVipTicket2VaultAccount);
    assert.equal(aliceVipTicket2VaultAccount.amount, BigInt(1));
  });

  it("should check-in 1 general ticket", async () => {
    // arrange
    const [generalTicket1MintPublicKey] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from("ticket_mint", "utf-8"),
          eventPublicKey.toBuffer(),
          eventGeneralTicketPublicKey.toBuffer(),
          generalTicket1Keypair.publicKey.toBuffer(),
        ],
        program.programId
      );
    const aliceGeneralTicket1AssociatedTokenPublicKey =
      await getAssociatedTokenAddress(
        generalTicket1MintPublicKey,
        aliceKeypair.publicKey
      );

    // act
    await program.methods
      .checkIn()
      .accounts({
        authority: aliceKeypair.publicKey,
        collaboratorBase: collaborator1Keypair.publicKey,
        eventBase: eventBaseKeypair.publicKey,
        eventTicketBase: eventGeneralTicketBaseKeypair.publicKey,
        ticketMintBase: generalTicket1Keypair.publicKey,
        ticketVault: aliceGeneralTicket1AssociatedTokenPublicKey,
        metadataProgram: metadataProgramPublicKey,
      })
      .signers([aliceKeypair, collaborator1Keypair])
      .rpc();
    // assert
    const eventGeneralTicket1NftAccount = await metaplex
      .nfts()
      .findByMint(generalTicket1MintPublicKey)
      .run();
    assert.isTrue(eventGeneralTicket1NftAccount.uses.remaining.eq(new BN(0)));
  });

  it("should fail when there are not enough tickets available", async () => {
    // arrange
    let error: AnchorError;
    const ticketName = "Tomorrowland 2022 - Ultra VIP";
    const ticketSymbol = "TMRLND2022";
    const ticketURI = "https://www.gooogle.com";
    const ticketPrice = 5;
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
    const ultraVipTicket1Keypair = anchor.web3.Keypair.generate();
    const [ultraVipTicket1MintPublicKey] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from("ticket_mint", "utf-8"),
          eventPublicKey.toBuffer(),
          eventUltraVipTicketPublicKey.toBuffer(),
          ultraVipTicket1Keypair.publicKey.toBuffer(),
        ],
        program.programId
      );
    const aliceUltraVipTicket1AssociatedTokenPublicKey =
      await getAssociatedTokenAddress(
        ultraVipTicket1MintPublicKey,
        aliceKeypair.publicKey
      );
    // act
    try {
      await program.methods
        .buyTickets()
        .accounts({
          authority: aliceKeypair.publicKey,
          eventBase: eventBaseKeypair.publicKey,
          eventTicketBase: eventUltraVipTicketBaseKeypair.publicKey,
          buyerVault: aliceAssociatedWalletPublicKey,
          ticketMintBase: ultraVipTicket1Keypair.publicKey,
          ticketVault: aliceUltraVipTicket1AssociatedTokenPublicKey,
          metadataProgram: metadataProgramPublicKey,
        })
        .preInstructions([
          await program.methods
            .createEventTicket(
              ticketName,
              ticketSymbol,
              ticketURI,
              new BN(ticketPrice),
              new BN(0)
            )
            .accounts({
              authority: provider.wallet.publicKey,
              eventBase: eventBaseKeypair.publicKey,
              eventTicketBase: eventUltraVipTicketBaseKeypair.publicKey,
              metadataProgram: metadataProgramPublicKey,
            })
            .instruction(),
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

  it("should fail when the ticket was already used", async () => {
    // arrange
    let error: AnchorError;
    const ticketName = "Tomorrowland 2022 - Ultra VIP";
    const ticketSymbol = "TMRLND2022";
    const ticketURI = "https://www.gooogle.com";
    const ticketPrice = 5;
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
    const ultraVipTicket1Keypair = anchor.web3.Keypair.generate();
    const [ultraVipTicket1MintPublicKey] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from("ticket_mint", "utf-8"),
          eventPublicKey.toBuffer(),
          eventUltraVipTicketPublicKey.toBuffer(),
          ultraVipTicket1Keypair.publicKey.toBuffer(),
        ],
        program.programId
      );
    const aliceUltraVipTicket1AssociatedTokenPublicKey =
      await getAssociatedTokenAddress(
        ultraVipTicket1MintPublicKey,
        aliceKeypair.publicKey
      );
    // act
    try {
      await program.methods
        .checkIn()
        .accounts({
          authority: aliceKeypair.publicKey,
          collaboratorBase: collaborator1Keypair.publicKey,
          eventBase: eventBaseKeypair.publicKey,
          eventTicketBase: eventUltraVipTicketBaseKeypair.publicKey,
          ticketMintBase: ultraVipTicket1Keypair.publicKey,
          ticketVault: aliceUltraVipTicket1AssociatedTokenPublicKey,
          metadataProgram: metadataProgramPublicKey,
        })
        .preInstructions([
          await program.methods
            .createEventTicket(
              ticketName,
              ticketSymbol,
              ticketURI,
              new BN(ticketPrice),
              new BN(5)
            )
            .accounts({
              authority: provider.wallet.publicKey,
              eventBase: eventBaseKeypair.publicKey,
              eventTicketBase: eventUltraVipTicketBaseKeypair.publicKey,
              metadataProgram: metadataProgramPublicKey,
            })
            .instruction(),
          await program.methods
            .buyTickets()
            .accounts({
              authority: aliceKeypair.publicKey,
              eventBase: eventBaseKeypair.publicKey,
              eventTicketBase: eventUltraVipTicketBaseKeypair.publicKey,
              buyerVault: aliceAssociatedWalletPublicKey,
              ticketMintBase: ultraVipTicket1Keypair.publicKey,
              ticketVault: aliceUltraVipTicket1AssociatedTokenPublicKey,
              metadataProgram: metadataProgramPublicKey,
            })
            .instruction(),
          await program.methods
            .checkIn()
            .accounts({
              authority: aliceKeypair.publicKey,
              collaboratorBase: collaborator1Keypair.publicKey,
              eventBase: eventBaseKeypair.publicKey,
              eventTicketBase: eventUltraVipTicketBaseKeypair.publicKey,
              ticketMintBase: ultraVipTicket1Keypair.publicKey,
              ticketVault: aliceUltraVipTicket1AssociatedTokenPublicKey,
              metadataProgram: metadataProgramPublicKey,
            })
            .instruction(),
        ])
        .signers([aliceKeypair, collaborator1Keypair])
        .rpc();
    } catch (err) {
      error = err;
    }
    // assert
    assert.isDefined(error);
    assert.equal(error.error.errorCode.code, "TicketAlreadyCheckedIn");
  });
});
