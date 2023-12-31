import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { assert, expect } from "chai";
import { PRE_INSTRUCTIONS, sendTransaction } from "./lib/sendTransaction";
import { call } from "./lib/interface";
import { airdrop, setupBankrun } from "./lib/utils";
import { UniversalMint } from "../target/types/universal_mint";
import {
  ASSOCIATED_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@coral-xyz/anchor/dist/cjs/utils/token";

import { TOKEN_PROGRAM_2022_ID } from "./lib/utils";

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

      it(`(tokenkeg) initialize mint`, async () => {
        const txId = await program.methods
          .createSplToken(6)
          .accounts({
            payer,
            mint,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .preInstructions(PRE_INSTRUCTIONS)
          .signers([mintKp])
          .rpc({ skipPreflight: false, commitment: "confirmed" });
      });

      it(`(token22) initialize mint`, async () => {
        const txId = await program.methods
          .createSplTokenExtension(6)
          .accounts({
            payer,
            mint,
            tokenProgram: TOKEN_PROGRAM_2022_ID,
          })
          .preInstructions(PRE_INSTRUCTIONS)
          .signers([mintKp])
          .rpc({ skipPreflight: false, commitment: "confirmed" });
      });

      it(`(token22) initialize mint + metadata`, async () => {
        const name = "name";
        const description = "description";
        let ata = anchor.web3.PublicKey.findProgramAddressSync(
          [payer.toBuffer(), TOKEN_PROGRAM_2022_ID.toBuffer(), mint.toBuffer()],
          ASSOCIATED_PROGRAM_ID
        )[0];

        // const txId = await program.methods
        //   .preflightCreateSplTokenExtensionMetadata(name, description)
        //   .accounts({
        //     payer,
        //     mint,
        //   })
        //   .preInstructions(PRE_INSTRUCTIONS)
        //   // .signers([mintKp])
        //   .rpc({ skipPreflight: false, commitment: "confirmed" });

        // const txId = await program.methods
        //   .createSplTokenExtensionMetadata(name, description)
        //   .accounts({
        //     payer,
        //     mint,
        //     ata,
        //     associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
        //     tokenProgram: TOKEN_PROGRAM_2022_ID,
        //   })
        //   .preInstructions(PRE_INSTRUCTIONS)
        //   .signers([mintKp])
        //   .rpc({ skipPreflight: false, commitment: "confirmed" });

        const txId = await call(
          provider.connection,
          program.programId,
          "create_spl_token_extension_metadata",
          [
            { pubkey: payer, isSigner: true, isWritable: true },
            { pubkey: mint, isSigner: true, isWritable: true },
          ],
          Buffer.concat([
            Buffer.from(new anchor.BN(name.length).toArray("le", 4)),
            Uint8Array.from(Buffer.from(name, "utf-8")),
            Buffer.from(new anchor.BN(description.length).toArray("le", 4)),
            Uint8Array.from(Buffer.from(description, "utf-8")),
          ]),
          { signers: [mintKp], verbose: true }
        );

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
        assert(accountInfo.name === name);
        assert(accountInfo.description === description);
      });
    });
  });
});
