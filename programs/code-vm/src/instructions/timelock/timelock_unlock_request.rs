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
        TimelockState,
        UnlockStateAccount,
        ChangeLogData,
    },
    types::Hash,
    CODE_VM_PREFIX
};

#[derive(Accounts)]
pub struct TimelockUnlockRequest<'info> {
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


pub fn timelock_unlock_request(
    ctx: Context<TimelockUnlockRequest>,
) -> Result<()> {
    require!(ctx.accounts.unlock_pda.state == TimelockState::Locked, CodeVmError::InvalidTimelockState);

    let mut vm = CodeVm::new(ctx.accounts.vm.to_owned());

    let now = Clock::get()?.unix_timestamp;
    let second_per_day = 86400; // 60sec * 60min * 24hrs = 86400
    let mut unlock_at = now + (ctx.accounts.vm.lock_duration as i64 * second_per_day); 
    if unlock_at % second_per_day > 0 {
        // Bump the unlock time to the start of the next UTC day if we haven't
        // landed on it. This drastically simplifies searching for newly unlocked
        // accounts.
        unlock_at = unlock_at + (second_per_day - (unlock_at % second_per_day))
    }

    msg!("The unlock time is set for: {}", unlock_at);

    ctx.accounts.unlock_pda.state = TimelockState::WaitingForTimeout;
    ctx.accounts.unlock_pda.unlock_at = Some(unlock_at);

    // Advance the vm state to include this instruction
    advance_poh!(ctx, vm, 
        get_message_hash(&vm, &ctx),
        Some(ChangeLogData::Unlock { 
            unlock_pda: ctx.accounts.unlock_pda.to_account_info().key(), 
            owner: ctx.accounts.virtual_account_owner.key(),
            address: ctx.accounts.unlock_pda.address, 
        })
    );

    Ok(())
}


fn get_message_hash(
    vm: &CodeVm,
    ctx: &Context<TimelockUnlockRequest>,
) -> Hash {

    let blockhash = vm.get_current_poh();
    let args = instruction::TimelockUnlockRequest {
    };

    let accounts = ctx.accounts.to_account_metas(None);
    let data = args.try_to_vec().unwrap();
    let ix = vec![
        Instruction {
            program_id: program::CodeVm::id(),
            accounts,
            data: [
                instruction::TimelockUnlockRequest::DISCRIMINATOR.to_vec(),
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