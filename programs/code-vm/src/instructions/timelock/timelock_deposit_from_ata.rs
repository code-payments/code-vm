use anchor_lang::{prelude::*, Discriminator};
use anchor_lang::solana_program::instruction::Instruction;
use anchor_spl::token::{TokenAccount, Token, Transfer, transfer, ID as TOKEN_PROGRAM_ID};

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
        MemoryAccountWithData,
        VirtualAccount,
        ChangeLogData,
    },
    types::Hash,
    CODE_VM_PREFIX
};

#[derive(Accounts)]
#[instruction(
    account_index: u16,
    amount: u64,
)]
pub struct TimelockDepositFromAta<'info> {
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

        token::mint = vm.mint,
        token::authority = vm_omnibus,

        seeds=[
            CODE_VM_PREFIX.as_bytes(),
            b"vm_omnibus",
            vm.mint.as_ref(),
            vm.authority.as_ref(),
            vm.lock_duration.to_le_bytes().as_ref(),
        ],
        bump,
    )]
    pub vm_omnibus: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = vm.mint,
        associated_token::authority = depositor,
    )]
    pub depositor_ata: Box<Account<'info, TokenAccount>>,
    pub depositor: Signer<'info>,

    #[account(address = TOKEN_PROGRAM_ID)]
    pub token_program: Option<Program<'info, Token>>,
}

pub fn timelock_deposit_from_ata(
    ctx: Context<TimelockDepositFromAta>, 
    account_index: u16, 
    amount: u64,
) -> Result<()> {
    require!(amount > 0, CodeVmError::InvalidDepositAmount);

    let info = ctx.accounts.vm_memory.to_account_info();
    let data = info.try_borrow_mut_data()?;
    let memory = MemoryAccountWithData::into_indexed_memory(data);

    let mut vm = CodeVm::new(ctx.accounts.vm.to_owned());

    vm.use_memory(memory);
    require!(vm.is_allocated(account_index), CodeVmError::VirtualAccountNotAllocated);

    let mut vta = vm.read_account(account_index)
        .unwrap()
        .into_inner_timelock()
        .unwrap();

    assert_eq!(vta.owner, ctx.accounts.depositor.key());

    // TODO: validate that the account has the right timelock address, owner, etc.

    let cpi_accounts = Transfer {
        from: ctx.accounts.depositor_ata.to_account_info(),
        to: ctx.accounts.vm_omnibus.to_account_info(),
        authority: ctx.accounts.depositor.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.as_ref().unwrap().to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    transfer(cpi_ctx, amount)?;

    // TODO: Range check the amount

    vta.balance += amount;

    vm.try_write_account(account_index, VirtualAccount::Timelock(vta))?;

    // Advance the vm state to include this instruction
    vm.advance_poh(get_message_hash(&vm, &ctx, account_index, amount));
    ctx.accounts.vm.slot = vm.advance_slot();
    ctx.accounts.vm.poh  = vm.get_current_poh();

    advance_poh!(ctx, vm,
        get_message_hash(&vm, &ctx, account_index, amount),
        Some(ChangeLogData::Deposit {
            account: vta,
            amount,
        })
    );

    Ok(())
}

fn get_message_hash(
    vm: &CodeVm,
    ctx: &Context<TimelockDepositFromAta>,
    account_index: u16,
    amount: u64,
) -> Hash {

    let blockhash = vm.get_current_poh();
    let args = instruction::TimelockDepositFromAta {
        account_index,
        amount,
    };

    let accounts = ctx.accounts.to_account_metas(None);
    let data = args.try_to_vec().unwrap();
    let ix = vec![
        Instruction {
            program_id: program::CodeVm::id(),
            accounts,
            data: [
                instruction::TimelockDepositFromAta::DISCRIMINATOR.to_vec(),
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