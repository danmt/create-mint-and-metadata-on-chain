import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { getMint } from "@solana/spl-token";
import { assert } from "chai";
import { Disco } from "../target/types/disco";

describe("disco", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider);

  const program = anchor.workspace.Disco as Program<Disco>;
  const eventBaseKeypair = anchor.web3.Keypair.generate();

  let eventPublicKey: anchor.web3.PublicKey;
  let ticketMintPublicKey: anchor.web3.PublicKey;

  before(async () => {
    [eventPublicKey] = await anchor.web3.PublicKey.findProgramAddress([
      Buffer.from('event', 'utf-8'),
      eventBaseKeypair.publicKey.toBuffer()
    ], program.programId);
    [ticketMintPublicKey] = await anchor.web3.PublicKey.findProgramAddress([
      Buffer.from('ticket_mint', 'utf-8'),
      eventPublicKey.toBuffer()
    ], program.programId)
  })


  it("Is initialized!", async () => {
    // act
    await program.methods.createEvent().accounts({
      authority: provider.wallet.publicKey,
      eventBase: eventBaseKeypair.publicKey,
    }).rpc();
    // assert
    const eventAccount = await program.account.event.fetch(eventPublicKey);
    const ticketMintAccount = await getMint(provider.connection, ticketMintPublicKey);
    
    console.log(eventAccount);
    console.log(ticketMintAccount);

    assert.isDefined(eventAccount);
  });
});
