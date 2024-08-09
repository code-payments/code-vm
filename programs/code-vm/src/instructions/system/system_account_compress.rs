
use anchor_lang::{prelude::*, Discriminator};
use anchor_lang::solana_program::instruction::Instruction;

use crate::advance_poh;
use crate::error::CodeVmError;
use crate::{ 
    utils,
    program,
    instruction,
    cvm::{ 
        CodeVm, 
        CodeVmAccount, 
        CompressedStorageAccount,
        MemoryAccount,
        MemoryAccountWithData,
        ChangeLogData,
    },
    types::{ Hash, Signature },
    CODE_VM_PREFIX
};

#[derive(Accounts)]
#[instruction(
    account_index: u16,
    signature: Signature,
)]
pub struct CodeVmVirtualAccountCompress<'info> {
    #[account(mut)]
    pub vm_authority: Signer<'info>,

    #[account(
        mut, // the POH value is updated
        constraint = vm.authority == vm_authority.key(),

        seeds=[
            CODE_VM_PREFIX.as_bytes(),
            vm.mint.as_ref(),
            vm.authority.as_ref(),
            vm.lock_duration.to_le_bytes().as_ref(),
        ],
        bump = vm.bump
    )]
    pub vm: Box<Account<'info, CodeVmAccount>>,

    #[account(
        mut,
        seeds = [
            CODE_VM_PREFIX.as_bytes(),
            b"vm_memory_account",
            vm_memory.name.as_ref(),
            vm.to_account_info().key.as_ref(),
        ], 
        bump = vm_memory.bump
    )]
    pub vm_memory: Account<'info, MemoryAccount>,

    #[account(
        mut,
        seeds = [
            CODE_VM_PREFIX.as_bytes(),
            b"vm_storage_account",
            &vm_storage.name.as_bytes(),
            vm.to_account_info().key.as_ref(),
        ], 
        bump = vm_storage.bump, 
    )]
    pub vm_storage: Account<'info, CompressedStorageAccount>,
}

pub fn cvm_va_compress(
    ctx: Context<CodeVmVirtualAccountCompress>,
    account_index: u16,
    signature: Signature,
) -> Result<()> {

    let info = ctx.accounts.vm_memory.to_account_info();
    let data = info.try_borrow_mut_data()?;
    let memory = MemoryAccountWithData::into_indexed_memory(data);
    let mut vm = CodeVm::new(ctx.accounts.vm.to_owned());

    vm.use_memory(memory);
    require!(vm.is_allocated(account_index), CodeVmError::VirtualAccountNotAllocated);

    let va = vm.read_account(account_index).unwrap();
    let tree = &mut ctx.accounts.vm_storage.memory_state;

    vm.try_compress(va, tree, signature)?;
    vm.try_delete_account(account_index)?;

    // Advance the vm state to include this instruction
    advance_poh!(ctx, vm, 
        get_message_hash(&vm, &ctx, account_index, signature),
        Some(ChangeLogData::Compress {
            store: ctx.accounts.vm_storage.key(),
            account: va,
            signature,
        })
    );

    Ok(())
}

fn get_message_hash(
    vm: &CodeVm,
    ctx: &Context<CodeVmVirtualAccountCompress>,
    account_index: u16,
    signature: Signature,
) -> Hash {

    let blockhash = vm.get_current_poh();
    let args = instruction::SystemAccountCompress {
        account_index,
        signature,
    };

    let accounts = ctx.accounts.to_account_metas(None);
    let data = args.try_to_vec().unwrap();
    let ix = vec![
        Instruction {
            program_id: program::CodeVm::id(),
            accounts,
            data: [
                instruction::SystemAccountCompress::DISCRIMINATOR.to_vec(),
                data,
            ].concat(),
        }
    ];
    
    let message = utils::message_with_sorted_keys(
        &ix,
        Some(&vm.get_authority()),
        &blockhash,
    );

    let message = message.serialize();
    utils::hash(&message)
}