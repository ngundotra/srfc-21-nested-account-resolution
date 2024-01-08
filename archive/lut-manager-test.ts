import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { assert, expect } from "chai";
import { PRE_INSTRUCTIONS, sendTransaction } from "./lib/sendTransaction";
import { call } from "./lib/interface";
import {
  airdrop,
  getAddressLookupTable,
  getSlot,
  setupBankrun,
} from "./lib/utils";
import { UniversalMint } from "../target/types/universal_mint";
import { TOKEN_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import { LutManager } from "../target/types/lut_manager";

describe("lut-manager-tests", () => {
  let provider: anchor.Provider;
  let program: Program<LutManager>;
  let payer: anchor.web3.PublicKey;

  beforeEach(async () => {
    const setup = await setupBankrun();
    provider = setup.provider;
    program = setup.lutManager;
    payer = setup.provider.publicKey;
  });

  describe("tests", () => {
    describe(`Create-Add-Close flow`, () => {
      let mintKp: anchor.web3.Keypair;
      let mint: anchor.web3.PublicKey;
      let destination: anchor.web3.PublicKey;
      beforeEach(async () => {
        mintKp = anchor.web3.Keypair.generate();
        mint = mintKp.publicKey;

        destination = anchor.web3.Keypair.generate().publicKey;
      });

      it(`create`, async () => {
        const slot = await getSlot(provider.connection);

        const [_, lut] =
          anchor.web3.AddressLookupTableProgram.createLookupTable({
            payer,
            authority: payer,
            recentSlot: slot,
          });

        const txId = await program.methods
          .createLut(new anchor.BN(slot))
          .accounts({
            authority: payer,
            payer,
            lut,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .preInstructions(PRE_INSTRUCTIONS)
          .rpc({ skipPreflight: false, commitment: "confirmed" });
      });

      // it(`add`, async () => {
      //   const txId = await program.methods
      //     .createSplTokenExtension(6)
      //     .accounts({
      //       payer,
      //       mint,
      //       tokenProgram: TOKEN_PROGRAM_2022_ID,
      //     })
      //     .preInstructions(PRE_INSTRUCTIONS)
      //     .signers([mintKp])
      //     .rpc({ skipPreflight: false, commitment: "confirmed" });
      // });
    });
  });
});
