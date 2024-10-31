import * as idl from "../target/idl/code_vm.json";

// Pulled from: 
// https://github.com/code-payments/code-vm/blob/aa4d916a1179feca99155ede5a6f11035eb0f2f2/api/src/instruction.rs#L11
const instructionValues = {
    init_vm: [1],
    init_memory: [2],
    init_storage: [3],
    init_relay: [4],
    init_nonce: [5],
    init_timelock: [6],
    init_unlock: [7],
    exec: [8],
    compress: [9],
    decompress: [10],
    resize_memory: [11],
    snapshot: [12],
    deposit: [13],
    withdraw: [14],
    unlock: [15],
};

// Pulled from: 
// https://github.com/code-payments/code-vm/blob/aa4d916a1179feca99155ede5a6f11035eb0f2f2/api/src/state.rs#L14
const accountValues = {
    CodeVmAccount: [1, 0, 0, 0, 0, 0, 0, 0],
    MemoryAccount: [2, 0, 0, 0, 0, 0, 0, 0],
    StorageAccount: [3, 0, 0, 0, 0, 0, 0, 0],
    RelayAccount: [4, 0, 0, 0, 0, 0, 0, 0],
    UnlockStateAccount: [5, 0, 0, 0, 0, 0, 0, 0],
    WithdrawReceiptAccount: [6, 0, 0, 0, 0, 0, 0, 0],
}

function updateDiscriminators() {
    const instructions = idl.instructions;
    for (let ix of instructions) {
        const val = instructionValues[ix.name];
        if (val === undefined) {
            throw new Error(`Instruction ${ix.name} not found`);
        }
        ix.discriminator = val;
    }

    const accounts = idl.accounts;
    for (const acc of accounts) {
        const val = accountValues[acc.name];
        if (val === undefined) {
            throw new Error(`Account ${acc.name} not found`);
        }
        acc.discriminator = val;
    }

    return idl;
}

console.log(JSON.stringify(updateDiscriminators(), null, 2));