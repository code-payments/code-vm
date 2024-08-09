import * as anchor from "@coral-xyz/anchor";
import { createMint } from "@solana/spl-token";
import { Program } from "@coral-xyz/anchor";
import { CodeVm } from "../../target/types/code_vm";

export type TestEnv = {
    program: Program<CodeVm>;
    connection: anchor.web3.Connection;
};

export type TestSandbox = TestEnv & {
    signer: anchor.web3.Keypair;
    mint: anchor.web3.PublicKey;
    mint_key: anchor.web3.Keypair;
    lock_duration: number;
    lut?: anchor.web3.AddressLookupTableAccount;
};

async function testEnv() : Promise<TestEnv> {
    anchor.setProvider(anchor.AnchorProvider.env());
    const program = anchor.workspace.CodeVm as Program<CodeVm>;
    const connection = anchor.getProvider().connection;
    return { program, connection };
}

async function testSandbox() : Promise<TestSandbox> {
    const { program, connection } = await testEnv();

    const signer = anchor.web3.Keypair.generate();
    const mint_key = anchor.web3.Keypair.generate();
    const mint = mint_key.publicKey;
    const lock_duration = 21;

    // Request SOL airdrop
    const tx = await connection.requestAirdrop(signer.publicKey, 1000 * anchor.web3.LAMPORTS_PER_SOL);
    await connection.confirmTransaction(tx);
    await createMint(connection, signer, signer.publicKey, null, 5, mint_key);

    return { program, connection, signer, mint, mint_key, lock_duration };
}


export {
    testEnv,
    testSandbox,
}