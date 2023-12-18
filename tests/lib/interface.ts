import * as anchor from "@coral-xyz/anchor";
import {
  additionalAccountsRequest,
  hashIxName,
} from "./additionalAccountsRequest";
import { getLocalKp, sendTransaction } from "./sendTransaction";

type CallOpts = {
  useLookupTable?: boolean;
  verbose?: boolean;
  txLogs?: boolean;
  signers?: anchor.web3.Keypair[];
};

/**
 * Todo: add renderers to make these AccountMetas
 */
type TransferBaseAccounts = {
  authority?: anchor.web3.PublicKey;
  object: anchor.web3.PublicKey;
  destination: anchor.web3.PublicKey;
};

export async function callTransferOnBase(
  connection: anchor.web3.Connection,
  programId: anchor.web3.PublicKey,
  ixName: string,
  accounts: TransferBaseAccounts,
  opts?: CallOpts
) {
  return await call(
    connection,
    programId,
    ixName,
    [
      {
        pubkey: accounts.authority ?? getLocalKp().publicKey,
        isSigner: true,
        isWritable: false,
      },
      { pubkey: accounts.object, isSigner: false, isWritable: true },
    ],
    accounts.destination.toBuffer(),
    opts
  );
}

type TransferDelegateAccounts = {
  /**
   * This is the program id that the object
   * was issued by
   */
  programId: anchor.web3.PublicKey;
} & TransferBaseAccounts;

export async function callTransferOnDelegate(
  connection: anchor.web3.Connection,
  delegateProgramId: anchor.web3.PublicKey,
  accounts: TransferDelegateAccounts,
  opts?: CallOpts
) {
  return await call(
    connection,
    delegateProgramId,
    "transfer",
    [
      { pubkey: accounts.programId, isSigner: false, isWritable: false },
      {
        pubkey: accounts.authority ?? getLocalKp().publicKey,
        isSigner: true,
        isWritable: false,
      },
      { pubkey: accounts.object, isSigner: false, isWritable: true },
      { pubkey: accounts.destination, isSigner: false, isWritable: false },
    ],
    Buffer.from([]),
    opts
  );
}

type TransferSuperDelegateAccounts = {
  /**
   * This is the program id that will invoke
   * the transfer on the base program
   */
  delegateProgramId: anchor.web3.PublicKey;
} & TransferDelegateAccounts;

export async function callTransferOnSuperDelegate(
  connection: anchor.web3.Connection,
  superDelegateProgramId: anchor.web3.PublicKey,
  accounts: TransferSuperDelegateAccounts,
  opts?: CallOpts
) {
  return await call(
    connection,
    superDelegateProgramId,
    "transfer",
    [
      {
        pubkey: accounts.delegateProgramId,
        isSigner: false,
        isWritable: false,
      },
      { pubkey: accounts.programId, isSigner: false, isWritable: false },
      {
        pubkey: accounts.authority ?? getLocalKp().publicKey,
        isSigner: true,
        isWritable: false,
      },
      { pubkey: accounts.object, isSigner: false, isWritable: true },
      { pubkey: accounts.destination, isSigner: false, isWritable: false },
    ],
    Buffer.from([]),
    opts
  );
}

type SwapAccounts = {
  programId: anchor.web3.PublicKey;
  ownerA: anchor.web3.PublicKey;
  objectA: anchor.web3.PublicKey;
  ownerB: anchor.web3.PublicKey;
  objectB: anchor.web3.PublicKey;
};

export async function callSwapOnDelegate(
  connection: anchor.web3.Connection,
  delegateProgramId: anchor.web3.PublicKey,
  accounts: SwapAccounts,
  opts?: CallOpts
) {
  return await call(
    connection,
    delegateProgramId,
    "swap",
    [
      { pubkey: accounts.programId, isSigner: false, isWritable: false },
      {
        pubkey: accounts.ownerA ?? getLocalKp().publicKey,
        isSigner: true,
        isWritable: false,
      },
      { pubkey: accounts.objectA, isSigner: false, isWritable: true },
      { pubkey: accounts.ownerB, isSigner: true, isWritable: false },
      { pubkey: accounts.objectB, isSigner: false, isWritable: true },
    ],
    Buffer.from([]),
    opts
  );
}

export async function call(
  connection: anchor.web3.Connection,
  programId: anchor.web3.PublicKey,
  ixName: string,
  accounts: anchor.web3.AccountMeta[],
  args: Buffer,
  opts?: CallOpts
) {
  const data = Buffer.concat([hashIxName(ixName), args]);

  let ix: anchor.web3.TransactionInstruction =
    new anchor.web3.TransactionInstruction({
      programId,
      data,
      keys: accounts,
    });

  const { ix: _ix, lookupTable } = await additionalAccountsRequest(
    connection,
    ix,
    ixName,
    opts?.verbose ?? false,
    opts?.useLookupTable ?? false
  );
  ix = _ix;

  const computeUnits = (
    await sendTransaction(connection, [ix], {
      lookupTableAddress: lookupTable,
      verbose: opts?.verbose,
      logs: opts?.txLogs,
      signers: opts?.signers,
    })
  ).computeUnits;
  return computeUnits;
}
