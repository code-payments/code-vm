import * as token from "@solana/spl-token";
import * as util from "../utils/relay";
import { sha256 } from '@noble/hashes/sha256';
import { expect } from "chai";
import { CodeVM } from "../env";
import { getCommitmentPda } from "../utils/relay";
import { MerkleTree } from "../utils/merkle-tree";
import { getAccount } from "./timelock";

export const getRelay = async (env: CodeVM, index: number) => {
    const { program } = env.vm;

    const local = env.relays[index];
    return await program.account.relayAccount.fetch(local.address);
}

export const getRelayVault = async (env: CodeVM, index: number) => {
    const relay = env.relays[index];
    const connection = env.vm.connection;
    return await token.getAccount(connection, relay.vault)
}

export const checkRelay = async (env: CodeVM, index: number, balance: number) => {
    const expected = env.relays[index];
    const actual = await getRelay(env, index);

    expect(actual.vm.toBase58()).to.equal(env.vm.vm.toBase58());
    expect(actual.bump).to.equal(expected.bump);
    expect(actual.treasury.vault.toBase58()).to.equal(expected.vault.toBase58());
    expect(actual.name).to.equal(expected.name);
    expect(actual.numHistory).to.equal(expected.numHistory);
    expect(actual.numLevels).to.equal(expected.numLevels);

    expect([...actual.history.root.value]).to.deep.equal([...expected.history.root]);

    const ata = await getRelayVault(env, index);
    expect(ata.amount.toString()).to.equal(balance.toString());
}

const checkHistory = (expected: MerkleTree, actual: MerkleTree) => {
    expect(expected.levels).to.eq(actual.levels, "levels");
    expect(expected.root.toString('hex')).to.eq(actual.root.toString('hex'), "root");
    expect(expected.nextIndex).to.eq(actual.nextIndex, "nextIndex");

    expect(expected.filledSubtrees.map(x => x.toString('hex')))
        .to.deep.eq(actual.filledSubtrees.map(x => x.toString('hex')), "filledSubtrees");

    expect(expected.zeroTree.map(x => x.toString('hex')))
        .to.deep.eq(actual.zeroTree.map(x => x.toString('hex')), "zeroTree");
}

export async function test_create_relays(env: CodeVM, count: number, balance: number): Promise<CodeVM> {
    const { connection, signer, mint, } = env.vm;
    const vm = { ...env, relays : [], relayProofs: [] }

    for (let i = 0; i < count; i++) {
        const relay = await util.createRelay(vm.vm, `pool_${i}`)
        vm.relays.push(relay);

        await token.mintTo(connection, signer, mint, relay.vault, signer, balance);
        await checkRelay(vm, i, balance);
    }

    return vm;
}

export async function test_save_relay_root(env: CodeVM, index: number): Promise<CodeVM> {
    const { vm } = env;
    const local = env.relays[index];

    const beforeState = await getRelay(env, index);
    await util.saveRelayRoot(vm, local);
    const afterState = await getRelay(env, index);

    // Expect to find the beforeRoot in the afterState history
    expect(afterState.recentRoots.items.map(x => x.toString()))
        .to.contain(beforeState.history.root.value.toString());

    return env;
}

export async function test_blind_transfer(
    env: CodeVM, relay_index: number, dst_index: number, amount: number, vra_index: number
): Promise<CodeVM> {
    const { vm } = env;
    const relay = env.relays[relay_index];
    const vta = env.accounts[dst_index];
    const destination = vta.ata; // sending to the user's real ATA token account

    // Get a root from the current on-chain program
    const recent_root = await relay.getRecentRoot();

    // Usually, we'd encode the payment details into a transcript, but for this
    // test we're just using a random transcript for simplicity. The later tests
    // will use the actual payment details.
    const transcript = Buffer.from(sha256(Buffer.from(`no strings attached; tokens: ${amount}`)));

    // Calculate the commitment and bump seed that could be used to open a
    // re-payment token account (we're not doing this here, hence "blind"
    // transfer)
    const [commitment, bump] = await getCommitmentPda(
        env.vm,
        relay.address,
        recent_root,
        transcript,
        destination,
        amount,
    );

    const beforeBalance = (await token.getAccount(vm.connection, destination)).amount;

    // Ask the pool to send tokens to a virtual account, no strings attached (blind transfer)
    await util.transferWithCommitmentExternal(
        vm,
        relay, 
        env.vm_memory,
        destination, 
        recent_root,
        transcript, 
        commitment.toBuffer(),
        amount,
        vra_index
    );

    const relayProof = await util.getRelayProof(env.vm_memory, vra_index);
    env.relayProofs[vra_index] = relayProof;
    expect(relayProof.destination.toBase58()).to.equal(relay.vault.toBase58());
    expect(relayProof.commitment.toBase58()).to.equal(commitment.toBase58());
    expect(relayProof.recentRoot.toBuffer().toString('hex')).to.equal(recent_root.toString('hex'));

    const afterBalance = (await token.getAccount(vm.connection, destination)).amount;

    // Add the commitment locally
    relay.history.insert(commitment.toBuffer());

    // Fetch the new on-chain state of the relay
    const state = await relay.getState();

    // Check that the on-chain state matches the local state
    const expectedTree = relay.history;
    const actualTree = MerkleTree.from(state.history); // create a local tree from the on-chain state

    checkHistory(expectedTree, actualTree);
    
    
    // Check the destination account balance
    expect(afterBalance.toString()).to.equal((Number(beforeBalance) + amount).toString());

    return env;
}

export async function test_blind_transfer_to_virtual_account(
    env: CodeVM, relay_index: number, dst_index: number, amount: number, vra_index: number
): Promise<CodeVM> {
    const { vm } = env;
    const relay = env.relays[relay_index];
    const vta = env.accounts[dst_index]; // virtual token account

    // Get a root from the current on-chain program
    const recent_root = await relay.getRecentRoot();

    // Usually, we'd encode the payment details into a transcript, but for this
    // test we're just using a random transcript for simplicity. The later tests
    // will use the actual payment details.
    const transcript = Buffer.from(sha256(Buffer.from(`no strings attached; tokens: ${amount}`)));

    // Calculate the commitment and bump seed that could be used to open a
    // re-payment token account (we're not doing this here, hence "blind"
    // transfer)
    const [commitment, bump] = await getCommitmentPda(
        env.vm,
        relay.address,
        recent_root,
        transcript,
        vta.virtual_vault_address,
        amount,
    );

    const beforeBalance = (await getAccount(env, dst_index)).balance;

    // Ask the pool to send tokens to a virtual account, no strings attached (blind transfer)
    await util.transferWithCommitmentInternal(
        vm,
        relay, 
        env.vm_memory, 
        vta, 
        recent_root,
        transcript, 
        commitment.toBuffer(),
        amount,
        vra_index
    );

    const relayProof = await util.getRelayProof(env.vm_memory, vra_index);
    env.relayProofs[vra_index] = relayProof;
    expect(relayProof.destination.toBase58()).to.equal(relay.vault.toBase58());
    expect(relayProof.commitment.toBase58()).to.equal(commitment.toBase58());
    expect(relayProof.recentRoot.toBuffer().toString('hex')).to.equal(recent_root.toString('hex'));

    const afterBalance = (await getAccount(env, dst_index)).balance;

    // Add the commitment locally
    relay.history.insert(commitment.toBuffer());

    // Fetch the new on-chain state of the relay
    const state = await relay.getState();

    // Check that the on-chain state matches the local state
    const expectedTree = relay.history;
    const actualTree = MerkleTree.from(state.history); // create a local tree from the on-chain state

    checkHistory(expectedTree, actualTree);

    // Check the destination account balance
    expect(afterBalance.toString()).to.equal((Number(beforeBalance) + amount).toString());

    return env;
}
