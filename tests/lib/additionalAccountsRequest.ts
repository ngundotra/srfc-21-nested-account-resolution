import * as anchor from "@coral-xyz/anchor";
import { sha256 } from "@noble/hashes/sha256";
import {
  PRE_INSTRUCTIONS,
  getLocalKp,
  sendTransaction,
} from "./sendTransaction";
import { ProgramTestContext } from "solana-bankrun";
import { getAddressLookupTable, getLatestBlockhash, getSlot } from "./utils";

type AdditionalAccounts = {
  accounts: anchor.web3.AccountMeta[];
  hasMore: boolean;
};

export let GLOBAL_CONTEXT: ProgramTestContext | null = null;
export function setGlobalContext(context: ProgramTestContext) {
  GLOBAL_CONTEXT = context;
}

const MAX_ACCOUNTS = 30;

/**
 *
 * @param program
 * @param instructions
 * @returns
 */
export async function resolveRemainingAccounts(
  connection: anchor.web3.Connection,
  instructions: anchor.web3.TransactionInstruction[],
  verbose: boolean = false,
  slut: anchor.web3.PublicKey | undefined = undefined
): Promise<AdditionalAccounts> {
  // Simulate transaction
  let lookupTable: anchor.web3.AddressLookupTableAccount | undefined;
  if (slut) {
    if (verbose) {
      console.log(`SLUT resolution with ${slut.toBase58()}`);
    }
    while (!lookupTable) {
      lookupTable = await getAddressLookupTable(connection, slut);
      await new Promise((resolve) => setTimeout(resolve, 1000));
    }
  }

  let message = anchor.web3.MessageV0.compile({
    payerKey: getLocalKp().publicKey!,
    instructions: PRE_INSTRUCTIONS.concat(instructions),
    addressLookupTableAccounts: slut ? [lookupTable] : undefined,
    recentBlockhash: await getLatestBlockhash(connection),
  });
  let transaction = new anchor.web3.VersionedTransaction(message);

  // let simulationResult;
  let unitsConsumed: number;
  let returnData: Buffer;
  let logs: string[];
  let err: any;
  if (!!GLOBAL_CONTEXT) {
    let simulationResult = await GLOBAL_CONTEXT.banksClient.simulateTransaction(
      transaction,
      "confirmed"
    );
    logs = simulationResult.meta.logMessages;
    // For some reason, still getting null from #[napi(getter)] on return_data
    // returnData = Buffer.from(simulationResult.meta.returnData);
    unitsConsumed = parseInt(
      simulationResult.meta.computeUnitsConsumed.toString()
    );
  } else {
    let simulationResult = await connection.simulateTransaction(transaction, {
      commitment: "confirmed",
    });
    logs = simulationResult.value.logs;
    unitsConsumed = simulationResult.value.unitsConsumed;
    err = simulationResult.value.err;
  }

  if (verbose) {
    console.log("CUs consumed:", unitsConsumed);
    console.log("Logs", logs);
    console.log("Result", err);
  }

  // When the simulation RPC response is fixed, then the following code will work
  // but until then, we have to parse the logs manually.
  //
  // ISSUE: rpc truncates trailing 0 bytes in `returnData` field, so we have
  // to actually parse the logs for the whole return data
  // ===============================================================
  // let returnDataTuple = simulationResult.value.returnData;
  // let [b64Data, encoding] = returnDataTuple["data"];
  // if (encoding !== "base64") {
  //   throw new Error("Unsupported encoding: " + encoding);
  // }
  // ===============================================================

  try {
    let b64Data = anchor.utils.bytes.base64.decode(
      logs[logs.length - 2].split(" ")[3]
    );
    let data = b64Data;

    if (!data.length) {
      throw new Error(
        `No return data found in preflight simulation:
      ${logs}`
      );
    }

    if (data.length !== 1024) {
      throw new Error(
        `Return data incorrect size in preflight simulation:
      ${data.length} (expected 1024)`
      );
    }

    // We start deserializing the Vec<IAccountMeta> from the 5th byte
    // The first 4 bytes are u32 for the Vec of the return data
    let protocolVersion = data[0];
    if (protocolVersion !== 0) {
      throw new Error(
        `Unsupported Account Resolution Protocol version: ${protocolVersion}`
      );
    }
    let hasMore = data[1];
    let numAccounts = data.slice(4, 8);
    let numMetas = new anchor.BN(numAccounts, null, "le");

    let offset = 8;
    let realAccountMetas: anchor.web3.AccountMeta[] = [];
    for (let i = 0; i < numMetas.toNumber(); i += 1) {
      let pubkey = new anchor.web3.PublicKey(
        data.slice(offset + i * 32, offset + (i + 1) * 32)
      );
      let writable = data[offset + MAX_ACCOUNTS * 32 + i];
      realAccountMetas.push({
        pubkey,
        isWritable: writable === 1,
        isSigner: false,
      });
    }

    return {
      accounts: realAccountMetas,
      hasMore: hasMore != 0,
    };
  } catch (e) {
    throw new Error(
      "Failed to parse return data: " + e + "\n" + logs.join("\n")
    );
  }
}

async function extendLookupTable(
  additionalAccounts: anchor.web3.AccountMeta[],
  lastSize: number,
  connection: anchor.web3.Connection,
  lookupTable: anchor.web3.PublicKey
): Promise<number> {
  while (additionalAccounts.flat().length - lastSize) {
    // 29 is max number of accounts we can extend a lookup table by in a single transaction
    // ironically due to tx limits
    const batchSize = Math.min(29, additionalAccounts.length - lastSize);

    const localPubkey = getLocalKp().publicKey;
    const ix = anchor.web3.AddressLookupTableProgram.extendLookupTable({
      authority: localPubkey,
      payer: localPubkey,
      addresses: additionalAccounts
        .flat()
        .slice(lastSize, lastSize + batchSize)
        .map((acc) => acc.pubkey),
      lookupTable,
    });

    await sendTransaction(connection, [ix]);
    lastSize += batchSize;
  }
  return lastSize;
}

async function pollForActiveLookupTable(
  additionalAccounts: anchor.web3.AccountMeta[],
  connection: anchor.web3.Connection,
  lookupTable: anchor.web3.PublicKey
) {
  if (!GLOBAL_CONTEXT) {
    let activeSlut = false;
    while (!activeSlut) {
      let table = await connection.getAddressLookupTable(lookupTable, {
        commitment: "finalized",
      });
      if (table.value) {
        activeSlut =
          table.value.isActive() &&
          table.value.state.addresses.length === additionalAccounts.length;
      }
      await new Promise((resolve) => setTimeout(resolve, 1000));
    }
  } else {
    let slot = (await getSlot(connection)) + 2;
    GLOBAL_CONTEXT.warpToSlot(BigInt(slot));
  }
}

export function hashIxName(ixName: string, namespace?: string): Buffer {
  return Buffer.from(sha256(`${namespace ?? "global"}:${ixName}`)).slice(0, 8);
}

/**
 * Takes a serialized Anchor Instruction
 * And executes a preflight instruction to get the remaining accounts
 * @param program
 * @param instruction
 * @param verbose
 * @returns
 */
export async function additionalAccountsRequest<I extends anchor.Idl>(
  connection: anchor.web3.Connection,
  instruction: anchor.web3.TransactionInstruction,
  methodName: string,
  verbose: boolean = false,
  slut: boolean = false,
  namespace?: string
): Promise<{
  ix: anchor.web3.TransactionInstruction;
  lookupTable?: anchor.web3.PublicKey;
}> {
  // NOTE: LOL we have to do this because slicing only generates a view
  // so we need to copy it to a new buffer
  let originalData = Buffer.from(instruction.data);
  let originalKeys = [].concat(instruction.keys);

  // Overwrite the discriminator
  let currentBuffer = Buffer.from(instruction.data);

  let newIxDisc = hashIxName(`preflight_${methodName}`, namespace);
  currentBuffer.set(newIxDisc, 0);

  let additionalAccounts: anchor.web3.AccountMeta[] = [];
  let hasMore = true;
  let i = 0;
  let lookupTable: anchor.web3.PublicKey | undefined;
  let lastSize = 0;
  while (hasMore) {
    if (verbose) {
      console.log(
        `Iteration: ${i} | additionalAccounts: ${additionalAccounts.length}`
      );
    }

    // Write the current page number at the end of the instruction data
    instruction.data = currentBuffer;

    // Add found accounts to instruction
    instruction.keys = originalKeys.concat(additionalAccounts.flat());

    let result = await resolveRemainingAccounts(
      connection,
      [instruction],
      verbose,
      lookupTable
    );

    if (verbose) {
      console.log(`Iteration: ${i} | requested: ${result.accounts.length}`);
    }
    hasMore = result.hasMore;
    additionalAccounts = additionalAccounts.concat(result.accounts);

    let localKp = getLocalKp().publicKey;
    if (additionalAccounts.length >= 10 && slut) {
      if (!lookupTable) {
        const [ix, tableAddr] =
          anchor.web3.AddressLookupTableProgram.createLookupTable({
            authority: localKp,
            payer: localKp,
            recentSlot: await getSlot(connection),
          });

        await sendTransaction(connection, [ix]);
        lookupTable = tableAddr;
      }

      // We want to minimize the number of non-transactional
      // txs we have to send on-chain. So we maximize # of accounts
      // to extend the lookup table by.
      // In practice, we can probably mix accounts from different resolutions
      // into the same extend LUT tx.
      if (additionalAccounts.length - lastSize >= 10) {
        if (verbose) {
          console.log("Extending lookup table...");
        }
        lastSize = await extendLookupTable(
          additionalAccounts,
          lastSize,
          connection,
          lookupTable
        );
        await pollForActiveLookupTable(
          additionalAccounts,
          connection,
          lookupTable
        );
        if (verbose) {
          console.log("...extended!");
        }
      }
    }

    i++;
    if (i >= 32) {
      throw new Error(`Too many iterations ${i}`);
    }
  }

  if (slut && lookupTable) {
    await extendLookupTable(
      additionalAccounts,
      lastSize,
      connection,
      lookupTable
    );
    await pollForActiveLookupTable(additionalAccounts, connection, lookupTable);
  }

  instruction.keys = originalKeys.concat(additionalAccounts);

  // Reset original data
  instruction.data = originalData;

  return { ix: instruction, lookupTable };
}
