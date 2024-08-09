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
    }, 
    types::{ Hash, MerkleTree }, 
    CODE_VM_PREFIX
};


#[derive(Accounts)]
#[instruction(
    name: String,
    levels: u8,
)]
pub struct CodeVmCompressedStorageInit<'info> {
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
            b"vm_storage_account",
            &name.as_bytes(),
            vm.to_account_info().key.as_ref(),
        ], 
        payer = vm_authority, 
        space = CompressedStorageAccount::max_size_for(levels),
        bump, 
    )]
    pub vm_storage_account: Account<'info, CompressedStorageAccount>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn cvm_storage_init(ctx: Context<CodeVmCompressedStorageInit>, name: String, levels: u8) -> Result<()> {
    require!(levels > 0, CodeVmError::InvalidStorageLevels);
    require!(name.len() <= CompressedStorageAccount::MAX_NAME_LEN, CodeVmError::InvalidStorageName);

    let mut vm = CodeVm::new(ctx.accounts.vm.to_owned());

    ctx.accounts.vm_storage_account.vm = ctx.accounts.vm.key();
    ctx.accounts.vm_storage_account.bump = ctx.bumps.vm_storage_account;
    ctx.accounts.vm_storage_account.name = name.clone();

    let seeds: &[&[u8]] = &[
        b"merkletree",
        &name.as_bytes(),
        ctx.accounts.vm.to_account_info().key.as_ref()
    ];
    let merkle_tree = MerkleTree::new(seeds, levels);

    ctx.accounts.vm_storage_account.memory_state = merkle_tree;

    // Advance the vm state to include this instruction
    advance_poh!(ctx, vm, 
        get_message_hash(&vm, &ctx, name.clone(), levels),
        None
    );

    Ok(())
}

fn get_message_hash(
    vm: &CodeVm,
    ctx: &Context<CodeVmCompressedStorageInit>,
    name: String,
    levels: u8,
) -> Hash {

    let blockhash = vm.get_current_poh();
    let args = instruction::VmStorageInit {
        name,
        levels
    };

    let accounts = ctx.accounts.to_account_metas(None);
    let data = args.try_to_vec().unwrap();
    let ix = vec![
        Instruction {
            program_id: program::CodeVm::id(),
            accounts,
            data: [
                instruction::VmStorageInit::DISCRIMINATOR.to_vec(),
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
