use anchor_lang::{prelude::*, Discriminator};
use anchor_lang::solana_program::instruction::Instruction;
use anchor_spl::token::{TokenAccount, Token, Transfer, transfer, ID as TOKEN_PROGRAM_ID};

use crate::advance_poh;
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
#[instruction(
    bump: u8,
)]
pub struct TimelockWithdrawFromDeposit<'info> {
    /// CHECK: Owner of the deposit
    #[account()]
    pub depositor: Signer<'info>,

    /// CHECK: Deposit address (off-curve)
    #[account( 
        seeds=[
            CODE_VM_PREFIX.as_bytes(),
            b"vm_deposit_pda",
            depositor.to_account_info().key.as_ref(),
            vm.to_account_info().key.as_ref(),
        ],
        bump = bump,
    )]
    pub deposit_pda: AccountInfo<'info>,

    /// Source token account (ATA off of the deposit address)
    /// (Note: This setup allows us to get around needing the depositor as a
    /// signer)
    #[account(
        mut,
        associated_token::mint = vm.mint,
        associated_token::authority = deposit_pda, 
    )]
    pub deposit_ata: Box<Account<'info, TokenAccount>>,

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
        constraint = unlock_pda.owner == depositor.key(),
        constraint = unlock_pda.state == TimelockState::Unlocked,

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

    /// The external address is used for external transfers. This is the
    /// account that tokens are transferred to.
    #[account(mut,
        token::mint = vm.mint,
    )]
    pub external_address: Account<'info, TokenAccount>,

    #[account(address = TOKEN_PROGRAM_ID)]
    pub token_program: Program<'info, Token>,
}

pub fn timelock_withdraw_from_deposit(
    ctx: Context<TimelockWithdrawFromDeposit>,
    bump: u8,
) -> Result<()> {

    // Note, this instruction is a special case. It can be called by the
    // depositor as many times as they want, as long as the unlock_pda is in
    // unlocked state.

    // Normally, a withdraw receipt would be issued, but in this instruction, we
    // don't need to as the tokens were never in a virtual account to begin
    // with. Additionally, it is possible for wallets to keep sending tokens to
    // the deposit address after a withdraw, so we must make it possible for the
    // deposit owner to withdraw them from here as many times as they want.

    let mut vm = CodeVm::new(ctx.accounts.vm.to_owned());

    let destination = ctx.accounts.external_address.to_account_info();
    let token_program = ctx.accounts.token_program.to_account_info();

    let seeds = [
        CODE_VM_PREFIX.as_bytes(),
        b"vm_deposit_pda",
        ctx.accounts.unlock_pda.owner.as_ref(),
        ctx.accounts.vm.to_account_info().key.as_ref(),
        &[bump],
    ];
    let signer_seeds = &[&seeds[..]];

    let cpi_accounts = Transfer {
        from: ctx.accounts.deposit_ata.to_account_info(),
        to: destination,
        authority: ctx.accounts.deposit_pda.to_account_info(),
    };
    let cpi_program = token_program;
    let cpi_ctx = CpiContext::new_with_signer(
        cpi_program, 
        cpi_accounts, 
        signer_seeds
    );

    let amount = ctx.accounts.deposit_ata.amount;

    transfer(cpi_ctx, amount)?;

    // Advance the vm state to include this instruction
    advance_poh!(ctx, vm, 
        get_message_hash(
            &vm,
            &ctx, 
            bump,
        ),
        Some(ChangeLogData::Withdraw {
            account: None,
            signature: None,
            src: ctx.accounts.depositor.key(),
            dst: ctx.accounts.external_address.key(),
            amount: amount,
        })
    );

    Ok(())
}

fn get_message_hash(
    vm: &CodeVm,
    ctx: &Context<TimelockWithdrawFromDeposit>,
    bump: u8,
) -> Hash {

    let blockhash = vm.get_current_poh();
    let args = instruction::TimelockWithdrawFromDeposit {
        bump
    };

    let accounts = ctx.accounts.to_account_metas(None);
    let data = args.try_to_vec().unwrap();
    let ix = vec![
        Instruction {
            program_id: program::CodeVm::id(),
            accounts,
            data: [
                instruction::TimelockWithdrawFromDeposit::DISCRIMINATOR.to_vec(),
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