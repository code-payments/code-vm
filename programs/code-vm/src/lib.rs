use anchor_lang::prelude::*;

pub static CODE_VM_PREFIX: &str = "code-vm";

pub mod cvm;
pub mod error;
pub mod instructions;
pub mod macros;
pub mod types;
pub mod utils;

use instructions::*;
use types::hash::Hash;
use types::signature::Signature;

declare_id!("vmZ1RZZyaskXS2fF668wonXFZiUDurcCr2eCev4kuuT");

#[program]
pub mod code_vm {

    use super::*;

    pub fn vm_init(
        ctx: Context<CodeVmInit>,
        lock_duration: u8,
    ) -> Result<()> {
        instructions::cvm_init(
            ctx,
            lock_duration
        )
    }

    pub fn vm_memory_init(
        ctx: Context<CodeVmMemoryInit>,
        name: String,
        layout: u8,
    ) -> Result<()> {
        instructions::cvm_memory_init(
            ctx,
            name,
            layout
        )
    }

    pub fn vm_memory_resize(
        ctx: Context<CodeVmMemoryResize>,
        len: u32
    ) -> Result<()> {
        instructions::cvm_memory_resize(
            ctx,
            len,
        )
    }

    pub fn vm_storage_init(
        ctx: Context<CodeVmCompressedStorageInit>,
        name: String,
        levels: u8,
    ) -> Result<()> {
        instructions::cvm_storage_init(
            ctx,
            name,
            levels,
        )
    }

    pub fn vm_exec(
        ctx: Context<CodeVmExec>,
        opcode: u8,
        mem_indicies: Vec<u16>,
        mem_banks: Vec<u8>,
        data: Vec<u8>,
    ) -> Result<()> {
        instructions::cvm_exec(
            ctx, 
            opcode,
            mem_indicies,
            mem_banks,
            data
        )
    }

    pub fn system_nonce_init(
        ctx: Context<CodeVmVirtualNonceInit>,
        account_index: u16,
    ) -> Result<()> {
        instructions::cvm_vdn_init(
            ctx, 
            account_index
        )
    }

    pub fn system_timelock_init(
        ctx: Context<CodeVmVirtualTimelockInit>,
        account_index: u16,
        virtual_timelock_bump: u8,
        virtual_vault_bump: u8,
        unlock_pda_bump: u8,
    ) -> Result<()> {
        instructions::cvm_vta_init(
            ctx, 
            account_index, 
            virtual_timelock_bump, 
            virtual_vault_bump, 
            unlock_pda_bump
        )
    }

    pub fn system_account_compress(
        ctx: Context<CodeVmVirtualAccountCompress>,
        account_index: u16,
        signature: Signature,
    ) -> Result<()> {
        instructions::cvm_va_compress(
            ctx,
            account_index,
            signature,
        )
    }

    pub fn system_account_decompress(
        ctx: Context<CodeVmVirtualAccountDecompress>,
        account_index: u16,
        packed_va: Vec<u8>,
        proof: Vec<Hash>,
        signature: Signature,
    ) -> Result<()> {
        instructions::cvm_va_decompress(
            ctx,
            account_index,
            packed_va, 
            proof,
            signature,
        )
    }

    pub fn timelock_unlock_init(
        ctx: Context<TimelockUnlockInit>,
        virtual_account_bump: u8,
    ) -> Result<()> {
        instructions::timelock_unlock_init(
            ctx, 
            virtual_account_bump,
        )
    }

    pub fn timelock_unlock_request(
        ctx: Context<TimelockUnlockRequest>,
    ) -> Result<()> {
        instructions::timelock_unlock_request(
            ctx,
        )
    }

    pub fn timelock_unlock_finalize(
        ctx: Context<TimelockUnlockFinalize>,
    ) -> Result<()> {
        instructions::timelock_unlock_finalize(
            ctx,
        )
    }

    pub fn timelock_deposit_from_ata(
        ctx: Context<TimelockDepositFromAta>,
        account_index: u16,
        amount: u64,
    ) -> Result<()> {
        instructions::timelock_deposit_from_ata(
            ctx,
            account_index,
            amount,
        )
    }

    pub fn timelock_deposit_from_pda(
        ctx: Context<TimelockDepositFromPda>,
        account_index: u16,
        amount: u64,
        bump: u8,
    ) -> Result<()> {
        instructions::timelock_deposit_from_pda(
            ctx,
            account_index,
            amount,
            bump,
        )
    }

    pub fn timelock_withdraw_from_deposit(
        ctx: Context<TimelockWithdrawFromDeposit>,
        bump: u8,
    ) -> Result<()> {
        instructions::timelock_withdraw_from_deposit(
            ctx,
            bump
        )
    }

    pub fn timelock_withdraw_from_memory(
        ctx: Context<TimelockWithdrawFromMemory>,
        account_index: u16,
        nonce: Hash,
    ) -> Result<()> {
        instructions::timelock_withdraw_from_memory(
            ctx,
            account_index,
            nonce,
        )
    }

    pub fn timelock_withdraw_from_storage(
        ctx: Context<TimelockWithdrawFromStorage>,
        unchecked_vta: cvm::VirtualTimelockAccount,
        proof: Vec<Hash>,
        signature: Signature,
    ) -> Result<()> {
        instructions::timelock_withdraw_from_storage(
            ctx,
            unchecked_vta,
            proof,
            signature,
        )
    }

    pub fn relay_init(
        ctx: Context<RelayInit>,
        num_levels: u8,
        num_history: u8,
        name: String,
    ) -> Result<()> {
        instructions::relay_init(
            ctx,
            num_levels,
            num_history,
            name,
        )
    }

    pub fn relay_save_root(
        ctx: Context<RelaySaveRoot>,
    ) -> Result<()> {
        instructions::relay_save_root(
            ctx,
        )
    }

}

