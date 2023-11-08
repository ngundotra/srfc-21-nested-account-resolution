import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { NestedAccountResolution } from "../target/types/nested_account_resolution";
import { BenchmarkAarCallee } from "../target/types/benchmark_aar_callee";
import { assert } from "chai";
import {
  additionalAccountsRequest,
  resolveRemainingAccounts,
} from "./additionalAccountsRequest";
import { BenchmarkAarCaller } from "../target/types/benchmark_aar_caller";

describe("nested-account-resolution", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace
    .BenchmarkAarCallee as Program<BenchmarkAarCallee>;

  const caller = anchor.workspace
    .BenchmarkAarCaller as Program<BenchmarkAarCaller>;

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

    let tx = new anchor.web3.Transaction().add(ix);
    txid = await provider.sendAndConfirm(tx, [], {
      skipPreflight: true,
      commitment: "confirmed",
    });

    const txresp = await provider.connection.getTransaction(txid, {
      commitment: "confirmed",
    });

    const computeUnits = txresp.meta.computeUnitsConsumed;
    console.log({ num: 1, computeUnits });

    node = await program.account.node.fetch(headNode, "confirmed");
    assert(node.owner.toString() === destination.toString());
  });
  describe("With 5 nodes", () => {
    const NUM_NODES = 5;
    let headNode: anchor.web3.PublicKey;

    beforeEach(async () => {
      let nodeKps: anchor.web3.Keypair[] = [];
      let nodeMetas: anchor.web3.AccountMeta[] = [];
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
      let txid = await program.methods
        .createLinkedList(NUM_NODES)
        .accounts({ payer })
        .remainingAccounts(nodeMetas)
        .signers(nodeKps)
        .rpc({ skipPreflight: true, commitment: "confirmed" });

      let nodes = await program.account.node.fetchMultiple(
        nodeKps.map((kp) => kp.publicKey),
        "confirmed"
      );

      for (let i = 0; i < NUM_NODES - 1; i++) {
        assert(
          nodes[i].next.toString() === nodeKps[i + 1].publicKey.toString(),
          `${i}th node's next is not correct!`
        );
      }
      assert(nodes[NUM_NODES - 1].next === null);
    });

    it("Can transfer a linked list with 5 nodes", async () => {
      let ix = await program.methods
        .transferLinkedList(destination)
        .accounts({
          headNode,
          owner: payer,
        })
        .instruction();

      ix = await additionalAccountsRequest(program, ix, "transfer_linked_list");

      let tx = new anchor.web3.Transaction().add(ix);
      let txid = await provider.sendAndConfirm(tx, [], {
        skipPreflight: true,
        commitment: "confirmed",
      });

      const txresp = await provider.connection.getTransaction(txid, {
        commitment: "confirmed",
      });

      const computeUnits = txresp.meta.computeUnitsConsumed;
      console.log({ num: NUM_NODES, computeUnits });

      let node = await program.account.node.fetch(headNode, "confirmed");
      assert(node.owner.toString() === destination.toString());
      while (node.next !== null) {
        node = await program.account.node.fetch(node.next, "confirmed");
        assert(node.owner.toString() === destination.toString());
      }
    });

    it("Can transfer a linked list via CPI", async () => {
      // Now perform the thing
      let ix = await caller.methods
        .transfer()
        .accounts({
          program: program.programId,
          head: headNode,
          owner: payer,
          destination,
        })
        .instruction();

      ix = await additionalAccountsRequest(caller, ix, "transfer", true);

      let tx = new anchor.web3.Transaction().add(ix);

      let message = anchor.web3.MessageV0.compile({
        payerKey: program.provider.publicKey!,
        instructions: [ix],
        recentBlockhash: (
          await program.provider.connection.getRecentBlockhash()
        ).blockhash,
      });
      let transaction = new anchor.web3.VersionedTransaction(message);

      let simulationResult =
        await program.provider.connection.simulateTransaction(transaction, {
          commitment: "confirmed",
        });
      console.log({ logs: simulationResult.value.logs });

      let txid = await provider.sendAndConfirm(tx, [], {
        skipPreflight: true,
        commitment: "confirmed",
      });

      const txresp = await provider.connection.getTransaction(txid, {
        commitment: "confirmed",
      });

      const computeUnits = txresp.meta.computeUnitsConsumed;
      console.log({ num: NUM_NODES, computeUnits });

      let node = await program.account.node.fetch(headNode, "confirmed");
      assert(node.owner.toString() === destination.toString());
      while (node.next !== null) {
        node = await program.account.node.fetch(node.next, "confirmed");
        assert(node.owner.toString() === destination.toString());
      }
    });
  });
});
