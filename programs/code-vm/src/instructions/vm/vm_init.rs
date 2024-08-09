use anchor_lang::{prelude::*, Discriminator};
use anchor_lang::solana_program::instruction::Instruction;
use anchor_spl::token::{TokenAccount, Mint, Token};

use crate::advance_poh;
use crate::error::CodeVmError;
use crate::{
    utils,
    program,
    instruction, 
    cvm::{
        CodeVm,
        CodeVmAccount,
        CodeVmAccountWithChangeLog,
    }, 
    types::Hash,
    CODE_VM_PREFIX
};

#[derive(Accounts)]
#[instruction(lock_duration: u8)]
pub struct CodeVmInit<'info> {
    #[account(mut)]
    pub vm_authority: Signer<'info>,

    #[account(
        init,
        seeds=[
            CODE_VM_PREFIX.as_bytes(),
            mint.to_account_info().key.as_ref(),
            vm_authority.key.as_ref(),
            lock_duration.to_le_bytes().as_ref(),
        ],
        payer = vm_authority,
        space = std::mem::size_of::<CodeVmAccountWithChangeLog>(),
        bump
    )]
    pub vm: Box<Account<'info, CodeVmAccount>>,

    #[account(
        init,
        payer = vm_authority,

        token::mint = mint,
        token::authority = omnibus,

        seeds=[
            CODE_VM_PREFIX.as_bytes(),
            b"vm_omnibus",
            mint.to_account_info().key.as_ref(),
            vm_authority.key.as_ref(),
            lock_duration.to_le_bytes().as_ref(),
        ],
        bump,
    )]
    pub omnibus: Box<Account<'info, TokenAccount>>,

    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn cvm_init(ctx: Context<CodeVmInit>, lock_duration: u8) -> Result<()> {
    require!(lock_duration > 0, CodeVmError::InvalidLockDuration);

    let mut vm = CodeVm::new(ctx.accounts.vm.to_owned());

    ctx.accounts.vm.authority = ctx.accounts.vm_authority.key();
    ctx.accounts.vm.mint = ctx.accounts.mint.key();
    ctx.accounts.vm.lock_duration = lock_duration;
    ctx.accounts.vm.bump = ctx.bumps.vm;
    ctx.accounts.vm.omnibus.vault = ctx.accounts.omnibus.key();
    ctx.accounts.vm.omnibus.vault_bump = ctx.bumps.omnibus;

    // Advance the vm state to include this instruction
    advance_poh!(ctx, vm, 
        get_message_hash(&vm, &ctx),
        None
    );

    Ok(())
}

fn get_message_hash(
    vm: &CodeVm,
    ctx: &Context<CodeVmInit>,
) -> Hash {

    let blockhash = vm.get_current_poh();
    let args = instruction::VmInit {
        lock_duration: vm.get_lock_duration(),
    };

    let accounts = ctx.accounts.to_account_metas(None);
    let data = args.try_to_vec().unwrap();
    let ix = vec![
        Instruction {
            program_id: program::CodeVm::id(),
            accounts,
            data: [
                instruction::VmInit::DISCRIMINATOR.to_vec(),
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
