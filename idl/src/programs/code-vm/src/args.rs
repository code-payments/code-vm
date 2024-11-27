use anchor_lang::prelude::*;
use crate::{Hash, Signature, MAX_NAME_LEN};


#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub struct InitVmArgs {
    pub lock_duration: u8,
    pub vm_bump: u8,
    pub vm_omnibus_bump: u8,
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub struct InitMemoryArgs {
    pub name: [u8; MAX_NAME_LEN],
    pub num_accounts: u32,
    pub account_size: u16,
    pub vm_memory_bump: u8,
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub struct ResizeMemoryArgs {
    pub account_size: u32,
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub struct InitStorageArgs {
    pub name: [u8; MAX_NAME_LEN],
    pub vm_storage_bump: u8,
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub struct ExecArgs {
    pub data: ExecArgsData,
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub struct ExecArgsData {
    pub opcode: u8,
    pub mem_indicies: Vec<u16>,
    pub mem_banks: Vec<u8>,
    pub data: Vec<u8>,
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub struct InitNonceArgs {
    pub account_index: u16,
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub struct InitTimelockArgs {
    pub account_index: u16,
    pub virtual_timelock_bump: u8,
    pub virtual_vault_bump: u8,
    pub unlock_pda_bump: u8,
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub struct CompressArgs {
    pub account_index: u16,
    pub signature: Signature,
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub struct DecompressArgs {
    pub data: DecompressArgsData,
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub struct DecompressArgsData {
    pub account_index: u16,
    pub packed_va: Vec<u8>,
    pub proof: Vec<Hash>,
    pub signature: Signature,
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub struct InitRelayArgs {
    pub name: [u8; MAX_NAME_LEN],
    pub relay_bump: u8,
    pub relay_vault_bump: u8,
}


#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub struct SnapshotArgs { // SaveRecentRoot
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub struct DepositArgs {
    pub account_index: u16,
    pub amount: u64,
    pub bump: u8,
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub struct InitUnlockArgs {
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub struct UnlockArgs {
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub struct WithdrawArgs {
    pub data: WithdrawArgsData,
}

#[repr(u8)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub enum WithdrawArgsData {
    FromMemory {
        account_index: u16,
    } = 0,
    FromStorage {
        packed_va: Vec<u8>,
        proof: Vec<Hash>,
        signature: Signature,
    } = 1,
    FromDeposit {
        bump: u8,
    } = 2,
}