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
        MemoryLayout,
    },
    types::{ Hash, FixedSize }, 
    CODE_VM_PREFIX
};

#[derive(Accounts)]
#[instruction(
    name: String,
    layout: u8,
)]
pub struct CodeVmMemoryInit<'info> {
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
        init, 
        seeds = [
            CODE_VM_PREFIX.as_bytes(),
            b"vm_memory_account",
            &name.to_fixed(MemoryAccount::MAX_NAME_LEN),
            vm.to_account_info().key.as_ref(),
        ], 
        payer = vm_authority, 
        space = MemoryAccount::LEN,
        bump, 
    )]
    pub vm_memory: Account<'info, MemoryAccount>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn cvm_memory_init(ctx: Context<CodeVmMemoryInit>, name: String, layout: u8) -> Result<()> {
    require!(MemoryLayout::from_u8(layout).is_some(), CodeVmError::InvalidMemoryLayout);
    require!(name.len() <= MemoryAccount::MAX_NAME_LEN, CodeVmError::InvalidMemoryName);

    let mut vm = CodeVm::new(ctx.accounts.vm.to_owned());

    let data = name.clone().to_fixed(MemoryAccount::MAX_NAME_LEN);
    ctx.accounts.vm_memory.name.copy_from_slice(data.as_ref());
    ctx.accounts.vm_memory.vm = ctx.accounts.vm.key();
    ctx.accounts.vm_memory.bump = ctx.bumps.vm_memory;
    ctx.accounts.vm_memory.layout = layout;

    // Advance the vm state to include this instruction
    advance_poh!(ctx, vm, 
        get_message_hash(&vm, &ctx, name, layout),
        None
    );

    Ok(())
}

fn get_message_hash(
    vm: &CodeVm,
    ctx: &Context<CodeVmMemoryInit>,
    name: String,
    layout: u8,
) -> Hash {

    let blockhash = vm.get_current_poh();
    let args = instruction::VmMemoryInit {
        name,
        layout,
    };

    let accounts = ctx.accounts.to_account_metas(None);
    let data = args.try_to_vec().unwrap();
    let ix = vec![
        Instruction {
            program_id: program::CodeVm::id(),
            accounts,
            data: [
                instruction::VmMemoryInit::DISCRIMINATOR.to_vec(),
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
