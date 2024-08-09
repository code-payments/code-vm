import * as anchor from "@coral-xyz/anchor";
import { TestSandbox, testSandbox } from "./env";

export type TestVm = TestSandbox & {
    vm: anchor.web3.PublicKey;
    vm_bump: number;
    omnibus: anchor.web3.PublicKey;

    getState: () => Promise<any>;
    getCurrentPoh: () => Promise<anchor.web3.PublicKey>;
};


async function testVm() : Promise<TestVm> {
    const sandbox = await testSandbox();
    const { program, connection, signer, mint, lock_duration } = sandbox;

    const vm_authority = signer;
    const [vm, vm_bump] = anchor.web3.PublicKey.findProgramAddressSync(
        [
            Buffer.from("code-vm"),
            mint.toBuffer(),
            vm_authority.publicKey.toBuffer(),
            Buffer.from([lock_duration])
        ],
        program.programId
    )

    const [omnibus,] = anchor.web3.PublicKey.findProgramAddressSync(
        [
            Buffer.from("code-vm"),
            Buffer.from("vm_omnibus"),
            mint.toBuffer(),
            vm_authority.publicKey.toBuffer(),
            Buffer.from([lock_duration])
        ],
        program.programId
    )

    const tx = await program.methods
        .vmInit(
            lock_duration,
        )
        .accounts({
           vmAuthority: vm_authority.publicKey,
           mint: mint,
        })
        .signers([signer])
        .rpc({ skipPreflight: true });

    await connection.confirmTransaction(tx);

    console.log("tx", tx);

    const getState = async () => {
        return await program.account.codeVmAccount.fetch(vm);
    }

    const getCurrentPoh = async () => {
        const state = await getState();
        const buf = Buffer.from(state.poh.value);
        return new anchor.web3.PublicKey(buf); 
    }

    return { ...sandbox, vm, vm_bump, omnibus, getState, getCurrentPoh, };
}

export {
    testVm,
}