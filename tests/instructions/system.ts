import { web3 } from "@coral-xyz/anchor";
import { token } from "@coral-xyz/anchor/dist/cjs/utils";
import { expect } from "chai";
import { CodeVM } from "../env";
import { createVirtualDurableNonce } from "../utils/nonce";
import * as util from "../utils/storage";
import { MerkleTree } from "../utils/merkle-tree";
import { TestStorage } from "../utils/storage";
import { VirtualDurableNonce } from "../memory/accounts";
import { createLut } from "../utils/lut";

export const getNonce = async (env: CodeVM, index: number) => {
    const va = await env.vm_memory.getAccount(index);
    expect(va).to.not.be.undefined;
    expect(va.account).to.not.be.undefined;
    expect(va.variant).to.be.eq(0);
    return va.account as VirtualDurableNonce;
};

export const checkNonce = async (vm: CodeVM, index: number) => {
    const expected = vm.nonces[index];
    const actual = await getNonce(vm, index);

    expect(actual.address.toBase58()).to.equal(expected.address.toBase58());
}

export async function test_create_nonces(env: CodeVM, count: number, offset: number): Promise<CodeVM> {
    const vm = { ...env, nonces : [] }

    for (let i = 0; i < count; i++) {
        const account_index = i + offset;
        vm.nonces[account_index] = await createVirtualDurableNonce(vm.vm, vm.vm_memory, account_index);
        await checkNonce(vm, account_index);
    }

    return vm;
}

export async function test_compress_nonce_account(
    env: CodeVM, vdn_index: number
): Promise<CodeVM> {

    const { vm } = env;
    const vdn = env.nonces[vdn_index]; // virtual nonce account

    const va_hash = await util.compressVirtualAccount(
        vm,
        env.vm_memory,
        vdn.index,
        env.store_accounts
    );

    vdn.data_hash = va_hash;

    await checkStorageState(env.store_accounts);

    return env;
}

export async function test_decompress_nonce_account(
    env: CodeVM, vdn_index: number, memory_index: number
): Promise<CodeVM> {

    const { vm } = env;
    const vdn = env.nonces[vdn_index]; // virtual token account

    if (!vdn.data_hash) {
        throw new Error("Account not compressed");
    }

    await util.decompressVirtualAccount(
        vm,
        env.vm_memory,
        memory_index,
        env.store_accounts,
        vdn.data_hash
    );

    // Update the memory index
    vdn.index = memory_index;
    vdn.data_hash = null;

    // move the account to the new memory index
    env.nonces[memory_index] = vdn;
    delete env.nonces[vdn_index];

    await checkStorageState(env.store_accounts);

    return env;
}

export async function test_compress_timelock_account(
    env: CodeVM, vta_index: number
): Promise<CodeVM> {

    const { vm } = env;
    const vta = env.accounts[vta_index]; // virtual token account

    const va_hash = await util.compressVirtualAccount(
        vm,
        env.vm_memory,
        vta.memory_index,
        env.store_accounts
    );

    await checkStorageState(env.store_accounts);

    vta.data_hash = va_hash;

    return env;
}

export async function test_decompress_timelock_account(
    env: CodeVM, vta_index: number, memory_index: number
): Promise<CodeVM> {

    const { vm } = env;
    const vta = env.accounts[vta_index]; // virtual token account

    if (!vta.data_hash) {
        throw new Error("Account not compressed");
    }

    await util.decompressVirtualAccount(
        vm,
        env.vm_memory,
        memory_index,
        env.store_accounts,
        vta.data_hash
    );

    // Update the memory index
    vta.memory_index = memory_index;
    vta.data_hash = null;

    // move the account to the new memory index
    env.accounts[memory_index] = vta;
    delete env.accounts[vta_index];

    await checkStorageState(env.store_accounts);

    return env;
}

export async function test_compress_relay_account(
    env: CodeVM, vra_index: number
): Promise<CodeVM> {
    
        const { vm } = env;
        const vra = env.relayProofs[vra_index]; // virtual relay account
    
        const va_hash = await util.compressVirtualAccount(
            vm,
            env.vm_memory,
            vra.index,
            env.store_accounts
        );
    
        await checkStorageState(env.store_accounts);
    
        vra.data_hash = va_hash;
    
        return env;

}

export async function test_decompress_relay_account(
    env: CodeVM, vra_index: number, memory_index: number
): Promise<CodeVM> {
    
        const { vm } = env;
        const vra = env.relayProofs[vra_index]; // virtual relay account
    
        if (!vra.data_hash) {
            throw new Error("Account not compressed");
        }
    
        await util.decompressVirtualAccount(
            vm,
            env.vm_memory,
            memory_index,
            env.store_accounts,
            vra.data_hash
        );
    
        // Update the memory index
        vra.index = memory_index;
        vra.data_hash = null;
    
        // move the account to the new memory index
        env.relayProofs[memory_index] = vra;
        delete env.relayProofs[vra_index];
    
        await checkStorageState(env.store_accounts);
    
        return env;
}

export async function test_create_lookup_table(env: CodeVM): Promise<CodeVM> {
    const lut = await createLut(env.vm, [
        env.vm.signer.publicKey,            // vm_authority
        env.vm.vm,                          // vm

        env.vm_memory.vm_memory_account,    // memory
        env.store_accounts.address,         // store

        env.relays[0].address,              // relay
        env.relays[0].vault,                // relay_vault
        env.vm.omnibus,                     // omnibus

        env.vm.mint,                        // mint
        token.TOKEN_PROGRAM_ID,             // token_program
        env.vm.program.programId,           // program_id
        web3.Ed25519Program.programId,      // ed25519_program
        web3.SystemProgram.programId,       // system_program
        web3.SYSVAR_RENT_PUBKEY,            // rent_sysvar
    ]);

    console.log("lut", lut);

    env.vm.lut = lut;

    return env;
}

async function checkStorageState(store: TestStorage) {
    const seeds = store.memoryState.seeds;
    const expected = store.memoryState;
    const actual = MerkleTree.from((
        await store.getState()
    ).memoryState, seeds);

    console.log("local\n", expected.toString());
    console.log("onchain\n", actual.toString());

    expect(expected.root.toString('hex')).to.equal(actual.root.toString('hex'));
    expect(expected.toString()).to.equal(actual.toString());
}
