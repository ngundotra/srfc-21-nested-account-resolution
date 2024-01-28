import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { assert } from "chai";
import { PRE_INSTRUCTIONS } from "./lib/sendTransaction";
import { call } from "./lib/interface";
import { getLatestBlockhash, setupBankrun } from "./lib/utils";
import { UniversalMint } from "../target/types/universal_mint";
import { getAccount } from "@solana/spl-token";
import {
  ASSOCIATED_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@coral-xyz/anchor/dist/cjs/utils/token";
import {
  createEmitInstruction,
  TokenMetadata,
  unpack as deserializeTokenMetadata,
} from "@solana/spl-token-metadata";

import { TOKEN_PROGRAM_2022_ID } from "./lib/utils";
import { GLOBAL_CONTEXT } from "./lib/additionalAccountsRequest";

async function getTokenMetadata(
  metadataPointer: anchor.web3.PublicKey,
  programId: anchor.web3.PublicKey,
  payer: anchor.web3.PublicKey,
  connection: anchor.web3.Connection
): Promise<TokenMetadata> {
  const ixs = [
    createEmitInstruction({
      metadata: metadataPointer,
      programId: programId,
    }),
  ];
  const message = anchor.web3.MessageV0.compile({
    payerKey: payer,
    recentBlockhash: await getLatestBlockhash(connection),
    instructions: ixs,
  });

  const res = await GLOBAL_CONTEXT.banksClient.simulateTransaction(
    new anchor.web3.VersionedTransaction(message)
  );

  const tm = deserializeTokenMetadata(
    Buffer.from(Array.from(res.meta.returnData.data))
  );
  return tm;
}

const MAX_URI_LEN = 98;

describe("universal-mint-tests", () => {
  let provider: anchor.Provider;
  let program: Program<UniversalMint>;
  let payer: anchor.web3.PublicKey;

  beforeEach(async () => {
    const setup = await setupBankrun();
    provider = setup.provider;
    program = setup.universalMint;
    payer = setup.provider.publicKey;
  });

  describe("Universal Mint tests", () => {
    describe(`Basic mint`, () => {
      let mintKp: anchor.web3.Keypair;
      let mint: anchor.web3.PublicKey;
      let destination: anchor.web3.PublicKey;
      beforeEach(async () => {
        mintKp = anchor.web3.Keypair.generate();
        mint = mintKp.publicKey;

        destination = anchor.web3.Keypair.generate().publicKey;
      });

      it(`(token22) initialize mint + metadata`, async () => {
        const name = (num: number) => `#${num.toString().padStart(4, " ")}`;
        const symbol = "UNDEAD";
        const uri = (num: number) =>
          `https://bafybeiffh25vb32ns6zspqjxcpkvqzvgmdn6xrzwnnt7eghfqkwdiwpeaq.ipfs.nftstorage.link/${num}.json`.padEnd(
            MAX_URI_LEN,
            "#"
          );
        const description = "This NFT changes its URI on every transfer";

        let ata = anchor.web3.PublicKey.findProgramAddressSync(
          [payer.toBuffer(), TOKEN_PROGRAM_2022_ID.toBuffer(), mint.toBuffer()],
          ASSOCIATED_PROGRAM_ID
        )[0];

        let computeUnits = await call(
          provider.connection,
          program.programId,
          "mint_new_nft",
          [
            { pubkey: payer, isSigner: true, isWritable: true },
            { pubkey: mint, isSigner: true, isWritable: true },
          ],
          Buffer.from([]),
          { signers: [mintKp], verbose: true }
        );

        let tokenInfo = await getAccount(
          provider.connection,
          ata,
          "confirmed",
          TOKEN_PROGRAM_2022_ID
        );
        console.log({ tokenInfo });

        let metadataPointer = anchor.web3.PublicKey.findProgramAddressSync(
          [
            mint.toBuffer(),
            Buffer.from("token22"),
            Buffer.from("metadata_pointer"),
          ],
          program.programId
        )[0];

        const accountInfo = await program.account.metadataInfo.fetch(
          metadataPointer
        );
        console.log("account info", accountInfo, name(0));
        assert(accountInfo.name === name(0));
        assert(accountInfo.description === description);

        const programAuthority = anchor.web3.PublicKey.findProgramAddressSync(
          [Buffer.from("AUTHORITY")],
          program.programId
        )[0];

        let tm = await getTokenMetadata(
          metadataPointer,
          program.programId,
          payer,
          provider.connection
        );
        assert.equal(tm.name, name(0));
        assert.equal(tm.symbol, symbol);
        assert.equal(tm.uri, uri(0));
        assert.equal(
          new Map(tm.additionalMetadata).get("Description"),
          description
        );
        assert.equal(
          tm.updateAuthority.toBase58(),
          programAuthority.toBase58(),
          "Expected update authority to be payer"
        );
        assert.equal(
          tm.mint.toBase58(),
          mint.toBase58(),
          "Expected mint to be correct"
        );

        // Transfer token
        computeUnits = await call(
          provider.connection,
          program.programId,
          "transfer_token",
          [
            { pubkey: payer, isSigner: true, isWritable: true },
            { pubkey: mint, isSigner: false, isWritable: true },
            { pubkey: destination, isSigner: false, isWritable: false },
          ],
          Buffer.concat([Buffer.from(new anchor.BN(1).toArray("le", 8))])
        );

        // Check that name was changed
        tm = await getTokenMetadata(
          metadataPointer,
          program.programId,
          payer,
          provider.connection
        );
        console.log("Final metadata", tm);
        assert.equal(tm.name, name(1));
        assert.equal(tm.symbol, symbol);
        assert.equal(tm.uri, uri(1));
      });
    });
  });
});
