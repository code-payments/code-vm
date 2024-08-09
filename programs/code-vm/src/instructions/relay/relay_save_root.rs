use anchor_lang::{prelude::*, Discriminator};
use anchor_lang::solana_program::instruction::Instruction;

use crate::advance_poh;
use crate::{
    utils,
    program,
    instruction,
    cvm::{ 
        CodeVm, 
        CodeVmAccount, 
        RelayAccount,
    },
    types::Hash,
    CODE_VM_PREFIX
};

#[derive(Accounts)]
#[instruction()]
pub struct RelaySaveRoot<'info> {
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
            b"vm_relay_account",
            relay.name.as_bytes(),
            vm.to_account_info().key.as_ref(),
        ], 
        bump = relay.bump
    )]
    pub relay: Account<'info, RelayAccount>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn relay_save_root(
    ctx: Context<RelaySaveRoot>,
) -> Result<()> {

    let mut vm = CodeVm::new(ctx.accounts.vm.to_owned());
    
    let current = ctx.accounts.relay.history.get_root();
    let last = ctx.accounts.relay.recent_roots.last();

    match last {
        None => {
            // There is no last root, proceed to save the current root
        }
        Some(last) => {
            // We have a last root, check if it is the same as the current root
            if current.as_ref().eq(last) {
                // The root is already saved
                return Ok(());
            }
        },
    };

    ctx.accounts.relay.recent_roots.push(current.as_ref());

    // Advance the vm state to include this instruction
    advance_poh!(ctx, vm, 
        get_message_hash(
            &vm,
            &ctx,
        ),
        None
    ); 

    Ok(())
}

fn get_message_hash(
    vm: &CodeVm,
    ctx: &Context<RelaySaveRoot>,
) -> Hash {

    let blockhash = vm.get_current_poh();
    let args = instruction::RelaySaveRoot {
    };

    let accounts = ctx.accounts.to_account_metas(None);
    let data = args.try_to_vec().unwrap();
    let ix = vec![
        Instruction {
            program_id: program::CodeVm::id(),
            accounts,
            data: [
                instruction::RelaySaveRoot::DISCRIMINATOR.to_vec(),
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