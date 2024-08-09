import * as anchor from "@coral-xyz/anchor";
import { TestVm } from "./vm";
import { MerkleTree } from "./merkle-tree";
import { TestMemory } from "./memory";
import { TestUser } from "./user";
import { OpCode, serialize } from "./opcode";
import { vm_relay_exec } from "./exec";
import { VirtualRelayAccount } from "../memory/accounts";

export type TestRelay = {
    address: anchor.web3.PublicKey;
    bump: number;
    vault: anchor.web3.PublicKey;

    numLevels: number;
    numHistory: number;
    name: string;

    history: MerkleTree;
    data_hash?: Buffer;

    getState: () => Promise<any>;
    getRoot: () => Promise<Buffer>;
    getRecentRoot: () => Promise<Buffer>;
};

export type TestRelayProof = {
    address: anchor.web3.PublicKey;
    commitment: anchor.web3.PublicKey;
    recentRoot: anchor.web3.PublicKey;
    destination: anchor.web3.PublicKey;
    index: number;
    data_hash?: Buffer;

    getInfo: () => Promise<VirtualRelayAccount>;
};

async function createRelay(env: TestVm, name: string) : Promise<TestRelay> {
    const { program, connection, signer, vm, mint } = env;

    const vm_authority = signer;

    const levels = 8;
    const history = 32;

    const [relay_address, relay_bump] = anchor.web3.PublicKey.findProgramAddressSync(
        [
            Buffer.from("code-vm"),
            Buffer.from("vm_relay_account"),
            Buffer.from(name),
            vm.toBuffer(),
        ],
        program.programId
    )

    const [relay_vault,] = anchor.web3.PublicKey.findProgramAddressSync(
        [
            Buffer.from("code-vm"),
            Buffer.from("vm_relay_vault"),
            relay_address.toBuffer(),
        ],
        program.programId
    )

    const tx = await program.methods
        .relayInit(
            levels,
            history,
            name,
        )
        .accountsPartial({
            vm,
            vmAuthority: vm_authority.publicKey,
            relay: relay_address,
            relayVault: relay_vault,
            mint,
        })
        .signers([signer])
        .rpc({ skipPreflight: false });

    await connection.confirmTransaction(tx);

    console.log("tx", tx);

    const seeds = Buffer.concat([
        Buffer.from("merkletree"),
        relay_address.toBytes(),
    ])
    const merkle_tree = new MerkleTree(seeds, levels);

    const getState = async () => {
        return await program.account.relayAccount.fetch(relay_address);
    }

    const getRoot = async () => {
        const state = await getState();
        return Buffer.from(state.history.root.value);
    }

    const getRecentRoot = async () => {
        const state = await getState();
        const index = state.recentRoots.offset;
        return Buffer.from(state.recentRoots.items[index]);
    }

    return { 
        address: relay_address, 
        bump: relay_bump, 
        vault: relay_vault,

        numLevels: levels,
        numHistory: history,
        name: name,

        history: merkle_tree,

        getState,
        getRoot,
        getRecentRoot,
    };
}

async function getRelayProof(
    mem: TestMemory,
    index: number,
) : Promise<TestRelayProof> {

    const getInfo = async () => {
        return (await mem.getAccount(index)).account as VirtualRelayAccount;
    }

    const info = await getInfo();

    return {
        address: info.address,
        commitment: info.commitment,
        recentRoot: info.recentRoot,
        destination: info.destination,
        index: index,
        getInfo,
    };
}

async function saveRelayRoot(env: TestVm, relay: TestRelay) {
    const { program, connection, signer, vm } = env;
    const vm_authority = signer;

    const tx = await program.methods
        .relaySaveRoot(
        )
        .accountsPartial({
            vm,
            vmAuthority: vm_authority.publicKey,
            relay: relay.address,
        })
        .signers([signer])
        .rpc({ skipPreflight: false });

    await connection.confirmTransaction(tx);

    console.log("tx", tx);
}


async function transferWithCommitmentInternal(
    vm: TestVm,
    relay: TestRelay, 
    memory: TestMemory,
    vta: TestUser,
    recent_root: Buffer,
    transcript: Buffer, 
    commitment: Buffer,
    amount: number, 
    vra_index: number,
    ) {

    // Create the opcode
    const opcode = OpCode.Splitter_TransferToInternal;
    const mem_indicies = [
        vta.memory_index,
        vra_index,
    ];
    const mem_banks = [0, 0]; // using one memory bank for all accounts
    const [data] = serialize({
        opcode,
        data: {
            amount,
            transcript: new anchor.web3.PublicKey(transcript),
            recent_root: new anchor.web3.PublicKey(recent_root),
            commitment: new anchor.web3.PublicKey(commitment),
        }
    });

    await vm_relay_exec(
        vm, 
        [memory, null, null, null],
        opcode, 
        data, 
        mem_indicies, 
        mem_banks, 
        null,  // no external address
        relay.address,
        relay.vault,
    );
}

async function transferWithCommitmentExternal(
    vm: TestVm,
    relay: TestRelay, 
    memory: TestMemory,
    destination: anchor.web3.PublicKey,
    recent_root: Buffer,
    transcript: Buffer, 
    commitment: Buffer,
    amount: number, 
    vra_index: number,
    ) {

    // Create the opcode
    const opcode = OpCode.Splitter_TransferToExternal;
    const mem_indicies = [
        vra_index,
    ];
    const mem_banks = [0]; // using one memory bank for all accounts
    const [data] = serialize({
        opcode,
        data: {
            amount,
            transcript: new anchor.web3.PublicKey(transcript),
            recent_root: new anchor.web3.PublicKey(recent_root),
            commitment: new anchor.web3.PublicKey(commitment),
        }
    });

    await vm_relay_exec(
        vm, 
        [memory, null, null, null],
        opcode, 
        data, 
        mem_indicies, 
        mem_banks, 
        destination, 
        relay.address,
        relay.vault,
    );
}

async function getCommitmentPda (
    env: TestVm,
    relay: anchor.web3.PublicKey,
    merkle_root: Buffer, 
    transcript: Buffer, 
    destination: anchor.web3.PublicKey,
    amount: number
    ) {
    const { program } = env;

    return anchor.web3.PublicKey.findProgramAddressSync(
        [
            Buffer.from("code-vm"),
            Buffer.from("relay_commitment"),
            relay.toBuffer(),
            merkle_root,
            transcript,
            destination.toBuffer(),
            new anchor.BN(amount).toBuffer('le', 8),
        ],
        program.programId
    )
}



export {
    createRelay,
    getRelayProof,

    saveRelayRoot,

    getCommitmentPda,

    transferWithCommitmentExternal,
    transferWithCommitmentInternal,
}