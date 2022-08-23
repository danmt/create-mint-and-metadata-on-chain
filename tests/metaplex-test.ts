import {
  bundlrStorage,
  isMintWithMetadata,
  Metaplex,
  walletAdapterIdentity,
} from "@metaplex-foundation/js";
import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { getAccount, getMint } from "@solana/spl-token";
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
  const metadataProgramPublicKey = new anchor.web3.PublicKey(
    "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
  );
  const metaplex = Metaplex.make(provider.connection)
    .use(walletAdapterIdentity(provider.wallet))
    .use(bundlrStorage());
  const aliceBalance = 5000;
  let aliceKeypair: anchor.web3.Keypair;
  let aliceAssociatedWalletPublicKey: anchor.web3.PublicKey;
  let eventPublicKey: anchor.web3.PublicKey;
  let eventVaultPublicKey: anchor.web3.PublicKey;
  let aliceTicketReceiverPublicKey: anchor.web3.PublicKey;
  let ticketMintPublicKey: anchor.web3.PublicKey;
  let metadataPublicKey: anchor.web3.PublicKey;
  let acceptedMintPublicKey: anchor.web3.PublicKey;

  before(async () => {
    [eventPublicKey] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("event", "utf-8"), eventBaseKeypair.publicKey.toBuffer()],
      program.programId
    );
    [ticketMintPublicKey] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("ticket_mint", "utf-8"), eventPublicKey.toBuffer()],
      program.programId
    );
    [eventVaultPublicKey] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("event_vault", "utf-8"), eventPublicKey.toBuffer()],
      program.programId
    );
    [metadataPublicKey] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("metadata", "utf-8"),
        metadataProgramPublicKey.toBuffer(),
        ticketMintPublicKey.toBuffer(),
      ],
      metadataProgramPublicKey
    );

    aliceKeypair = await createFundedWallet(provider);
    acceptedMintPublicKey = await createMint(provider);
    aliceAssociatedWalletPublicKey = await createUserAndAssociatedWallet(
      provider,
      acceptedMintPublicKey,
      aliceBalance,
      aliceKeypair
    );
    [aliceTicketReceiverPublicKey] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from("ticket_receiver", "utf-8"),
          aliceKeypair.publicKey.toBuffer(),
          ticketMintPublicKey.toBuffer(),
        ],
        program.programId
      );
  });

  it("should create an event", async () => {
    // arrange
    const ticketTitle = "Tomorrowland 2022";
    const ticketSymbol = "TMRLND2022";
    const ticketURI = "https://www.gooogle.com";
    const ticketPrice = 5;
    // act
    await program.methods
      .createEvent(ticketTitle, ticketSymbol, ticketURI, ticketPrice)
      .accounts({
        authority: provider.wallet.publicKey,
        eventBase: eventBaseKeypair.publicKey,
        acceptedMint: acceptedMintPublicKey,
        metadataProgram: metadataProgramPublicKey,
        ticketMetadata: metadataPublicKey,
      })
      .rpc();
    // assert
    const eventAccount = await program.account.event.fetch(eventPublicKey);
    const ticketMintAccount = await getMint(
      provider.connection,
      ticketMintPublicKey
    );
    const eventVaultAccount = await getAccount(
      provider.connection,
      eventVaultPublicKey
    );
    const metaplexNft = await metaplex
      .nfts()
      .findMintWithMetadataByAddress(ticketMintPublicKey)
      .run();
    assert.isDefined(eventAccount);
    assert.equal(eventAccount.ticketTitle, ticketTitle);
    assert.equal(eventAccount.ticketSymbol, ticketSymbol);
    assert.equal(eventAccount.ticketUri, ticketURI);
    assert.isDefined(ticketMintAccount);
    assert.equal(ticketMintAccount.decimals, 0);
    assert.equal(ticketMintAccount.supply, BigInt(0));
    assert.isDefined(metaplexNft);
    assert.equal(metaplexNft.decimals, 0);
    assert.isTrue(metaplexNft.supply.basisPoints.eq(new anchor.BN(0)));
    assert.isTrue(isMintWithMetadata(metaplexNft));
    if (isMintWithMetadata(metaplexNft)) {
      assert.equal(metaplexNft.metadata.name, ticketTitle);
      assert.equal(metaplexNft.metadata.symbol, ticketSymbol);
      assert.equal(metaplexNft.metadata.uri, ticketURI);
    }
    assert.isDefined(eventVaultAccount);
    assert.equal(eventVaultAccount.amount, BigInt(0));
    assert.isTrue(eventVaultAccount.mint.equals(acceptedMintPublicKey));
  });

  it("should buy tickets", async () => {
    // arrange
    const ticketQuantity = 10;
    const beforeAliceAccount = await getAccount(
      provider.connection,
      aliceAssociatedWalletPublicKey
    );
    const beforeEventVaultAccount = await getAccount(
      provider.connection,
      eventVaultPublicKey
    );
    const beforeTicketMintAccount = await getMint(
      provider.connection,
      ticketMintPublicKey
    );
    // act
    await program.methods
      .buyTickets(ticketQuantity)
      .accounts({
        authority: aliceKeypair.publicKey,
        eventBase: eventBaseKeypair.publicKey,
        acceptedMint: acceptedMintPublicKey,
        payerToken: aliceAssociatedWalletPublicKey,
      })
      .signers([aliceKeypair])
      .rpc();
    // assert
    const eventAccount = await program.account.event.fetch(eventPublicKey);
    const afterAliceAccount = await getAccount(
      provider.connection,
      aliceAssociatedWalletPublicKey
    );
    const afterEventVaultAccount = await getAccount(
      provider.connection,
      eventVaultPublicKey
    );
    const afterTicketMintAccount = await getMint(
      provider.connection,
      ticketMintPublicKey
    );
    const aliceTicketReceiverAccount = await getAccount(
      provider.connection,
      aliceTicketReceiverPublicKey
    );
    assert.isDefined(eventAccount);
    assert.isDefined(beforeAliceAccount);
    assert.isDefined(afterAliceAccount);
    assert.equal(
      afterAliceAccount.amount,
      beforeAliceAccount.amount -
        BigInt(ticketQuantity * eventAccount.ticketPrice)
    );
    assert.isDefined(beforeEventVaultAccount);
    assert.isDefined(afterEventVaultAccount);
    assert.equal(
      afterEventVaultAccount.amount,
      beforeEventVaultAccount.amount +
        BigInt(ticketQuantity * eventAccount.ticketPrice)
    );
    assert.isDefined(aliceTicketReceiverAccount);
    assert.equal(aliceTicketReceiverAccount.amount, BigInt(ticketQuantity));
    assert.isDefined(beforeTicketMintAccount);
    assert.isDefined(afterTicketMintAccount);
    assert.equal(
      afterTicketMintAccount.supply,
      beforeTicketMintAccount.supply + BigInt(ticketQuantity)
    );
  });
});
