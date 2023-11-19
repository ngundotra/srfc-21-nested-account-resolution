import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { CallerWrapper } from "../target/types/caller_wrapper";
import { Callee } from "../target/types/callee";
import { assert } from "chai";
import { additionalAccountsRequest } from "./additionalAccountsRequest";
import { Caller } from "../target/types/caller";
import { PRE_INSTRUCTIONS, sendTransaction } from "./sendTransaction";
import { sha256 } from "@noble/hashes/sha256";

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

describe("nested-account-resolution", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Callee as Program<Callee>;

  const caller = anchor.workspace.Caller as Program<Caller>;

  const callerWrapper = anchor.workspace
    .CallerWrapper as Program<CallerWrapper>;

  const provider = anchor.getProvider();
  const payer = provider.publicKey;

  describe("Base costs", () => {
    it("Return data 1024", async () => {
      async function getCost(amount: 0 | 512 | 1024) {
        let ix = await program.methods
          .returnData(amount)
          .accounts({})
          .instruction();
        return (await sendTransaction(provider.connection, [ix], {}))
          .computeUnits;
      }

      let costs = [await getCost(0), await getCost(512), await getCost(1024)];
      console.log(costs);
      console.log(
        `Deltas: ${(costs[1] - costs[0]) / 512}, ${
          (costs[2] - costs[1]) / 1024
        }`
      );
    });
    it("Return data with CPI", async () => {
      async function getCost(amount: 0 | 512 | 1024) {
        let ix = await caller.methods
          .returnData(amount)
          .accounts({
            program: program.programId,
          })
          .instruction();
        return (await sendTransaction(provider.connection, [ix], {}))
          .computeUnits;
      }

      let costs = [await getCost(0), await getCost(512), await getCost(1024)];
      console.log(costs);
      console.log(
        `Deltas: ${(costs[1] - costs[0]) / 512}, ${
          (costs[2] - costs[1]) / 1024
        }`
      );
    });
    // it("");
  });

  let destinationKp = anchor.web3.Keypair.generate();
  let destination = destinationKp.publicKey;
  describe("Linked list tests", () => {
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

      let node = await program.account.node.fetch(
        nodeKp.publicKey,
        "confirmed"
      );
      assert(node.next === null);

      let ix = await program.methods
        .transferLinkedList(destination)
        .accounts({
          headNode,
          owner: payer,
        })
        .instruction();

      const { ix: _ix, lookupTable } = await additionalAccountsRequest(
        program,
        ix,
        "transfer_linked_list"
      );
      ix = _ix;

      const computeUnits = (
        await sendTransaction(provider.connection, [ix], {
          lookupTableAddress: lookupTable,
        })
      ).computeUnits;
      console.log({ num: 1, computeUnits });

      await validateTransfer(program, [nodeKp], 1);
    });

    // for (let i = 2; i < 11; i++) {
    for (let i = 2; i < 3; i++) {
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

          const { ix: _ix, lookupTable } = await additionalAccountsRequest(
            program,
            ix,
            "transfer_linked_list"
          );
          ix = _ix;

          const computeUnits = (
            await sendTransaction(provider.connection, [ix], {
              lookupTableAddress: lookupTable,
            })
          ).computeUnits;
          console.log({ num: NUM_NODES, computeUnits });

          await validateTransfer(program, nodeKps, NUM_NODES);
        });

        it(`Can transfer a linked list (${NUM_NODES}) via CPI`, async () => {
          let ix = await caller.methods
            .transfer()
            .accounts({
              program: program.programId,
              object: headNode,
              owner: payer,
              destination,
            })
            .instruction();

          const { ix: _ix, lookupTable } = await additionalAccountsRequest(
            caller,
            ix,
            "transfer"
          );
          ix = _ix;

          const computeUnits = (
            await sendTransaction(provider.connection, [ix], {
              lookupTableAddress: lookupTable,
              verbose: true,
            })
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
              object: headNode,
              owner: payer,
              destination,
            })
            .instruction();

          const { ix: _ix, lookupTable } = await additionalAccountsRequest(
            callerWrapper,
            ix,
            "transfer"
          );
          ix = _ix;

          const computeUnits = (
            await sendTransaction(provider.connection, [ix], {
              lookupTableAddress: lookupTable,
            })
          ).computeUnits;

          console.log({ num: NUM_NODES, computeUnits });

          await validateTransfer(program, nodeKps, NUM_NODES);
        });
      });
    }
  });
  describe("Ownership List tests", () => {
    // for (let i = 31; i < ; i++) {
    // // for (let i = 31; i < ; i++) {
    // for (const i of [131, 200, 230]) {
    // for (const i of [125]) (works on devnet account resolution)
    // for (const i of [31]) {
    for (const i of [3]) {
      const NUM_NODES = i;

      describe(`With ${NUM_NODES} nodes`, () => {
        let ownershipListKp: anchor.web3.Keypair;
        let ownershipList: anchor.web3.PublicKey;
        beforeEach(async () => {
          ownershipListKp = anchor.web3.Keypair.generate();
          ownershipList = ownershipListKp.publicKey;

          await program.methods
            .createOwnershipList(NUM_NODES)
            .accounts({
              payer,
              ownershipList,
            })
            .preInstructions(PRE_INSTRUCTIONS)
            .signers([ownershipListKp])
            .rpc({ skipPreflight: false, commitment: "confirmed" });
        });

        it(`Can transfer an ownership list (${NUM_NODES})`, async () => {
          let ix = await program.methods
            .transferOwnershipList(ownershipList)
            .accounts({
              ownershipList,
              owner: payer,
            })
            .instruction();

          const { ix: _ix, lookupTable } = await additionalAccountsRequest(
            program,
            ix,
            "transfer_ownership_list",
            false,
            true
          );
          ix = _ix;

          const computeUnits = (
            await sendTransaction(provider.connection, [ix], {
              lookupTableAddress: lookupTable,
              verbose: true,
            })
          ).computeUnits;

          console.log({ num: NUM_NODES, computeUnits });
        });

        it(`Can transfer an ownership list (${NUM_NODES}) via CPI`, async () => {
          let ix = await caller.methods
            .transfer()
            .accounts({
              program: program.programId,
              object: ownershipList,
              owner: payer,
              destination,
            })
            .instruction();

          const { ix: _ix, lookupTable } = await additionalAccountsRequest(
            caller,
            ix,
            "transfer",
            false,
            true
          );
          ix = _ix;

          const computeUnits = (
            await sendTransaction(provider.connection, [ix], {
              lookupTableAddress: lookupTable,
              verbose: true,
            })
          ).computeUnits;

          console.log({ num: NUM_NODES, computeUnits });
        });

        it(`Can transfer an ownership list (${NUM_NODES}) via CPI-CPI`, async () => {
          // Now perform the thing
          let ix = await callerWrapper.methods
            .transfer()
            .accounts({
              delegateProgram: caller.programId,
              program: program.programId,
              object: ownershipList,
              owner: payer,
              destination,
            })
            .instruction();

          const { ix: _ix, lookupTable } = await additionalAccountsRequest(
            callerWrapper,
            ix,
            "transfer",
            false,
            true
          );
          ix = _ix;

          const computeUnits = (
            await sendTransaction(provider.connection, [ix], {
              lookupTableAddress: lookupTable,
            })
          ).computeUnits;

          console.log({ num: NUM_NODES, computeUnits });

          // await validateTransfer(program, nodeKps, NUM_NODES);
        });
      });
    }
  });
});
