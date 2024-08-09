import * as anchor from "@coral-xyz/anchor";
import { TestVm } from "./vm";
import { VirtualAccount } from "../memory/accounts";
import { MemoryAccountWithData } from "../memory/state";
import { AccountBuffer } from "../memory/allocator";
import { Page } from "../memory/page";

export type TestMemory = {
    vm_memory_account: anchor.web3.PublicKey;

    getData: () => Promise<Buffer>;
    getAccount: (index: number) => Promise<VirtualAccount>;
};

async function createMemory(env: TestVm, name: string, layout: number) : Promise<TestMemory> {
    const { program, connection, signer, vm } = env;

    const vm_authority = signer;
    const [vm_memory_account, _] = anchor.web3.PublicKey.findProgramAddressSync(
        [
            Buffer.from("code-vm"),
            Buffer.from("vm_memory_account"),
            // truncate and pad the name into a 32-byte buffer
            Buffer.from(name.slice(0, 32).padEnd(32, '\0')),
            vm.toBuffer(),
        ],
        program.programId
    )

    const tx = await program.methods
        .vmMemoryInit(
            name,
            layout
        )
        .accountsPartial({
            vm: vm,
            vmAuthority: vm_authority.publicKey,
            vmMemory: vm_memory_account,
        })
        .signers([signer])
        .rpc({ skipPreflight: false });

    await connection.confirmTransaction(tx);
    console.log("tx", tx);

    const getData = async () => {
        return (await connection.getAccountInfo(vm_memory_account)).data;
    }

    const getAccount = async (index: number) => {
        const mem_data = await getData();
        const mem_account = MemoryAccountWithData.struct.deserialize(mem_data)[0];
        const mem = AccountBuffer.struct.deserialize(Buffer.from(mem_account.data))[0];

        const account_info = mem.accounts[index];
        const sector = mem.sectors[account_info.sector];

        let pages: Page[] = [];
        let current: Page = sector.pages[account_info.page];
        pages.push(current);
        while (current.next_page != 0) {
            current = sector.pages[current.next_page];
            pages.push(current);
        }

        const data = Buffer.concat(pages.map(p => Buffer.from(p.data)));

        return new VirtualAccount(data);
    }

    return {
        vm_memory_account,

        getData,
        getAccount,
    };
}

async function resizeMemory(env: TestVm, memory: TestMemory, len: number) {
    const { program, connection, signer, vm } = env;
    const vm_authority = signer;

    const tx = await program.methods
        .vmMemoryResize(
            len
        )
        .accountsPartial({
            vm: vm,
            vmAuthority: vm_authority.publicKey,
            vmMemory: memory.vm_memory_account,
        })
        .signers([signer])
        .rpc({ skipPreflight: false });

    await connection.confirmTransaction(tx);

    console.log("tx", tx);
}

export {
    createMemory,
    resizeMemory,
}