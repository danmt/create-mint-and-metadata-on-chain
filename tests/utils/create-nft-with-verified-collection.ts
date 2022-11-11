import { Metaplex } from "@metaplex-foundation/js";
import * as anchor from "@project-serum/anchor";
import { createSetAndVerifyCollectionInstruction } from "@metaplex-foundation/mpl-token-metadata";

export const createNftWithVerifiedCollection = async (
  metaplex: Metaplex,
  provider: anchor.AnchorProvider,
  collectionName = "my collection nft",
  nftName = "my nft"
) => {
  const collectionNft = await metaplex
    .nfts()
    .create({
      name: collectionName,
      sellerFeeBasisPoints: 0,
      uri: "",
    })
    .run();

  const { nft } = await metaplex
    .nfts()
    .create({
      name: nftName,
      sellerFeeBasisPoints: 1,
      uri: "",
      collection: {
        key: collectionNft.metadataAddress,
        verified: false,
      },
    })
    .run();

  await provider.sendAndConfirm(
    new anchor.web3.Transaction().add(
      createSetAndVerifyCollectionInstruction({
        collectionMint: collectionNft.nft.mintAddress,
        collection: collectionNft.nft.metadataAddress,
        collectionAuthority: provider.wallet.publicKey,
        collectionMasterEditionAccount: collectionNft.masterEditionAddress,
        metadata: nft.metadataAddress,
        payer: provider.wallet.publicKey,
        updateAuthority: provider.wallet.publicKey,
      })
    )
  );

  return [
    collectionNft.nft.mintAddress,
    nft.mintAddress
  ];
};
