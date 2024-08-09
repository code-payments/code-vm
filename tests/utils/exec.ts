import * as anchor from "@coral-xyz/anchor";
import * as token from "@solana/spl-token";
import { OpCode } from "./opcode";
import { TestVm } from "./vm";
import { TestMemory } from "./memory";
import { sendLutTransaction } from "./lut";

export async function vm_timelock_exec(
    vm: TestVm, 
    memory: TestMemory[],
    opcode: OpCode, 
    data: Buffer, 
    mem_indicies: number[], 
    mem_banks: number[], 
    externalAddress: anchor.web3.PublicKey | null = null,
) {
    const memoryAccounts = getMemoryAccounts(memory);
    const vmOmnibus = externalAddress ? vm.omnibus : null;
    const tokenProgram = externalAddress ? token.TOKEN_PROGRAM_ID : null;

    const ix = [];
    ix.push(await createInstruction(
        vm, 
        memoryAccounts, 
        opcode, 
        data, 
        mem_indicies, 
        mem_banks, 
        externalAddress, 
        vmOmnibus, 
        tokenProgram
    ));

    const sig = await sendLutTransaction(vm, ix, vm.lut);
    console.log("tx", sig);
}

export async function vm_timelock_exec_multiple(
    vm: TestVm, 
    memory: TestMemory[],
    actions: {
        opcode: OpCode, 
        data: Buffer, 
        mem_indicies: number[], 
        mem_banks: number[], 
        externalAddress: anchor.web3.PublicKey | null,
    }[]
) {
    const memoryAccounts = getMemoryAccounts(memory);
    const hasExternalAddress = actions.some((a) => a.externalAddress !== null);
    const vmOmnibus = hasExternalAddress ? vm.omnibus : null;
    const tokenProgram = hasExternalAddress ? token.TOKEN_PROGRAM_ID : null;

    const ix = await Promise.all(actions.map(async (action) => 
        createInstruction(
            vm, 
            memoryAccounts, 
            action.opcode, 
            action.data, 
            action.mem_indicies, 
            action.mem_banks, 
            action.externalAddress, 
            vmOmnibus, 
            tokenProgram
        )
    ));

    const sig = await sendLutTransaction(
        vm, 
        [...ix ],
        vm.lut
    );

    console.log("tx", sig);
}


export async function vm_relay_exec(
    vm: TestVm, 
    memory: TestMemory[],
    opcode: OpCode, 
    data: Buffer, 
    memIndicies: number[], 
    memBanks: number[], 
    externalAddress: anchor.web3.PublicKey | null = null,
    relay: anchor.web3.PublicKey | null = null,
    relayVault: anchor.web3.PublicKey | null = null,
) {
    const memoryAccounts = getMemoryAccounts(memory);

    // Unlike the timelock variant, these two accounts are always required for
    // internal transfers even though this is a "internal transfer". The reason
    // is that real tokens are being moved from the relay treasury to the omibus
    // account.
    const vmOmnibus = vm.omnibus;
    const tokenProgram = token.TOKEN_PROGRAM_ID;

    const ix = [
        await createInstruction(
            vm, 
            memoryAccounts, 
            opcode, 
            data, 
            memIndicies, 
            memBanks, 
            externalAddress, 
            vmOmnibus, 
            tokenProgram, 
            relay, 
            relayVault
        )
    ];

    const sig = await sendLutTransaction(vm, ix, vm.lut);

    console.log("tx", sig);
}

async function createInstruction(
    vm: TestVm, 
    memory: (anchor.web3.PublicKey | null)[],
    opcode: OpCode, 
    data: Buffer, 
    memIndicies: number[], 
    memBanks: number[], 
    externalAddress: anchor.web3.PublicKey | null = null,
    vmOmnibus: anchor.web3.PublicKey | null = null,
    tokenProgram: anchor.web3.PublicKey | null = null,
    relay: anchor.web3.PublicKey | null = null,
    relayVault: anchor.web3.PublicKey | null = null
): Promise<anchor.web3.TransactionInstruction> {
    const { program, signer } = vm;

    const tx = program.methods
        .vmExec(
            opcode,
            memIndicies,
            Buffer.from(memBanks), // Anchor requires this conversion
            data,
        )
        .accountsPartial({
            vm: vm.vm,
            vmAuthority: signer.publicKey,

            memA: memory[0],
            memB: memory[1],
            memC: memory[2],
            memD: memory[3],

            externalAddress, 
            vmOmnibus, 
            tokenProgram,
            relay,
            relayVault,
        });

    return await tx.instruction();
}

function getMemoryAccounts(memory: TestMemory[]): (anchor.web3.PublicKey | null)[] {
    return memory.map((m) => m ? m.vm_memory_account : null);
}