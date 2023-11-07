import * as anchor from "@coral-xyz/anchor";

type ReturnData = {
  accounts: anchor.web3.AccountMeta[];
  hasMore: boolean;
};

/**
 *
 * @param program Assumes this program's IDL has `ExternalIAccountMeta` defined (copy of `IAccountMeta`)
 * @param instructions
 * @returns
 */
export async function resolveRemainingAccounts<I extends anchor.Idl>(
  program: anchor.Program<I>,
  instructions: anchor.web3.TransactionInstruction[],
  verbose: boolean = false
): Promise<ReturnData> {
  // Simulate transaction
  let message = anchor.web3.MessageV0.compile({
    payerKey: program.provider.publicKey!,
    instructions,
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

    // We start deserializing the Vec<IAccountMeta> from the 5th byte
    // The first 4 bytes are u32 for the Vec of the return data
    let numBytes = data.slice(0, 4);
    let numMetas = new anchor.BN(numBytes, null, "le");
    let offset = 4;

    let realAccountMetas: anchor.web3.AccountMeta[] = [];
    let coder = program.coder.types;
    const metaSize = 34;
    for (let i = 0; i < numMetas.toNumber(); i += 1) {
      const start = offset + i * metaSize;
      const end = start + metaSize;
      let meta = coder.decode("ExternalIAccountMeta", data.slice(start, end));
      realAccountMetas.push({
        pubkey: meta.pubkey,
        isWritable: meta.writable,
        isSigner: meta.signer,
      });
    }
    let hasMore = data.slice(offset + numMetas.toNumber() * metaSize)[0];
    return {
      accounts: realAccountMetas,
      hasMore: hasMore != 0,
    };
  } catch (e) {
    throw new Error("Failed to parse return data: " + e + "\n" + logs);
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
  verbose: boolean = false
): Promise<anchor.web3.TransactionInstruction> {
  // NOTE: LOL we have to do this because slicing only generates a view
  // so we need to copy it to a new buffer
  let ixDisc = Buffer.from(instruction.data.slice(0, 8));
  let newIxDisc = Buffer.from(
    anchor.utils.sha256.hash(`global:preflight_${methodName}`),
    "hex"
  ).slice(0, 8);

  instruction.data.set(newIxDisc, 0);
  if (verbose) {
    console.log("\tix", instruction.data.toString("hex"));
  }
  let hasMore = true;
  let i = 0;
  while (hasMore) {
    let result = await resolveRemainingAccounts(
      program,
      [instruction],
      verbose
    );
    if (verbose) {
      console.log(`Preflight result: ${result} (${i})`);
    }
    hasMore = result.hasMore;
    instruction.keys = instruction.keys.concat(result.accounts);
    i++;
  }
  instruction.data.set(ixDisc, 0);
  if (verbose) {
    console.log("\tix", instruction.data.toString("hex"));
  }
  return instruction;
}
