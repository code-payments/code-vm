import * as anchor from "@coral-xyz/anchor";
import * as ed25519 from '@noble/ed25519';
import { sha512 } from '@noble/hashes/sha512';
import { TestVm } from "./vm";
import { VirtualAccount, VirtualDurableNonce, VirtualTimelockAccount } from "../memory/accounts";
import { TestMemory } from "./memory";
import { MerkleTree } from "./merkle-tree";
import { hash } from "./hash";
import { getAddressesFor } from "./user";

ed25519.etc.sha512Sync = (...m) => sha512(ed25519.etc.concatBytes(...m));

export type TestStorage = {
    address: anchor.web3.PublicKey;
    bump: number;

    numLevels: number;
    name: string;

    memoryState: MerkleTree;
    compressed: Map<string, {va: VirtualAccount, sig: Buffer}>;

    getState: () => Promise<any>;
    getRoot: () => Promise<Buffer>;
    getFilledSubtrees: () => Promise<Buffer[]>;
    getZeroValues: () => Promise<Buffer[]>;
};

async function createStorage(env: TestVm, name: string) : Promise<TestStorage> {
    const { program, connection, signer, vm } = env;

    const vm_authority = signer;

    // chosend arbitrarily for testing but needs to be small enough for proofs
    // to fit in a single transaction
    const levels = 8; 

    const [vm_storage_account, bump] = anchor.web3.PublicKey.findProgramAddressSync(
        [
            Buffer.from("code-vm"),
            Buffer.from("vm_storage_account"),
            Buffer.from(name),
            vm.toBuffer(),
        ],
        program.programId
    )

    const tx = await program.methods
        .vmStorageInit(
            name,
            levels,
        )
        .accountsPartial({
            vm: vm,
            vmAuthority: vm_authority.publicKey,
            vmStorageAccount: vm_storage_account,
        })
        .signers([signer])
        .rpc({ skipPreflight: false });

    await connection.confirmTransaction(tx);

    console.log("tx", tx);

    const seeds = Buffer.concat([
        Buffer.from("merkletree"),
        Buffer.from(name),
        vm.toBuffer(),
    ])
    const merkle_tree = new MerkleTree(seeds, levels);

    const getState = async () => {
        return await program.account.compressedStorageAccount.fetch(vm_storage_account);
    }

    const getRoot = async () => {
        const state = await getState();
        return Buffer.from(state.memoryState.root.value);
    }

    const getFilledSubtrees = async () => {
        const state = await getState();
        return state.memoryState.filledSubtrees.map(x => Buffer.from(x.value));
    }

    const getZeroValues = async () => {
        const state = await getState();
        return state.memoryState.zeroValues.map(x => Buffer.from(x.value));
    }

    return { 
        address: vm_storage_account, 
        numLevels: levels,

        memoryState: merkle_tree,

        name,
        bump,

        compressed: new Map<string, {va: VirtualAccount, sig: Buffer}>(),

        getState,
        getRoot,
        getFilledSubtrees,
        getZeroValues,
    };
}

async function compressVirtualAccount(
    env: TestVm, memory: TestMemory, memory_index: number, storage: TestStorage) {

    const { program, connection, signer, vm } = env;
    const vm_authority = signer;

    const va = await memory.getAccount(memory_index);
    const va_data = va.pack();
    const va_hash = hash(va_data);
    const sig = Buffer.from(ed25519.sign(va_hash, env.signer.secretKey.subarray(0, 32)));
    const sig_hash = hash(Buffer.concat([sig, va_hash]));

    storage.compressed.set(va_hash.toString('hex'), {va, sig});
    storage.memoryState.insert(sig_hash);

    const tx = await program.methods
        .systemAccountCompress(
            memory_index,
            { value: [...sig] },
        )
        .accountsPartial({
            vm: vm,
            vmAuthority: vm_authority.publicKey,
            vmMemory: memory.vm_memory_account,
            vmStorage: storage.address,
        })
        .signers([signer])
        .rpc({ skipPreflight: false });

    await connection.confirmTransaction(tx);

    console.log("tx", tx);

    return va_hash;
}

async function decompressVirtualAccount(
    env: TestVm, 
    memory: TestMemory, 
    memory_index: number, 
    storage: TestStorage, 
    va_hash: Buffer,
    ) {

    const { program, connection, signer, vm } = env;
    const vm_authority = signer;

    const info = storage.compressed.get(va_hash.toString('hex'));
    if (!info) {
        throw new Error("Account not compressed");
    }

    const sig_hash = hash(Buffer.concat([info.sig, va_hash]));
    const proof = storage.memoryState.getProofFor(sig_hash);
    storage.memoryState.remove(proof, sig_hash);

    let unlockPda : anchor.web3.PublicKey | null = null;
    let withdrawReceipt : anchor.web3.PublicKey | null = null;

    if (info.va.variant == 1) { // Timelock accounts need additional accounts to decompress
        const vta = (info.va.account as VirtualTimelockAccount);
        const { unlock_pda, } = getAddressesFor(env, vta.owner);
        const [ withdraw_receipt, ] = anchor.web3.PublicKey.findProgramAddressSync(
            [
                Buffer.from("code-vm"),
                Buffer.from("vm_withdraw_receipt_account"),
                unlock_pda.toBuffer(),
                vta.nonce.toBuffer(),
                vm.toBuffer(),
            ],
            program.programId
        )

        unlockPda = unlock_pda;
        withdrawReceipt = withdraw_receipt;
    }

    const tx = await program.methods
        .systemAccountDecompress(
            memory_index,

            // The account we are decompressing
            VirtualAccount.pack(info.va), 

            // The merkle proof for this account
            [ ...proof.map((v) => ({ value: [...v] })) ], 

            // The original signature
            { value: [...info.sig] },
        )
        .accountsPartial({
            vm: vm,
            vmAuthority: vm_authority.publicKey,

            vmMemory: memory.vm_memory_account,
            vmStorage: storage.address,

            unlockPda,
            withdrawReceipt,
        })
        .signers([signer])
        .rpc({ skipPreflight: false });

    await connection.confirmTransaction(tx);

    console.log("tx", tx);
}


export {
    createStorage,

    compressVirtualAccount,
    decompressVirtualAccount,
    
}