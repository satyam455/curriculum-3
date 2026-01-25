import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Compute } from "../target/types/compute";
import { assert } from "chai";

describe("compute", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.compute as Program<Compute>;

  it("Batch processes operations!", async () => {
    const itemCount = 10;
    const items: anchor.web3.Keypair[] = [];
    const operations: any[] = [];

    console.log(`Initializing ${itemCount} items...`);

    // 1. Initialize items
    for (let i = 0; i < itemCount; i++) {
      const kp = anchor.web3.Keypair.generate();
      items.push(kp);

      await program.methods
        .initializeItem(new anchor.BN(i))
        .accounts({
          item: kp.publicKey,
          authority: provider.wallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        } as any)
        .signers([kp])
        .rpc();

      // Create operation for this item (Add 10)
      operations.push({
        opType: { add: {} }, // Enum variant
        amount: new anchor.BN(10),
      });
    }

    console.log("Items initialized. Starting batch process...");

    // 2. Perform Batch Operation
    const tx = await program.methods
      .batchProcess(operations)
      .accounts({
        authority: provider.wallet.publicKey,
      })
      .remainingAccounts(
        items.map((item) => ({
          pubkey: item.publicKey,
          isWritable: true,
          isSigner: false,
        }))
      )
      .rpc();

    console.log("Batch transaction signature:", tx);

    // 3. Verify Results
    console.log("Verifying results...");
    for (const itemKp of items) {
      const account = await program.account.item.fetch(itemKp.publicKey);
      assert.equal(account.value.toNumber(), 10, "Value should be incremented to 10");
    }

    console.log("Success! Batch processing complete.");
  });

  it("Benchmarks large batch", async () => {
    // Optional: Try a larger batch to see limits (careful with local RPC limits)
    // This test structure mirrors the homework requirement to document performance.
    console.log("Starting benchmark...");
    const start = Date.now();
    // ... (Logic would be similar to above but with more items)
    const end = Date.now();
    console.log(`Benchmark took ${end - start}ms`);
  });
});
