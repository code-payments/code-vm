#pragma endian little

#define MAX_NAME_LEN 32

// Type Definitions
struct Vec<T> {
    u32 len;
    T data[len];
};

struct Pubkey {
    u8 value[32];
};

struct Hash {
    u8 value[32];
};

struct Signature {
    u8 value[64];
};

enum Instruction : u8 {
    Unknown = 0,
    InitVmIx = 1,
    InitMemoryIx = 2,
    InitStorageIx = 3,
    InitRelayIx = 4,
    InitNonceIx = 5,
    InitTimelockIx = 6,
    InitUnlockIx = 7,
    ExecIx = 8,
    CompressIx = 9,
    DecompressIx = 10,
    ResizeMemoryIx = 11,
    SnapshotIx = 12,
    DepositIx = 13,
    WithdrawIx = 14,
    UnlockIx = 15,
};

enum Opcode : u8 {
    Unknown = 0,
    ExternalTransferOp = 10,
    TransferOp = 11,
    ConditionalTransferOp = 12,
    ExternalWithdrawOp = 13,
    WithdrawOp = 14,
    ExternalRelayOp = 20,
    RelayOp = 21,
};


struct TransferOp {
    Signature signature;
    u64 amount;
};

struct WithdrawOp {
    Signature signature;
};

struct RelayOp {
    u64 amount;
    Hash transcript;
    Hash recent_root;
    Pubkey commitment;
};

struct ExternalTransferOp {
    Signature signature;
    u64 amount;
};

struct ExternalWithdrawOp {
    Signature signature;
};

struct ExternalRelayOp {
    u64 amount;
    Hash transcript;
    Hash recent_root;
    Pubkey commitment;
};

struct ConditionalTransferOp {
    Signature signature;
    u64 amount;
};

struct ExecArgs {
    Opcode opcode;
    Vec<u16> mem_indices;
    Vec<u8> mem_banks;
    u32 data_len;

    // Parsing the data field based on opcode
    if (opcode == Opcode::TransferOp) {
        TransferOp data;
    }
    else if (opcode == Opcode::WithdrawOp) {
        WithdrawOp data;
    }
    else if (opcode == Opcode::RelayOp) {
        RelayOp data;
    }
    else if (opcode == Opcode::ExternalTransferOp) {
        ExternalTransferOp data;
    }
    else if (opcode == Opcode::ExternalWithdrawOp) {
        ExternalWithdrawOp data;
    }
    else if (opcode == Opcode::ExternalRelayOp) {
        ExternalRelayOp data;
    }
    else if (opcode == Opcode::ConditionalTransferOp) {
        ConditionalTransferOp data;
    }
    else {
        // Unknown opcode, read data as bytes
        u8 data[data_bytes.len];
    }
};

struct InitVmArgs {
    u8 lock_duration;
    u8 vm_bump;
    u8 vm_omnibus_bump;
};

struct InitMemoryArgs {
    u8 name[MAX_NAME_LEN];
    u32 num_accounts;
    u16 account_size;
    u8 vm_memory_bump;
};

struct ResizeMemoryArgs {
    u32 account_size;
};

struct InitStorageArgs {
    u8 name[MAX_NAME_LEN];
    u8 vm_storage_bump;
};

struct InitNonceArgs {
    u16 account_index;
};

struct InitTimelockArgs {
    u16 account_index;
    u8 virtual_timelock_bump;
    u8 virtual_vault_bump;
    u8 unlock_pda_bump;
};

struct CompressArgs {
    u16 account_index;
    Signature signature;
};

struct DecompressArgs {
    u16 account_index;
    Vec<u8> packed_va;
    Vec<Hash> proof;
    Signature signature;
};

struct InitRelayArgs {
    u8 name[MAX_NAME_LEN];
    u8 relay_bump;
    u8 relay_vault_bump;
};

struct SnapshotArgs { };

struct DepositArgs {
    u16 account_index;
    u64 amount;
    u8 bump;
};

struct InitUnlockArgs { };

struct UnlockArgs { };

struct WithdrawArgsFromMemory {
    u16 account_index;
};

struct WithdrawArgsFromStorage {
    Vec<u8> packed_va;
    Vec<Hash> proof;
    Signature signature;
};

struct WithdrawArgsFromDeposit {
    u8 bump;
};

struct WithdrawArgs {
    u8 discriminator;
    if (discriminator == 0) { // FromMemory
        WithdrawArgsFromMemory data;
    }
    else if (discriminator == 1) { // FromStorage
        WithdrawArgsFromStorage data;
    }
    else if (discriminator == 2) { // FromDeposit
        WithdrawArgsFromDeposit data;
    }
    else {
        u8 data[];
    }
};

struct InstructionData {
    Instruction discriminator;

    if (discriminator == Instruction::InitVmIx) {
        InitVmArgs data;
    }
    else if (discriminator == Instruction::InitMemoryIx) {
        InitMemoryArgs data;
    }
    else if (discriminator == Instruction::InitStorageIx) {
        InitStorageArgs data;
    }
    else if (discriminator == Instruction::InitRelayIx) {
        InitRelayArgs data;
    }
    else if (discriminator == Instruction::InitNonceIx) {
        InitNonceArgs data;
    }
    else if (discriminator == Instruction::InitTimelockIx) {
        InitTimelockArgs data;
    }
    else if (discriminator == Instruction::InitUnlockIx) {
        InitUnlockArgs data;
    }
    else if (discriminator == Instruction::ExecIx) {
        ExecArgs data;
    }
    else if (discriminator == Instruction::CompressIx) {
        CompressArgs data;
    }
    else if (discriminator == Instruction::DecompressIx) {
        DecompressArgs data;
    }
    else if (discriminator == Instruction::ResizeMemoryIx) {
        ResizeMemoryArgs data;
    }
    else if (discriminator == Instruction::SnapshotIx) {
        SnapshotArgs data;
    }
    else if (discriminator == Instruction::DepositIx) {
        DepositArgs data;
    }
    else if (discriminator == Instruction::WithdrawIx) {
        WithdrawArgs data;
    }
    else if (discriminator == Instruction::UnlockIx) {
        UnlockArgs data;
    }
    else {
        u8 data[];
    }
};

// Entry Point
InstructionData instruction_data @ 0x00;
