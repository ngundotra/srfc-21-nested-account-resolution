import * as anchor from "@coral-xyz/anchor";
import { readFileSync } from "fs";
import { homedir } from "os";

type Opts = {
  logs?: boolean;
  simulate?: boolean;
  verbose?: boolean;
  lookupTableAddress?: anchor.web3.PublicKey;
};

export const PRE_INSTRUCTIONS = [
  anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
    units: 1_400_000,
  }),
  anchor.web3.ComputeBudgetProgram.requestHeapFrame({
    bytes: 1024 * 32 * 6,
  }),
];

export async function sendTransaction(
  connection: anchor.web3.Connection,
  ixs: anchor.web3.TransactionInstruction[],
  opts: Opts = {
    simulate: false,
    verbose: false,
    lookupTableAddress: undefined,
  }
): Promise<{ computeUnits: number }> {
  let kp = anchor.web3.Keypair.fromSecretKey(
    Buffer.from(
      JSON.parse(readFileSync(`${homedir()}/.config/solana/id.json`).toString())
    )
  );

  let lookupTable: anchor.web3.AddressLookupTableAccount | undefined;
  if (opts.lookupTableAddress) {
    lookupTable = (
      await connection.getAddressLookupTable(opts.lookupTableAddress, {
        commitment: "finalized",
      })
    ).value;
  }

  let message = anchor.web3.MessageV0.compile({
    payerKey: kp.publicKey,
    instructions: PRE_INSTRUCTIONS.concat(ixs),
    addressLookupTableAccounts: lookupTable ? [lookupTable] : undefined,
    recentBlockhash: (await connection.getRecentBlockhash()).blockhash,
  });
  let transaction = new anchor.web3.VersionedTransaction(message);

  if (opts.simulate) {
    let simulationResult = await connection.simulateTransaction(transaction, {
      commitment: "confirmed",
    });

    if (opts.logs) {
      console.log(simulationResult.value.logs.join("\n"));
    }

    return { computeUnits: simulationResult.value.unitsConsumed };
  } else {
    transaction.sign([kp]);

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
}
