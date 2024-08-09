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
        MemoryAccount,
    },
    types::Hash,
    CODE_VM_PREFIX
};

#[derive(Accounts)]
#[instruction(
    len: u32
)]
pub struct CodeVmMemoryResize<'info> {
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

        realloc = len as usize, 
        realloc::zero = true, 
        realloc::payer=vm_authority,

        seeds = [
            CODE_VM_PREFIX.as_bytes(),
            b"vm_memory_account",
            vm_memory.name.as_ref(),
            vm.to_account_info().key.as_ref(),
        ], 
        bump = vm_memory.bump
    )]
    pub vm_memory: Account<'info, MemoryAccount>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn cvm_memory_resize(ctx: Context<CodeVmMemoryResize>, len: u32) -> Result<()> {
    require!(len > 0, CodeVmError::InvalidMemorySize);

    let mut vm = CodeVm::new(ctx.accounts.vm.to_owned());

    // Advance the vm state to include this instruction
    advance_poh!(ctx, vm, 
        get_message_hash(&vm, &ctx, len),
        None
    );

    Ok(())
}

fn get_message_hash(
    vm: &CodeVm,
    ctx: &Context<CodeVmMemoryResize>,
    len: u32,
) -> Hash {

    let blockhash = vm.get_current_poh();
    let args = instruction::VmMemoryResize {
        len,
    };

    let accounts = ctx.accounts.to_account_metas(None);
    let data = args.try_to_vec().unwrap();
    let ix = vec![
        Instruction {
            program_id: program::CodeVm::id(),
            accounts,
            data: [
                instruction::VmMemoryResize::DISCRIMINATOR.to_vec(),
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
