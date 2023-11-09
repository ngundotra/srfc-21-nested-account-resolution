import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { CallerWrapper } from "../target/types/caller_wrapper";
import { Callee } from "../target/types/callee";
import { assert } from "chai";
import { additionalAccountsRequest } from "./additionalAccountsRequest";
import { Caller } from "../target/types/caller";
import { readFileSync } from "fs";
import { homedir } from "os";

async function validateTransfer(
  program: anchor.Program<Callee>,
  nodeKps: anchor.web3.Keypair[],
  numNodes: number
) {
  let nodes = await program.account.node.fetchMultiple(
    nodeKps.map((kp) => kp.publicKey),
    "confirmed"
  );

  for (let i = 0; i < numNodes - 1; i++) {
    assert(
      nodes[i].next.toString() === nodeKps[i + 1].publicKey.toString(),
      `${i}th node's next is not correct!`
    );
  }
  assert(nodes[numNodes - 1].next === null);
}

async function sendTransaction(
  connection: anchor.web3.Connection,
  ixs: anchor.web3.TransactionInstruction[],
  logs: boolean = false,
  simulate: boolean = false
): Promise<{ computeUnits: number }> {
  let kp = anchor.web3.Keypair.fromSecretKey(
    Buffer.from(
      JSON.parse(readFileSync(`${homedir()}/.config/solana/id.json`).toString())
    )
  );
  let message = anchor.web3.MessageV0.compile({
    payerKey: kp.publicKey,
    instructions: [
      anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
        units: 1_400_000,
      }),
    ].concat(ixs),
    recentBlockhash: (await connection.getRecentBlockhash()).blockhash,
  });
  let transaction = new anchor.web3.VersionedTransaction(message);

  if (simulate) {
    let simulationResult = await connection.simulateTransaction(transaction, {
      commitment: "confirmed",
    });

    if (logs) {
      console.log(simulationResult.value.logs.join("\n"));
    }

    return { computeUnits: simulationResult.value.unitsConsumed };
  } else {
    transaction.sign([kp]);

    let txid = await connection.sendRawTransaction(transaction.serialize(), {
      skipPreflight: true,
    });
    await connection.confirmTransaction(txid, "confirmed");

    const txresp = await connection.getTransaction(txid, {
      commitment: "confirmed",
      maxSupportedTransactionVersion: 2,
    });

    if (logs || txresp.meta.err) {
      console.log(txresp.meta.logMessages.join("\n"));
    }
    if (txresp.meta.err) {
      throw new Error(txresp.meta.err.toString());
    }

    return { computeUnits: txresp.meta.computeUnitsConsumed };
  }
}

describe("nested-account-resolution", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Callee as Program<Callee>;

  const caller = anchor.workspace.Caller as Program<Caller>;

  const callerWrapper = anchor.workspace
    .CallerWrapper as Program<CallerWrapper>;

  const provider = anchor.getProvider();
  const payer = provider.publicKey;

  let destinationKp = anchor.web3.Keypair.generate();
  let destination = destinationKp.publicKey;

  it("Can initialize a linked list with 1 node", async () => {
    const nodeKp = anchor.web3.Keypair.generate();
    let headNode = nodeKp.publicKey;
    let txid = await program.methods
      .createLinkedList(1)
      .accounts({ payer })
      .remainingAccounts([
        { pubkey: headNode, isWritable: true, isSigner: true },
      ])
      .signers([nodeKp])
      .rpc({ skipPreflight: true, commitment: "confirmed" });

    let node = await program.account.node.fetch(nodeKp.publicKey, "confirmed");
    assert(node.next === null);

    let ix = await program.methods
      .transferLinkedList(destination)
      .accounts({
        headNode,
        owner: payer,
      })
      .instruction();

    ix = await additionalAccountsRequest(program, ix, "transfer_linked_list");

    const computeUnits = (
      await sendTransaction(provider.connection, [ix], false)
    ).computeUnits;
    console.log({ num: 1, computeUnits });

    await validateTransfer(program, [nodeKp], 1);
  });

  for (let i = 2; i < 12; i++) {
    const NUM_NODES = i;
    describe(`With ${NUM_NODES} nodes`, () => {
      let headNode: anchor.web3.PublicKey;

      let nodeKps: anchor.web3.Keypair[] = [];
      let nodeMetas: anchor.web3.AccountMeta[] = [];
      beforeEach(async () => {
        nodeKps = [];
        nodeMetas = [];
        for (let i = 0; i < NUM_NODES; i++) {
          let kp = anchor.web3.Keypair.generate();
          nodeKps.push(kp);
          nodeMetas.push({
            pubkey: kp.publicKey,
            isWritable: true,
            isSigner: true,
          });
        }
        headNode = nodeKps[0].publicKey;
        await program.methods
          .createLinkedList(NUM_NODES)
          .accounts({ payer })
          .remainingAccounts(nodeMetas)
          .signers(nodeKps)
          .rpc({ skipPreflight: true, commitment: "confirmed" });
      });

      it(`Can transfer a linked list (${NUM_NODES})`, async () => {
        let ix = await program.methods
          .transferLinkedList(destination)
          .accounts({
            headNode,
            owner: payer,
          })
          .instruction();

        ix = await additionalAccountsRequest(
          program,
          ix,
          "transfer_linked_list"
        );

        const computeUnits = (
          await sendTransaction(provider.connection, [ix], false)
        ).computeUnits;
        console.log({ num: NUM_NODES, computeUnits });

        await validateTransfer(program, nodeKps, NUM_NODES);
      });

      it(`Can transfer a linked list (${NUM_NODES}) via CPI`, async () => {
        let ix = await caller.methods
          .transfer()
          .accounts({
            program: program.programId,
            head: headNode,
            owner: payer,
            destination,
          })
          .instruction();

        ix = await additionalAccountsRequest(caller, ix, "transfer");

        const computeUnits = (
          await sendTransaction(provider.connection, [ix], false)
        ).computeUnits;

        console.log({ num: NUM_NODES, computeUnits });

        await validateTransfer(program, nodeKps, NUM_NODES);
      });

      it(`Can transfer a linked list (${NUM_NODES}) via CPI-CPI`, async () => {
        // Now perform the thing
        let ix = await callerWrapper.methods
          .transfer()
          .accounts({
            delegateProgram: caller.programId,
            program: program.programId,
            head: headNode,
            owner: payer,
            destination,
          })
          .instruction();

        ix = await additionalAccountsRequest(callerWrapper, ix, "transfer");

        const computeUnits = (
          await sendTransaction(provider.connection, [ix], false)
        ).computeUnits;

        console.log({ num: NUM_NODES, computeUnits });

        await validateTransfer(program, nodeKps, NUM_NODES);
      });
    });
  }
});
