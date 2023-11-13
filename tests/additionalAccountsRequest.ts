import * as anchor from "@coral-xyz/anchor";
import { sha256 } from "@noble/hashes/sha256";
import { PRE_INSTRUCTIONS, sendTransaction } from "./sendTransaction";

type AdditionalAccounts = {
  accounts: anchor.web3.AccountMeta[];
  hasMore: boolean;
};

const MAX_ACCOUNTS = 30;

/**
 *
 * @param program Assumes this program's IDL has `ExternalIAccountMeta` defined (copy of `IAccountMeta`)
 * @param instructions
 * @returns
 */
export async function resolveRemainingAccounts<I extends anchor.Idl>(
  program: anchor.Program<I>,
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
      lookupTable = (
        await program.provider.connection.getAddressLookupTable(slut, {
          commitment: "confirmed",
        })
      ).value;
      await new Promise((resolve) => setTimeout(resolve, 1000));
    }
  }

  let message = anchor.web3.MessageV0.compile({
    payerKey: program.provider.publicKey!,
    instructions: PRE_INSTRUCTIONS.concat(instructions),
    addressLookupTableAccounts: slut ? [lookupTable] : undefined,
    recentBlockhash: (await program.provider.connection.getRecentBlockhash())
      .blockhash,
  });
  let transaction = new anchor.web3.VersionedTransaction(message);
  let simulationResult = await program.provider.connection.simulateTransaction(
    transaction,
    {
      commitment: "confirmed",
    }
  );

  if (verbose) {
    console.log("CUs consumed:", simulationResult.value.unitsConsumed);
    console.log("Logs", simulationResult.value.logs);
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
  let logs = simulationResult.value.logs;

  try {
    let b64Data = anchor.utils.bytes.base64.decode(
      logs[logs.length - 2].split(" ")[3]
    );
    let data = b64Data;

    if (!data.length) {
      throw new Error(
        `No return data found in preflight simulation:
      ${simulationResult.value.logs}`
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
    // let hasMore = data.slice(offset + numMetas.toNumber() * metaSize)[0];

    // if (verbose) {
    //   console.log("num metas:", numMetas.toNumber());
    //   console.log("offset", numMetas.toNumber() * metaSize + offset);
    //   console.log("length", data.length);
    //   console.log(
    //     "Remaining bytes:",
    //     data.slice(offset + numMetas.toNumber() * metaSize)
    //   );

    //   console.log("hasMore", hasMore);
    // }
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

/**
 * Takes a serialized Anchor Instruction
 * And executes a preflight instruction to get the remaining accounts
 * @param program
 * @param instruction
 * @param verbose
 * @returns
 */
export async function additionalAccountsRequest<I extends anchor.Idl>(
  program: anchor.Program<I>,
  instruction: anchor.web3.TransactionInstruction,
  methodName: string,
  verbose: boolean = false,
  slut: boolean = false
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

  let newIxDisc = Buffer.from(sha256(`global:preflight_${methodName}`)).slice(
    0,
    8
  );
  currentBuffer.set(newIxDisc, 0);

  if (verbose) {
    console.log("\tix", instruction.data.toString("hex"));
  }

  let additionalAccounts: anchor.web3.AccountMeta[][] = [[]];
  let hasMore = true;
  let page = 0;
  let i = 0;
  let lookupTable: anchor.web3.PublicKey | undefined;
  let lastSize = 0;
  while (hasMore) {
    if (verbose) {
      console.log(
        `Page: ${page} | additionalAccounts: ${
          additionalAccounts.flat().length
        }`
      );
    }

    // Write the current page number at the end of the instruction data
    instruction.data = Buffer.concat([currentBuffer, Buffer.from([page])]);

    // Add found accounts to instruction
    instruction.keys = originalKeys.concat(additionalAccounts.flat());
    let result = await resolveRemainingAccounts(
      program,
      [instruction],
      false,
      lookupTable
    );
    // if (verbose) {
    //   console.log(`Preflight result: ${JSON.stringify(result)} (${i})`);
    // }

    if (verbose) {
      console.log(`Page: ${page} | requested: ${result.accounts.length}`);
    }
    hasMore = result.hasMore;
    additionalAccounts[page] = result.accounts;

    if (additionalAccounts.flat().length >= 10 && slut) {
      if (!lookupTable) {
        const [ix, tableAddr] =
          anchor.web3.AddressLookupTableProgram.createLookupTable({
            authority: program.provider.publicKey!,
            payer: program.provider.publicKey!,
            recentSlot: (
              await program.provider.connection.getLatestBlockhashAndContext()
            ).context.slot,
          });

        await sendTransaction(program.provider.connection, [ix]);
        lookupTable = tableAddr;
      }

      if (additionalAccounts.flat().length - lastSize > 10) {
        const ix = anchor.web3.AddressLookupTableProgram.extendLookupTable({
          authority: program.provider.publicKey!,
          payer: program.provider.publicKey!,
          addresses: additionalAccounts
            .flat()
            .slice(lastSize)
            .map((acc) => acc.pubkey),
          lookupTable,
        });

        await sendTransaction(program.provider.connection, [ix]);
        lastSize = additionalAccounts.flat().length;
      }
    }

    i++;
    if (i >= 16) {
      throw new Error(`Too many iterations ${i}`);
    }
    if (result.accounts.length === MAX_ACCOUNTS && hasMore) {
      page++;
    }
  }

  if (slut && lookupTable && additionalAccounts.flat().length - lastSize > 0) {
    const ix = anchor.web3.AddressLookupTableProgram.extendLookupTable({
      authority: program.provider.publicKey!,
      payer: program.provider.publicKey!,
      addresses: additionalAccounts
        .flat()
        .slice(lastSize)
        .map((acc) => acc.pubkey),
      lookupTable,
    });

    await sendTransaction(program.provider.connection, [ix]);
  }

  instruction.keys = originalKeys.concat(additionalAccounts.flat());

  // Reset original data
  instruction.data = originalData;

  if (slut && lookupTable) {
    let activeSlut = false;
    while (!activeSlut) {
      let table = await program.provider.connection.getAddressLookupTable(
        lookupTable,
        { commitment: "finalized" }
      );
      if (table.value) {
        activeSlut =
          table.value.isActive() &&
          table.value.state.addresses.length ===
            additionalAccounts.flat().length;
      }
      await new Promise((resolve) => setTimeout(resolve, 1000));
    }
  }
  return { ix: instruction, lookupTable };
}
