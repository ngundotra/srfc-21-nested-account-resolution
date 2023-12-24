import { startAnchor } from "solana-bankrun";
import { BankrunProvider } from "anchor-bankrun";
import { Callee, IDL as CalleeIDL } from "../target/types/callee";
import * as anchor from "@coral-xyz/anchor";
import { parse } from "toml";
import { readFileSync } from "fs";
import { join } from "path";
import { getLocalKp } from "./lib/sendTransaction";
import { assert } from "chai";
import { call, callTransferOnBase } from "./lib/interface";
import {
  createLinkedList,
  setupBankrun,
  validateLinkedListTransfer,
} from "./lib/utils";
import {
  GLOBAL_CONTEXT,
  setGlobalContext,
} from "./lib/additionalAccountsRequest";

test("Test Bankrun", async () => {
  const { callee, caller, callerWrapper, provider, context } =
    await setupBankrun();

  let headNode: anchor.web3.PublicKey;

  let nodeKps: anchor.web3.Keypair[] = [];
  let nodeMetas: anchor.web3.AccountMeta[] = [];

  const NUM_NODES = 5;
  const { metas, signers } = await createLinkedList(callee, NUM_NODES);
  nodeKps = signers;
  nodeMetas = metas;
  headNode = nodeKps[0].publicKey;
  const destination = anchor.web3.Keypair.generate().publicKey;
  const computeUnits = await callTransferOnBase(
    provider.connection,
    callee.programId,
    "transfer_linked_list",
    {
      object: headNode,
      destination,
    }
  );
  console.log({ num: NUM_NODES, computeUnits });

  await validateLinkedListTransfer(callee, nodeKps, NUM_NODES, destination);
});
