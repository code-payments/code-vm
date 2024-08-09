import { expect } from "chai";
import { CodeVM } from "../env";
import { createMemory, resizeMemory } from "../utils/memory";
import { createStorage } from "../utils/storage";
import { testVm } from "../utils/vm";
import { MemoryAccountWithData } from "../memory/state";

export enum MemoryLayout {
    Mixed = 0,
    Timelock,
    Nonce,
    Relay,
}

export async function test_create_vm() : Promise<CodeVM> {
    return { vm: await testVm() };
}

export async function test_create_memory(vm: CodeVM, name: string, layout: MemoryLayout): Promise<CodeVM> {
    return {
        ...vm,
        vm_memory: await createMemory(vm.vm, name, layout),
    };
}

export async function test_resize_memory(vm: CodeVM): Promise<CodeVM> {
    const required_size = MemoryAccountWithData.LEN;
    const chunk_size = 10*1024;
    const chunks = Math.floor(required_size / chunk_size);
    for (let i = 1; i <= chunks; i++) {
        await resizeMemory(vm.vm, vm.vm_memory, i*chunk_size);
    }
    await resizeMemory(vm.vm, vm.vm_memory, required_size);
    return vm;
}

export async function test_create_storage(vm: CodeVM): Promise<CodeVM> {
    const store_accounts = await createStorage(vm.vm, "vm_accounts");

    let expectedRoot = store_accounts.memoryState.root;
    let actualRoot = await store_accounts.getRoot();

    expect(expectedRoot.toString('hex')).to.equal(actualRoot.toString('hex'));

    return { 
        ...vm,
        store_accounts,
    };
}
