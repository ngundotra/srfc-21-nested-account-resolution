import * as anchor from "@coral-xyz/anchor";
import { readFileSync } from "fs";
import { homedir } from "os";
import { ProgramTestContext } from "solana-bankrun";
import { GLOBAL_CONTEXT } from "./additionalAccountsRequest";

type Opts = {
  logs?: boolean;
  simulate?: boolean;
  verbose?: boolean;
  signers?: anchor.web3.Keypair[];
  lookupTableAddress?: anchor.web3.PublicKey;
};

export const PRE_INSTRUCTIONS = [
  anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
    units: 1_400_000,
  }),
  // Only need this is we consume too much heap while resolving / identifying accounts
  anchor.web3.ComputeBudgetProgram.requestHeapFrame({
    bytes: 1024 * 32 * 8,
  }),
];

export function getLocalKp(): anchor.web3.Keypair {
  return anchor.web3.Keypair.fromSecretKey(
    Buffer.from(
      JSON.parse(readFileSync(`${homedir()}/.config/solana/id.json`).toString())
    )
  );
}

export async function sendTransaction(
  connection: anchor.web3.Connection,
  ixs: anchor.web3.TransactionInstruction[],
  opts: Opts = {
    simulate: false,
    verbose: false,
    lookupTableAddress: undefined,
  }
): Promise<{ computeUnits: number }> {
  let kp = getLocalKp();
  let lookupTable: anchor.web3.AddressLookupTableAccount | undefined;
  if (opts.lookupTableAddress) {
    lookupTable = (
      await connection.getAddressLookupTable(opts.lookupTableAddress, {
        commitment: "finalized",
      })
    ).value;
  }

  let numReplays = 0;
  while (numReplays < 3) {
    try {
      let message = anchor.web3.MessageV0.compile({
        payerKey: kp.publicKey,
        instructions: PRE_INSTRUCTIONS.concat(ixs),
        addressLookupTableAccounts: lookupTable ? [lookupTable] : undefined,
        recentBlockhash: !!GLOBAL_CONTEXT
          ? (
              await GLOBAL_CONTEXT.banksClient.getLatestBlockhash("confirmed")
            )[0]
          : (await connection.getRecentBlockhash()).blockhash,
      });
      let transaction = new anchor.web3.VersionedTransaction(message);

      if (opts.simulate) {
        let simulationResult = await connection.simulateTransaction(
          transaction,
          {
            commitment: "confirmed",
          }
        );

        if (opts.logs) {
          console.log(simulationResult.value.logs.join("\n"));
        }

        return { computeUnits: simulationResult.value.unitsConsumed };
      } else if (!!GLOBAL_CONTEXT) {
        const meta = await (
          GLOBAL_CONTEXT as ProgramTestContext
        ).banksClient.processTransaction(transaction);
        return { computeUnits: parseInt(meta.computeUnitsConsumed.toString()) };
      } else {
        transaction.sign([kp]);
        transaction.sign(opts.signers ?? []);

        let serialized = transaction.serialize();
        let txid = await connection.sendRawTransaction(serialized, {
          skipPreflight: true,
        });

        if (opts.verbose) {
          console.log({
            serialized: serialized.length,
            keys: ixs[ixs.length - 1].keys.length,
          });
          console.log({ txid });
        }

        await connection.confirmTransaction(txid, "confirmed");

        const txresp = await connection.getTransaction(txid, {
          commitment: "confirmed",
          maxSupportedTransactionVersion: 2,
        });

        if (txresp.meta.err) {
          console.error(txresp.meta.logMessages.join("\n"));
        } else if (opts.logs) {
          console.log(txresp.meta.logMessages.join("\n"));
        }
        if (txresp.meta.err) {
          throw new Error(
            `Error sending transaction: ${JSON.stringify(txresp.meta.err)}`
          );
        }

        return { computeUnits: txresp.meta.computeUnitsConsumed };
      }
    } catch (e) {
      if (e instanceof anchor.web3.TransactionExpiredTimeoutError) {
        console.log("Retrying transaction");
        numReplays += 1;
      } else {
        throw e;
      }
    }
  }
}
