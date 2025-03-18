//TDOO: edit test file
import * as anchor from "@coral-xyz/anchor";
import BN from "bn.js";
import assert from "assert";
import * as web3 from "@solana/web3.js";
import type { FlashPerpetualsToken } from "../target/types/flash_perpetuals_token";

describe("flash-perpetuals-token", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.FlashPerpetualsToken as anchor.Program<FlashPerpetualsToken>;

  it("initializes global state", async () => {
    // Generate a keypair for the GlobalState account.
    const globalStateKp = web3.Keypair.generate();

    // Define initialization parameters.
    const feeRate = new BN(10);
    const maintenanceMargin = new BN(150); // 150%
    const minStakeDuration = new BN(60);   // in seconds
    const executionFee = new BN(1);

    // Call the initialize method.
    const txHash = await program.methods
      .initialize(feeRate, maintenanceMargin, minStakeDuration, executionFee)
      .accounts({
        globalState: globalStateKp.publicKey,
        admin: program.provider.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([globalStateKp])
      .rpc();

    console.log(`Transaction hash: ${txHash}`);
    await program.provider.connection.confirmTransaction(txHash);

    // Fetch the GlobalState account.
    const globalStateAccount = await program.account.globalState.fetch(globalStateKp.publicKey);
    console.log("GlobalState account data:", globalStateAccount);

    // Verify that the on-chain data matches the initialization parameters.
    assert.ok(globalStateAccount.admin.equals(program.provider.publicKey));
    assert.ok(new BN(globalStateAccount.feeRate).eq(feeRate));
    assert.ok(new BN(globalStateAccount.maintenanceMargin).eq(maintenanceMargin));
    assert.ok(new BN(globalStateAccount.minStakeDuration).eq(minStakeDuration));
    assert.ok(new BN(globalStateAccount.executionFee).eq(executionFee));
  });
});
