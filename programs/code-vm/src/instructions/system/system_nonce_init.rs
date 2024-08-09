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
        VirtualDurableNonce,
        ChangeLogData,
    },
    types::Hash,
    CODE_VM_PREFIX,
};

#[derive(Accounts)]
#[instruction(
    account_index: u16,
)]
pub struct CodeVmVirtualNonceInit<'info> {
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

    /// CHECK: Within the SVM, the virtual account owner would be the owner of
    /// the durable nonce account. This is not the case here, as the durable
    /// nonce account is owned by the vm authority. This is done to simplify the
    /// implementation and guarantee uniqueness accross all memory buffers.
    pub virtual_account_owner: AccountInfo<'info>,
}

pub fn cvm_vdn_init(
    ctx: Context<CodeVmVirtualNonceInit>,
    account_index: u16,
) -> Result<()> {

    let info = ctx.accounts.vm_memory.to_account_info();
    let data = info.try_borrow_mut_data()?;
    let memory = MemoryAccountWithData::into_indexed_memory(data);
    let mut vm = CodeVm::new(ctx.accounts.vm.to_owned());

    vm.use_memory(memory);
    require!(vm.is_empty(account_index), CodeVmError::VirtualAccountAlreadyAllocated);

    // Note: Within the CodeVM, the nonce address is derived from the nonce_seed
    // and the current blockhash. We do this to guarantee that the nonce account
    // is unique and deterministic within the entire vm state. Within the SVM,
    // the address would be equivalent to the nonce_seed above.

    let nonce_seed = ctx.accounts.virtual_account_owner.key();
    let nonce_value = vm.get_current_poh();
    let nonce_address: Pubkey = utils::hashv(&[
        nonce_seed.as_ref(), 
        nonce_value.as_ref()
    ]).into();

    // Note: we're not doing signature verification on the owner, it is
    // unnecessary as all durable nonces in the vm are owned by the vm authority
    // (which has signed the outer context).

    let vdn = VirtualDurableNonce {
        address: nonce_address,
        nonce: nonce_value,
    };
    let va = VirtualAccount::Nonce(vdn);

    vm.try_write_account(account_index, va)?;

    // Advance the vm state to include this instruction
    advance_poh!(ctx, vm, 
        get_message_hash(&vm, &ctx, account_index),
        Some(ChangeLogData::Create(va))
    );

    Ok(())
}

fn get_message_hash(
    vm: &CodeVm,
    ctx: &Context<CodeVmVirtualNonceInit>,
    account_index: u16,
) -> Hash {

    let blockhash = vm.get_current_poh();
    let args = instruction::SystemNonceInit {
        account_index,
    };

    let accounts = ctx.accounts.to_account_metas(None);
    let data = args.try_to_vec().unwrap();
    let ix = vec![
        Instruction {
            program_id: program::CodeVm::id(),
            accounts,
            data: [
                instruction::SystemNonceInit::DISCRIMINATOR.to_vec(),
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