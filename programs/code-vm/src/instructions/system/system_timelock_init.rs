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
        MemoryAccountWithData,
        VirtualAccount,
        VirtualTimelockAccount,
        ChangeLogData,
    },
    types::Hash,
    CODE_VM_PREFIX,
};

#[derive(Accounts)]
#[instruction(
    account_index: u16,
    virtual_timelock_bump: u8,
    virtual_vault_bump: u8,
    unlock_pda_bump: u8,
)]
pub struct CodeVmVirtualTimelockInit<'info> {
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

    /// CHECK: The timelock program doesn't require the virtual account owner to
    /// be a signer
    pub virtual_account_owner: AccountInfo<'info>,
}

pub fn cvm_vta_init(
    ctx: Context<CodeVmVirtualTimelockInit>,
    account_index: u16,
    virtual_timelock_bump: u8,
    virtual_vault_bump: u8,
    unlock_pda_bump: u8,
) -> Result<()> {

    let info = ctx.accounts.vm_memory.to_account_info();
    let data = info.try_borrow_mut_data()?;
    let memory = MemoryAccountWithData::into_indexed_memory(data);
    let mut vm = CodeVm::new(ctx.accounts.vm.to_owned());

    vm.use_memory(memory);
    require!(vm.is_empty(account_index), CodeVmError::VirtualAccountAlreadyAllocated);

    let owner = ctx.accounts.virtual_account_owner.key();
    let nonce = vm.get_current_poh();

    let timelock_address = utils::create_virtual_timelock_address(
        vm.get_mint(), 
        vm.get_authority(), 
        owner, 
        vm.get_lock_duration(), 
        virtual_timelock_bump,
    );

    let unlock_address = utils::create_unlock_address(
        owner, 
        timelock_address, 
        vm.get_address(), 
        unlock_pda_bump);

    // We could technically require the user to provide the withdraw_bump,
    // however, that would make using this instruction more cumbersome since the
    // nonce value is determined above.
    let (_, withdraw_bump) = utils::find_withdraw_receipt_address( // This call *can* be expensive
        unlock_address, 
        nonce, 
        vm.get_address());

    let vta = VirtualTimelockAccount {
        owner,
        nonce,
        bump: virtual_timelock_bump,
        token_bump: virtual_vault_bump,
        unlock_bump: unlock_pda_bump,
        withdraw_bump,
        balance: 0,
    };
    let va = VirtualAccount::Timelock(vta);

    vm.try_write_account(account_index, va)?;

    // Advance the vm state to include this instruction
    advance_poh!(ctx, vm, 
        get_message_hash(
            &vm,
            &ctx, 
            account_index,
            virtual_timelock_bump,
            virtual_vault_bump,
            unlock_pda_bump,
        ),
        Some(ChangeLogData::Create(va))
    );

    Ok(())
}

fn get_message_hash(
    vm: &CodeVm,
    ctx: &Context<CodeVmVirtualTimelockInit>,
    account_index: u16,
    virtual_timelock_bump: u8,
    virtual_vault_bump: u8,
    unlock_pda_bump: u8,
) -> Hash {

    let blockhash = vm.get_current_poh();
    let args = instruction::SystemTimelockInit {
        account_index,
        virtual_timelock_bump,
        virtual_vault_bump,
        unlock_pda_bump,
    };

    let accounts = ctx.accounts.to_account_metas(None);
    let data = args.try_to_vec().unwrap();
    let ix = vec![
        Instruction {
            program_id: program::CodeVm::id(),
            accounts,
            data: [
                instruction::SystemTimelockInit::DISCRIMINATOR.to_vec(),
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