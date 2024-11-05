import { describe, it } from "vitest";
import * as anchor from "@coral-xyz/anchor";
import { PublicKey, Keypair, Connection, LAMPORTS_PER_SOL } from '@solana/web3.js'
import { CodeVm } from "../target/types/code_vm";
import { createMint } from "@solana/spl-token";

describe("vm-code", async () => {
  it("should run an instruction using the IDL", async () => {
    const { program, mint, signer } = await testSandbox();

    const [vmAddress, vmBump] = await PublicKey.findProgramAddress(
      [
        Buffer.from('code_vm'),
        mint.toBuffer(),
        signer.publicKey.toBuffer(),
        Buffer.from([21]),
      ],
      program.programId
    )

    const [OmnibusAddress, vmOmnibusBump] = await PublicKey.findProgramAddress(
      [
        Buffer.from('code_vm'),
        Buffer.from('vm_omnibus'),
        vmAddress.toBuffer()
      ],
      program.programId
    )

    const tx = await program.methods.initVm(
      { 
        lockDuration: 21,
        vmBump,
        vmOmnibusBump,
      }
    ).accountsPartial({
        vmAuthority: signer.publicKey,
        vm: vmAddress,
        omnibus: OmnibusAddress,
        mint,
    })
    .signers([signer])
    .rpc({ skipPreflight: true });

    console.log("Your transaction signature", tx);
  });
});

export type TestEnv = {
    program: anchor.Program<CodeVm>;
    connection: Connection;
};

export type TestSandbox = TestEnv & {
    signer: Keypair;
    mint: PublicKey;
    mint_key: Keypair;
    lock_duration: number;
};

async function testEnv() : Promise<TestEnv> {
    // Configure the client to use the local cluster and a provided wallet
    // keypair. We do this because "anchor test" runs a script to build the IDL,
    // this would fail because our discriminator values are not correct.

    process.env.ANCHOR_WALLET = "keypair-owner.json";
    const provider = anchor.AnchorProvider.local();
    anchor.setProvider(provider);

    const connection = anchor.getProvider().connection;
    const program = anchor.workspace.CodeVm as anchor.Program<CodeVm>;
    return { program, connection };
}

async function testSandbox() : Promise<TestSandbox> {
    const { program, connection } = await testEnv();

    const signer = Keypair.generate();
    const mint_key = Keypair.generate();
    const mint = mint_key.publicKey;
    const lock_duration = 21;

    const tx = await connection.requestAirdrop(signer.publicKey, 1000 * LAMPORTS_PER_SOL);
    await connection.confirmTransaction(tx);
    await createMint(connection, signer, signer.publicKey, null, 5, mint_key);

    return { program, connection, signer, mint, mint_key, lock_duration };
}