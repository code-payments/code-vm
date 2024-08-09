import { expect } from "chai";
import * as token from "@solana/spl-token";
import * as util from "../utils/user";
import { CodeVM } from "../env";
import { VirtualTimelockAccount } from "../memory/accounts";

export const getAccount = async (env: CodeVM, index: number) => {
    const va = await env.vm_memory.getAccount(index);
    expect(va).to.not.be.undefined;
    expect(va.account).to.not.be.undefined;
    expect(va.variant).to.be.eq(1);
    return va.account as VirtualTimelockAccount;
}

export const getAta = async (env: CodeVM, index: number) => {
    const user = env.accounts[index];
    const connection = env.vm.connection;
    return await token.getAccount(connection, user.ata)
}

const checkAccount = async (vm: CodeVM, index: number, balance: number) => {
    const expected = vm.accounts[index];
    const actual = await getAccount(vm, index);

    expect(actual.owner.toBase58()).to.equal(expected.keypair.publicKey.toBase58());
    expect(actual.bump).to.equal(expected.virtual_timelock_bump);
    expect(actual.tokenBump).to.equal(expected.virtual_vault_bump);
    expect(actual.unlockBump).to.equal(expected.unlock_bump);

    expect(actual.balance.toString()).to.equal(balance.toString());
}

export async function test_create_accounts(env: CodeVM, count: number, offset: number): Promise<CodeVM> {
    const vm = { ...env, accounts : [] }
    for (let i = 0; i < count; i++) {
        const account_index = i + offset;
        const user = await util.createUser(vm.vm, account_index);

        vm.accounts[account_index] = user;

        await util.createVirtualAccount(env.vm, vm.vm_memory, vm.accounts[account_index]);
        await checkAccount(vm, account_index, 0);
    }

    return vm;
}

export async function test_create_unlock_state(env: CodeVM, vta_index: number): Promise<CodeVM> {
    const { vm } = env;

    await util.createUnlockPda(vm, env.accounts[vta_index]);
    await util.unlockUnlockPda(vm, env.accounts[vta_index]);

    return env;
}

export async function test_deposit_from_ata(env: CodeVM, vta_index: number, amount: number): Promise<CodeVM> {
    const { vm, vm_memory } = env;

    const before = await getAccount(env, vta_index);
    await util.depositFromAta(vm, vm_memory, env.accounts[vta_index], amount);
    await checkAccount(env, vta_index, Number(before.balance) + amount);

    return env;
}

export async function test_deposit_from_pda(env: CodeVM, vta_index: number, amount: number): Promise<CodeVM> {
    const { vm, vm_memory } = env;

    const before = await getAccount(env, vta_index);
    await util.depositFromPda(vm, vm_memory, env.accounts[vta_index], amount);
    await checkAccount(env, vta_index, Number(before.balance) + amount);

    return env;
}

export async function test_transfer(env: CodeVM, from_index: number, to_index: number, vdn_index: number, amount: number): Promise<CodeVM> {
    const { vm, vm_memory } = env;

    const src = await getAccount(env, from_index);
    const dst = await getAccount(env, to_index);

    await util.transferWithAuthority(
        vm,
        vm_memory,
        env.accounts[from_index],
        env.accounts[to_index],
        env.nonces[vdn_index],
        amount
    );

    await checkAccount(env, from_index, Number(src.balance) - amount);
    await checkAccount(env, to_index, Number(dst.balance) + amount);

    return env;
}

export async function test_transfer_multiple(
    env: CodeVM,
    actions: {
        from_index: number,
        to_index: number,
        vdn_index: number,
        amount: number
    }[]
): Promise<CodeVM> {
    const { vm, vm_memory } = env;

    // Get the set of all involved accounts
    const involvedIndices = Array.from(new Set(actions.flatMap(action => [action.from_index, action.to_index])));

    // Record initial balances
    const initialBalances = await Promise.all(involvedIndices.map(async index => {
        const account = await getAccount(env, index);
        return { index, balance: Number(account.balance) };
    }));

    const ix = actions.map(action => ({
        source_user: env.accounts[action.from_index],
        dest_user: env.accounts[action.to_index],
        vdn: env.nonces[action.vdn_index],
        amount: action.amount,
    }));

    await util.transferWithAuthorityMultiple(vm, vm_memory, ix);

    // Record expected final balances
    const expectedBalances = initialBalances.map(({ index, balance }) => {
        const finalBalance = actions.reduce((acc, action) => {
            if (action.from_index === index) return acc - action.amount;
            if (action.to_index === index) return acc + action.amount;
            return acc;
        }, balance);
        return { index, balance: finalBalance };
    });

    console.log("expectedBalances", expectedBalances);

    // Check final balances
    await Promise.all(expectedBalances.map(async ({ index, balance }) => {
        await checkAccount(env, index, balance);
    }));

    return env;
}

export async function test_transfer_external(env: CodeVM, from_index: number, to_index: number, vdn_index: number, amount: number): Promise<CodeVM> {
    const { vm, vm_memory } = env;

    const src = await getAccount(env, from_index);
    const dst = await getAccount(env, to_index);
    const beforeAta = await getAta(env, to_index);

    await util.transferWithAuthorityExternal(
        vm,
        vm_memory,
        env.accounts[from_index],
        env.accounts[to_index],
        env.nonces[vdn_index],
        amount
    );

    const afterAta = await getAta(env, to_index);
    await checkAccount(env, from_index, Number(src.balance) - amount);
    await checkAccount(env, to_index, Number(dst.balance)); // Should not change

    expect(afterAta.amount).to.equal(beforeAta.amount + BigInt(amount));

    return env;
}

