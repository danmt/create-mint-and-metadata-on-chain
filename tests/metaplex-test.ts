import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { getMint } from "@solana/spl-token";
import { assert } from "chai";
import { Disco } from "../target/types/disco";
import {
  bundlrStorage,
  Metaplex,
  walletAdapterIdentity,
  isMintWithMetadata,
} from "@metaplex-foundation/js";

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

  let eventPublicKey: anchor.web3.PublicKey;
  let ticketMintPublicKey: anchor.web3.PublicKey;
  let metadataPublicKey: anchor.web3.PublicKey;

  before(async () => {
    [eventPublicKey] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("event", "utf-8"), eventBaseKeypair.publicKey.toBuffer()],
      program.programId
    );
    [ticketMintPublicKey] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("ticket_mint", "utf-8"), eventPublicKey.toBuffer()],
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
  });

  it("should create an event", async () => {
    // arrange
    const ticketTitle = "Ticket Title";
    const ticketSymbol = "TICKET";
    const ticketURI = "https://google.com";
    // act
    await program.methods
      .createEvent(ticketTitle, ticketSymbol, ticketURI)
      .accounts({
        authority: provider.wallet.publicKey,
        eventBase: eventBaseKeypair.publicKey,
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
  });
});
