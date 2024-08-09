import * as anchor from "@coral-xyz/anchor";
import { sha256 } from '@noble/hashes/sha256';
import { TestVm } from "./vm";
import { TestMemory } from "./memory";
import { VirtualDurableNonce } from "../memory/accounts";

export type TestDurableNonce = {
    owner: anchor.web3.Keypair;
    address: anchor.web3.PublicKey;
    index: number;
    data_hash?: Buffer;

    getValue: () => Promise<anchor.web3.PublicKey>;
    getInfo: () => Promise<VirtualDurableNonce>;
};

async function createVirtualDurableNonce(
    env: TestVm, mem: TestMemory, index: number = 0
) : Promise<TestDurableNonce> {

    const { program, connection, signer, vm } = env;
    const vm_authority = signer;

    const owner = anchor.web3.Keypair.generate();

    // Not the solana blockhash, but the virtual machine blockhash
    const blockhash = await env.getCurrentPoh();

    // The CodeVM durable nonce addresses are derived from the owner's public
    // key. This is different from solana's nonce accounts.
    const address = new anchor.web3.PublicKey(
        sha256(Buffer.concat([
            owner.publicKey.toBuffer(),
            blockhash.toBuffer(),
        ])
    ));

    /*
    console.log("nonce index", index);
    console.log("nonce seed", owner.publicKey);
    console.log("nonce value", blockhash);
    console.log("nonce address", address);
    */
   

    const tx = await program.methods
        .systemNonceInit(
            index,
        )
        .accountsPartial({
            vm,
            vmAuthority: vm_authority.publicKey,
            vmMemory: mem.vm_memory_account,
            virtualAccountOwner: owner.publicKey,
        })
        .signers([signer])
        .rpc({ skipPreflight: false });

    await connection.confirmTransaction(tx);

    console.log("tx", tx);

    const getInfo = async () => {
        return (await mem.getAccount(index)).account as VirtualDurableNonce;
    }

    const getValue = async () => {
        return (await getInfo()).nonce;
    }

    return {
        owner,
        address,
        index,

        getValue,
        getInfo,
    };
}

export {
    createVirtualDurableNonce,
}