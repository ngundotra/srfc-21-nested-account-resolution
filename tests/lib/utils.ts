import * as anchor from "@coral-xyz/anchor";
import { Callee, IDL as CalleeIDL } from "../../target/types/callee";
import { assert } from "chai";
import {
  CallerWrapper,
  IDL as CallerWrapperIDL,
} from "../../target/types/caller_wrapper";
import { Caller, IDL as CallerIDL } from "../../target/types/caller";
import {
  UniversalMint,
  IDL as UniversalMintIdl,
} from "../../target/types/universal_mint";
import { startAnchor } from "solana-bankrun";
import { GLOBAL_CONTEXT, setGlobalContext } from "./additionalAccountsRequest";
import { getLocalKp } from "./sendTransaction";
import { BankrunProvider } from "anchor-bankrun";
import { parse } from "toml";
import { readFileSync, existsSync, mkdirSync } from "fs";
import { join } from "path";
import { exec } from "child_process";

export const TOKEN_PROGRAM_2022_ID = new anchor.web3.PublicKey(
  "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
);

export type ObjectCreationMeta = {
  metas: anchor.web3.AccountMeta[];
  signers: anchor.web3.Keypair[];
};

export async function createLinkedList(
  program: anchor.Program<Callee>,
  numNodes: number,
  opts?: {
    payer?: anchor.web3.Keypair;
  }
): Promise<ObjectCreationMeta> {
  let headKp: anchor.web3.Keypair = null;

  let nodeKps: anchor.web3.Keypair[] = [];
  let nodeMetas: anchor.web3.AccountMeta[] = [];
  nodeKps = [];
  nodeMetas = [];
  for (let i = 0; i < numNodes; i++) {
    let kp = anchor.web3.Keypair.generate();
    nodeKps.push(kp);
    nodeMetas.push({
      pubkey: kp.publicKey,
      isWritable: true,
      isSigner: true,
    });
  }
  headKp = nodeKps[0];

  // Override payer & signers if provided
  let payer: anchor.web3.PublicKey;
  let signers = nodeKps;
  if (opts && opts.payer) {
    payer = opts.payer.publicKey ?? program.provider.publicKey!;
    signers = [opts.payer].concat(signers);
  }

  await program.methods
    .createLinkedList(numNodes)
    .accounts({ payer })
    .remainingAccounts(nodeMetas)
    .signers(signers)
    .rpc({ skipPreflight: true, commitment: "confirmed" });

  return { metas: nodeMetas, signers: nodeKps };
}

interface PublicKeyGetter {
  publicKey: anchor.web3.PublicKey;
}

export async function validateLinkedListTransfer(
  program: anchor.Program<Callee>,
  nodeKps: PublicKeyGetter[],
  numNodes: number,
  destination: anchor.web3.PublicKey
) {
  // Normally you would do fetchMultiple, but because we want
  // our underlying connection object to work with Bankrun, we instead
  // Promise.all here
  let nodes = await Promise.all(
    nodeKps.map((kp) => program.account.node.fetch(kp.publicKey, "confirmed"))
  );

  for (let i = 0; i < numNodes - 1; i++) {
    assert(nodes[i].owner.toBase58() === destination.toBase58());
    assert(
      nodes[i].next.toString() === nodeKps[i + 1].publicKey.toString(),
      `${i}th node's next is not correct!`
    );
  }
  assert(nodes[numNodes - 1].next === null);
}

export async function validateOwnershipListTransfer(
  program: anchor.Program<Callee>,
  ownershipListKey: anchor.web3.PublicKey,
  destination: anchor.web3.PublicKey
) {
  let ownershipList = await program.account.ownershipList.fetch(
    ownershipListKey,
    "confirmed"
  );

  assert(ownershipList.owner.toBase58() === destination.toBase58());
}

export async function getSlot(connection: anchor.web3.Connection) {
  return !!GLOBAL_CONTEXT
    ? parseInt(
        ((await GLOBAL_CONTEXT.banksClient.getSlot()) - BigInt(1)).toString()
      )
    : (await connection.getLatestBlockhashAndContext()).context.slot;
}

export async function getLatestBlockhash(connection: anchor.web3.Connection) {
  return !!GLOBAL_CONTEXT
    ? (await GLOBAL_CONTEXT.banksClient.getLatestBlockhash("confirmed"))[0]
    : (await connection.getRecentBlockhash()).blockhash;
}

export async function getAddressLookupTable(
  connection: anchor.web3.Connection,
  table: anchor.web3.PublicKey,
  commitment: anchor.web3.Commitment = "confirmed"
): Promise<anchor.web3.AddressLookupTableAccount | null> {
  if (!!GLOBAL_CONTEXT) {
    const data = await GLOBAL_CONTEXT.banksClient.getAccount(table);

    const deactivationSlot = BigInt(
      new anchor.BN(data.data.slice(0, 8), "le").toString()
    );
    const lastExtendedSlot = new anchor.BN(
      data.data.slice(8, 16),
      "le"
    ).toNumber();
    const lastExtendedSlotStartIndex = data.data[20];
    const authority = new anchor.web3.PublicKey(data.data.slice(22, 54));
    const addresses: anchor.web3.PublicKey[] = [];
    for (let i = 0; i < (data.data.length - 56) / 32; i++) {
      addresses.push(
        new anchor.web3.PublicKey(
          data.data.slice(56 + 32 * i, 56 + 32 + 32 * i)
        )
      );
    }
    return new anchor.web3.AddressLookupTableAccount({
      key: table,
      state: {
        deactivationSlot,
        lastExtendedSlot,
        lastExtendedSlotStartIndex,
        authority,
        addresses,
      },
    });
  } else {
    return (
      await connection.getAddressLookupTable(table, {
        commitment,
      })
    ).value;
  }
}

export async function airdrop(
  connection: anchor.web3.Connection,
  destination: anchor.web3.PublicKey,
  sol: number
) {
  if (!!GLOBAL_CONTEXT) {
    GLOBAL_CONTEXT.setAccount(destination, {
      /** `true` if this account's data contains a loaded program */
      executable: false,
      /** Identifier of the program that owns the account */
      owner: anchor.web3.SystemProgram.programId,
      /** Number of lamports assigned to the account */
      lamports: sol * anchor.web3.LAMPORTS_PER_SOL,
      /** Optional data assigned to the account */
      data: Buffer.from([]),
    });
  } else {
    await connection.requestAirdrop(
      destination,
      sol * anchor.web3.LAMPORTS_PER_SOL
    );
  }
}

export async function setupBankrun() {
  const config = {
    defaultUrl: "https://api.devnet.solana.com",
    programs: [
      {
        programId: TOKEN_PROGRAM_2022_ID,
        name: "token2022",
      },
      {
        programId: new anchor.web3.PublicKey(
          "8bvPnYE5Pvz2Z9dE6RAqWr1rzLknTndZ9hwvRE6kPDXP"
        ),
        name: "libreplex_fair_launch",
      },
    ],
  };

  // Downloads & caches programs
  for (const program of config.programs) {
    const directory = join(__dirname, "../../tests/fixtures");
    if (!existsSync(directory)) {
      mkdirSync(directory);
    }

    if (!existsSync(join(directory, `${program.name}.so`))) {
      const { stderr } = exec(
        `solana program dump ${program.programId} ${directory}/${program.name}.so -u ${config.defaultUrl}`
      );
      if (stderr) {
        console.error(stderr);
        console.error(
          `Error caching program: ${program.name} from ${config.defaultUrl}.`
        );
        throw new Error("Failed to cache program");
      }
    }
  }

  const context = await startAnchor(
    join(__dirname, "../.."),
    config.programs,
    []
  );

  setGlobalContext(context);

  const payerKp = getLocalKp();
  const payer = payerKp.publicKey;
  const provider = new BankrunProvider(context, new anchor.Wallet(payerKp));

  context.setAccount(payer, {
    /** `true` if this account's data contains a loaded program */
    executable: false,
    /** Identifier of the program that owns the account */
    owner: anchor.web3.SystemProgram.programId,
    /** Number of lamports assigned to the account */
    lamports: 50 * anchor.web3.LAMPORTS_PER_SOL,
    /** Optional data assigned to the account */
    data: Buffer.from([]),
  });

  const fname = join(__dirname, "../../Anchor.toml");
  const anchorToml = parse(readFileSync(fname).toString());
  const programs: Record<string, string> = anchorToml.programs.localnet;

  const callee = new anchor.Program<Callee>(
    CalleeIDL,
    new anchor.web3.PublicKey(programs.callee),
    provider
  );

  const caller = new anchor.Program<Caller>(
    CallerIDL,
    new anchor.web3.PublicKey(programs.caller),
    provider
  );

  const callerWrapper = new anchor.Program<CallerWrapper>(
    CallerWrapperIDL,
    new anchor.web3.PublicKey(programs.caller_wrapper),
    provider
  );

  const universalMint = new anchor.Program<UniversalMint>(
    UniversalMintIdl,
    new anchor.web3.PublicKey(programs.universal_mint),
    provider
  );

  return {
    callee,
    caller,
    callerWrapper,
    universalMint,
    provider,
    context,
  };
}
