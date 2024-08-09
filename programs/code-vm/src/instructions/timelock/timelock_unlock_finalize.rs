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
        UnlockStateAccount,
        TimelockState,
    },
    types::Hash,
    CODE_VM_PREFIX
};

#[derive(Accounts)]
pub struct TimelockUnlockFinalize<'info> {
    pub virtual_account_owner: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut, // the POH value is updated
        seeds=[
            CODE_VM_PREFIX.as_bytes(),
            vm.mint.as_ref(),
            vm.authority.as_ref(),
            vm.lock_duration.to_le_bytes().as_ref(),
        ],
        bump = vm.bump
    )]
    pub vm: Box<Account<'info, CodeVmAccount>>,

    /// This account contains the timelock state; for example: Locked, Unlocked, etc.
    /// It is a real account that may or may not exist but is checked any time
    /// the associated virtual account is mutated.
    #[account(
        mut, 
        constraint = unlock_pda.owner == virtual_account_owner.key(),
        constraint = unlock_pda.unlock_at.is_some(),
        constraint = unlock_pda.state == TimelockState::WaitingForTimeout,

        seeds = [
            CODE_VM_PREFIX.as_bytes(),
            b"vm_unlock_pda_account",
            unlock_pda.owner.as_ref(),
            unlock_pda.address.as_ref(),
            vm.to_account_info().key.as_ref(),
        ], 
        bump = unlock_pda.bump, 
    )]
    pub unlock_pda: Account<'info, UnlockStateAccount>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}


pub fn timelock_unlock_finalize(
    ctx: Context<TimelockUnlockFinalize>,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;

    if ctx.accounts.unlock_pda.state != TimelockState::WaitingForTimeout {
        return Err(CodeVmError::InvalidTimelockState.into());
    }

    if ctx.accounts.unlock_pda.unlock_at.is_none() {
        return Err(CodeVmError::InvalidTimelockState.into());
    }

    if ctx.accounts.unlock_pda.unlock_at.unwrap() > now {
        msg!("Cannot deactivate lock just yet.");
        msg!("The current time is: {}", now);
        msg!("The earliest unlock at: {}", ctx.accounts.unlock_pda.unlock_at.unwrap());

        return Err(CodeVmError::InvalidTimelockState.into());
    }

    let mut vm = CodeVm::new(ctx.accounts.vm.to_owned());

    ctx.accounts.unlock_pda.state = TimelockState::Unlocked;
    ctx.accounts.unlock_pda.unlock_at = None;

    // Advance the vm state to include this instruction
    advance_poh!(ctx, vm, 
        get_message_hash(&vm, &ctx),
        None
    );

    Ok(())
}


fn get_message_hash(
    vm: &CodeVm,
    ctx: &Context<TimelockUnlockFinalize>,
) -> Hash {

    let blockhash = vm.get_current_poh();
    let args = instruction::TimelockUnlockFinalize {
    };

    let accounts = ctx.accounts.to_account_metas(None);
    let data = args.try_to_vec().unwrap();
    let ix = vec![
        Instruction {
            program_id: program::CodeVm::id(),
            accounts,
            data: [
                instruction::TimelockUnlockFinalize::DISCRIMINATOR.to_vec(),
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