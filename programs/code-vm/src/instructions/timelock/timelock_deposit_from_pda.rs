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
        MemoryAccountWithData,
        MemoryAccount,
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
    bump: u8,
)]
pub struct TimelockDepositFromPda<'info> {
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

    /// CHECK: Owner of the deposit, must match the VTA at the account_index
    #[account()]
    pub depositor: AccountInfo<'info>,

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

    /// Destination token account
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

    #[account(address = TOKEN_PROGRAM_ID)]
    pub token_program: Option<Program<'info, Token>>,
}

pub fn timelock_deposit_from_pda(
    ctx: Context<TimelockDepositFromPda>, 
    account_index: u16,
    amount: u64,
    bump: u8,
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

    let seeds = [
        CODE_VM_PREFIX.as_bytes(),
        b"vm_deposit_pda",
        vta.owner.as_ref(),
        ctx.accounts.vm.to_account_info().key.as_ref(),
        &[bump],
    ];
    let signer = &[&seeds[..]];

    let cpi_accounts = Transfer {
        from: ctx.accounts.deposit_ata.to_account_info(),
        to: ctx.accounts.vm_omnibus.to_account_info(),
        authority: ctx.accounts.deposit_pda.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.as_ref().unwrap().to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    transfer(cpi_ctx, amount)?;

    // TODO: Range check the amount
    vta.balance += amount;

    vm.try_write_account(account_index, VirtualAccount::Timelock(vta))?;

    // Advance the vm state to include this instruction
    advance_poh!(ctx, vm,
        get_message_hash(
            &vm, 
            &ctx, 
            account_index, 
            amount, 
            bump
        ),
        Some(ChangeLogData::Deposit {
            account: vta,
            amount,
        })
    );

    Ok(())
}

fn get_message_hash(
    vm: &CodeVm,
    ctx: &Context<TimelockDepositFromPda>,
    account_index: u16,
    amount: u64,
    bump: u8,
) -> Hash {

    let blockhash = vm.get_current_poh();
    let args = instruction::TimelockDepositFromPda {
        account_index,
        amount,
        bump
    };

    let accounts = ctx.accounts.to_account_metas(None);
    let data = args.try_to_vec().unwrap();
    let ix = vec![
        Instruction {
            program_id: program::CodeVm::id(),
            accounts,
            data: [
                instruction::TimelockDepositFromPda::DISCRIMINATOR.to_vec(),
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