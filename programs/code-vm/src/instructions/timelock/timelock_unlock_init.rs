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
#[instruction(
    virtual_account_bump: u8,
)]
pub struct TimelockUnlockInit<'info> {
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

    /// CHECK: This account is not a real account on Solana, only exists in the
    /// VM. This is a special PDA as it uses the path from the original timelock
    /// program.
    /// 
    /// Refer to the original timelock program for the seeds used
    /// https://github.com/code-payments/code-program-library/blob/282fbeeababe787ffad932304482b9e53b7ce12b/timelock/programs/timelock/src/context.rs#L25-L32
    #[account(
        seeds=[
            b"timelock_state",
            vm.mint.as_ref(),
            vm.authority.as_ref(),
            virtual_account_owner.key.as_ref(),
            vm.lock_duration.to_le_bytes().as_ref(),
        ],
        bump = virtual_account_bump,

        seeds::program = timelock::ID, // <- Using the timelock program ID here, not the VM ID
    )]
    pub virtual_account: UncheckedAccount<'info>,

    /// This account contains the timelock state; for example: Locked, Unlocked, etc.
    /// It is a real account that may or may not exist but is checked any time
    /// the associated virtual account is mutated.
    #[account(
        init, 
        seeds = [
            CODE_VM_PREFIX.as_bytes(),
            b"vm_unlock_pda_account",
            virtual_account_owner.key.as_ref(),
            virtual_account.key.as_ref(),
            vm.to_account_info().key.as_ref(),
        ], 
        payer = payer, 
        space = UnlockStateAccount::LEN,
        bump, 
    )]
    pub unlock_pda: Account<'info, UnlockStateAccount>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn timelock_unlock_init(
    ctx: Context<TimelockUnlockInit>,
    virtual_account_bump: u8,
) -> Result<()> {
    require!(ctx.accounts.unlock_pda.state == TimelockState::Unknown, CodeVmError::InvalidTimelockState);

    let mut vm = CodeVm::new(ctx.accounts.vm.to_owned());

    ctx.accounts.unlock_pda.vm = ctx.accounts.vm.key();
    ctx.accounts.unlock_pda.owner = ctx.accounts.virtual_account_owner.key();
    ctx.accounts.unlock_pda.address = ctx.accounts.virtual_account.key();
    ctx.accounts.unlock_pda.bump = ctx.bumps.unlock_pda;
    ctx.accounts.unlock_pda.state = TimelockState::Locked;
    ctx.accounts.unlock_pda.unlock_at = None;

    // Advance the vm state to include this instruction
    advance_poh!(ctx, vm, 
        get_message_hash(&vm, &ctx, virtual_account_bump),
        None
    );

    Ok(())
}

fn get_message_hash(
    vm: &CodeVm,
    ctx: &Context<TimelockUnlockInit>,
    virtual_account_bump: u8,
) -> Hash {

    let blockhash = vm.get_current_poh();
    let args = instruction::TimelockUnlockInit {
        virtual_account_bump,
    };

    let accounts = ctx.accounts.to_account_metas(None);
    let data = args.try_to_vec().unwrap();
    let ix = vec![
        Instruction {
            program_id: program::CodeVm::id(),
            accounts,
            data: [
                instruction::TimelockUnlockInit::DISCRIMINATOR.to_vec(),
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