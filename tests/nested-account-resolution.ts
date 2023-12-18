import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { CallerWrapper } from "../target/types/caller_wrapper";
import { Callee } from "../target/types/callee";
import { assert } from "chai";
import { additionalAccountsRequest } from "./lib/additionalAccountsRequest";
import { Caller } from "../target/types/caller";
import { PRE_INSTRUCTIONS, sendTransaction } from "./lib/sendTransaction";
import { sha256 } from "@noble/hashes/sha256";

type ObjectCreationMeta = {
  metas: anchor.web3.AccountMeta[];
  signers: anchor.web3.Keypair[];
};

async function createOwnershipList(
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

async function validateLinkedListTransfer(
  program: anchor.Program<Callee>,
  nodeKps: anchor.web3.Keypair[],
  numNodes: number,
  destination: anchor.web3.PublicKey
) {
  let nodes = await program.account.node.fetchMultiple(
    nodeKps.map((kp) => kp.publicKey),
    "confirmed"
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

async function validateOwnershipListTransfer(
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

describe("nested-account-resolution", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Callee as Program<Callee>;

  const caller = anchor.workspace.Caller as Program<Caller>;

  const callerWrapper = anchor.workspace
    .CallerWrapper as Program<CallerWrapper>;

  const provider = anchor.getProvider();
  const payer = provider.publicKey;

  describe.skip("Base costs", () => {
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

      await validateLinkedListTransfer(program, [nodeKp], 1, destination);
    });

    for (const i of [1, 2, 10]) {
      const NUM_NODES = i;
      describe(`With ${NUM_NODES} nodes`, () => {
        let headNode: anchor.web3.PublicKey;

        let nodeKps: anchor.web3.Keypair[] = [];
        let nodeMetas: anchor.web3.AccountMeta[] = [];
        beforeEach(async () => {
          const { metas, signers } = await createOwnershipList(
            program,
            NUM_NODES
          );
          nodeKps = signers;
          nodeMetas = metas;
          headNode = nodeKps[0].publicKey;
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

          await validateLinkedListTransfer(
            program,
            nodeKps,
            NUM_NODES,
            destination
          );
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
            })
          ).computeUnits;

          console.log({ num: NUM_NODES, computeUnits });

          await validateLinkedListTransfer(
            program,
            nodeKps,
            NUM_NODES,
            destination
          );
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

          await validateLinkedListTransfer(
            program,
            nodeKps,
            NUM_NODES,
            destination
          );
        });
      });
    }
  });
  describe("Ownership List tests", () => {
    // for (const i of [131, 200, 230]) {
    // for (const i of [125]) (works on devnet account resolution)
    for (const i of [1, 2, 31]) {
      const NUM_NODES = i;

      describe(`With ${NUM_NODES} nodes`, () => {
        let ownershipListKp: anchor.web3.Keypair;
        let ownershipList: anchor.web3.PublicKey;
        let destination: anchor.web3.PublicKey;
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

          destination = anchor.web3.Keypair.generate().publicKey;
        });

        it(`Can transfer an ownership list (${NUM_NODES})`, async () => {
          let ix = await program.methods
            .transferOwnershipList(destination)
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
            })
          ).computeUnits;

          console.log({ num: NUM_NODES, computeUnits });
          await validateOwnershipListTransfer(
            program,
            ownershipList,
            destination
          );
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
            })
          ).computeUnits;

          console.log({ num: NUM_NODES, computeUnits });
          await validateOwnershipListTransfer(
            program,
            ownershipList,
            destination
          );
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

          await validateOwnershipListTransfer(
            program,
            ownershipList,
            destination
          );
        });
      });
    }
  });

  describe("Swap tests", () => {
    let ownerBKp = anchor.web3.Keypair.generate();
    let ownerB = ownerBKp.publicKey;

    for (const i of [1, 2, 25]) {
      const NUM_NODES = i;

      describe(`With Ownership Lists Containing ${NUM_NODES} Accounts`, () => {
        let ownershipListKpA: anchor.web3.Keypair;
        let ownershipListA: anchor.web3.PublicKey;

        let ownershipListKpB: anchor.web3.Keypair;
        let ownershipListB: anchor.web3.PublicKey;
        beforeEach(async () => {
          await provider.connection.requestAirdrop(
            ownerB,
            1 * anchor.web3.LAMPORTS_PER_SOL
          );

          ownershipListKpA = anchor.web3.Keypair.generate();
          ownershipListA = ownershipListKpA.publicKey;

          await program.methods
            .createOwnershipList(NUM_NODES)
            .accounts({
              payer,
              ownershipList: ownershipListA,
            })
            .preInstructions(PRE_INSTRUCTIONS)
            .signers([ownershipListKpA])
            .rpc({ skipPreflight: false, commitment: "confirmed" });

          ownershipListKpB = anchor.web3.Keypair.generate();
          ownershipListB = ownershipListKpB.publicKey;

          await program.methods
            .createOwnershipList(NUM_NODES)
            .accounts({
              payer: ownerB,
              ownershipList: ownershipListB,
            })
            .preInstructions(PRE_INSTRUCTIONS)
            .signers([ownerBKp, ownershipListKpB])
            .rpc({ skipPreflight: false, commitment: "confirmed" });
        });

        it("Can swap ownership list for ownership list", async () => {
          // Now perform the thing
          let ix = await caller.methods
            .swap()
            .accounts({
              program: program.programId,
              objectA: ownershipListA,
              ownerA: payer,
              objectB: ownershipListB,
              ownerB: ownerB,
            })
            .instruction();

          const { ix: _ix, lookupTable } = await additionalAccountsRequest(
            caller,
            ix,
            "swap",
            false,
            true
          );
          ix = _ix;

          const computeUnits = (
            await sendTransaction(provider.connection, [ix], {
              lookupTableAddress: lookupTable,
              signers: [ownerBKp],
            })
          ).computeUnits;

          console.log({ num: NUM_NODES, computeUnits });
          await validateOwnershipListTransfer(program, ownershipListA, ownerB);
          await validateOwnershipListTransfer(program, ownershipListB, payer);
        });
      });
    }

    // We're throttled at 9 accounts per linkedlist because
    // we can only create a linked list of size 9 for ownerB's keypair.
    // In other LL tests we can do 10, but creating for ownerB requires addnl pubkey + sig
    for (const i of [1, 2, 9]) {
      const NUM_NODES = i;
      describe(`With Linked Lists Containing ${NUM_NODES} Accounts`, () => {
        let linkedListA: anchor.web3.PublicKey;
        let listAInfo: ObjectCreationMeta;

        let linkedListB: anchor.web3.PublicKey;
        let listBInfo: ObjectCreationMeta;
        beforeEach(async () => {
          await provider.connection.requestAirdrop(
            ownerB,
            1 * anchor.web3.LAMPORTS_PER_SOL
          );

          listAInfo = await createOwnershipList(program, NUM_NODES);
          linkedListA = listAInfo.signers[0].publicKey;

          listBInfo = await createOwnershipList(program, NUM_NODES, {
            payer: ownerBKp,
          });
          linkedListB = listBInfo.signers[0].publicKey;
        });

        it("Can swap linked list for linked list", async () => {
          let ix = await caller.methods
            .swap()
            .accounts({
              program: program.programId,
              objectA: linkedListA,
              ownerA: payer,
              objectB: linkedListB,
              ownerB: ownerB,
            })
            .instruction();

          const { ix: _ix, lookupTable } = await additionalAccountsRequest(
            caller,
            ix,
            "swap",
            false,
            true
          );
          ix = _ix;

          const computeUnits = (
            await sendTransaction(provider.connection, [ix], {
              lookupTableAddress: lookupTable,
              signers: [ownerBKp],
            })
          ).computeUnits;

          console.log({ num: NUM_NODES, computeUnits });
          await validateLinkedListTransfer(
            program,
            listAInfo.signers,
            NUM_NODES,
            ownerB
          );
          await validateLinkedListTransfer(
            program,
            listBInfo.signers,
            NUM_NODES,
            payer
          );
        });
      });
    }
  });
});
