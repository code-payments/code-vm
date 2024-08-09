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
        CompressedStorageAccount,
        MemoryAccount,
        MemoryAccountWithData,
        VirtualAccount,
        VirtualDurableNonce,
        VirtualTimelockAccount,
        VirtualRelayAccount,
        ChangeLogData,
    },
    types::{Hash, Signature},
    CODE_VM_PREFIX,
};

#[derive(Accounts)]
#[instruction(
    account_index: u16,
    packed_va: Vec<u8>,
    proof: Vec<Hash>,
    signature: Signature,
)]
pub struct CodeVmVirtualAccountDecompress<'info> {
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
        seeds = [
            CODE_VM_PREFIX.as_bytes(),
            b"vm_storage_account",
            &vm_storage.name.as_bytes(),
            vm.to_account_info().key.as_ref(),
        ], 
        bump = vm_storage.bump, 
    )]
    pub vm_storage: Account<'info, CompressedStorageAccount>,

    /// This is required if uncompressing a virtual timelock account.
    /// 
    /// Note: We expect this to be uninitialized in the happy path, so we can't
    /// use Account<'info, TimelockStateAccount> as that would trigger a check
    /// that the account exists.
    ///
    /// Note: If this account exists and has a non-locked state, then the
    /// decompress instruction will fail.
    #[account(
        // The seeds for this account are in a virtual account, so we can't use
        // anchor to validate this address.
    )]
    pub unlock_pda: Option<AccountInfo<'info>>,

    /// This account is used to prove that a user has not non-custodially
    /// withdrawn tokens from a virtual account.
    /// 
    /// Note: If this account exists, then the decompress instruction will fail.
    /// 
    /// CHECK: This account is expected to be empty in the happy path. When it
    /// is not empty, we can assume a non-custodial withdraw was made. If it is
    /// initialized, then this instruction should fail.
    #[account(
        // The seeds for this account are in a virtual account, so we can't use
        // anchor to validate this address.
    )]
    pub withdraw_receipt: Option<AccountInfo<'info>>,
}

pub fn cvm_va_decompress(
    ctx: Context<CodeVmVirtualAccountDecompress>,
    account_index: u16,
    packed_va: Vec<u8>,
    proof: Vec<Hash>,
    signature: Signature,
) -> Result<()> {

    let info = ctx.accounts.vm_memory.to_account_info();
    let data = info.try_borrow_mut_data()?;
    let memory = MemoryAccountWithData::into_indexed_memory(data);
    let mut vm = CodeVm::new(ctx.accounts.vm.to_owned());

    vm.use_memory(memory);

    let unchecked_va = VirtualAccount::unpack(packed_va.as_ref())?;
    require!(match unchecked_va {
        VirtualAccount::Timelock(vta) => {
            verify_timelock_account(&ctx, &vm, account_index, vta)
        }
        VirtualAccount::Nonce(vdn) => {
            verify_durable_nonce(&ctx, &vm, account_index, vdn)
        }
        VirtualAccount::Relay(vra) => {
            verify_relay_account(&ctx, &vm, account_index, vra)
        }
    }.is_ok(), CodeVmError::InvalidVirtualAccount);

    let va = unchecked_va;
    let tree = &mut ctx.accounts.vm_storage.memory_state;

    vm.try_decompress(unchecked_va, tree, proof.clone(), signature)?;
    vm.try_write_account(account_index, va)?;

    // Advance the vm state to include this instruction
    advance_poh!(ctx, vm, 
        get_message_hash(
            &vm,
            &ctx,
            account_index,
            packed_va,
            proof,
            signature,
        ),
        Some(ChangeLogData::Decompress {
            store: ctx.accounts.vm_storage.key(),
            account: va,
            signature,
        })
    ); 

    Ok(())
}

fn verify_durable_nonce(
    _ctx: &Context<CodeVmVirtualAccountDecompress>,
    _vm: &CodeVm,
    _account_index: u16,
    _unchecked_vdn: VirtualDurableNonce,
) -> Result<()> {

    // No checks are required for durable nonces beyond the merkle proof

    // TODO: verify that this is true

    Ok(())
}

fn verify_relay_account(
    _ctx: &Context<CodeVmVirtualAccountDecompress>,
    _vm: &CodeVm,
    _account_index: u16,
    _unchecked_vra: VirtualRelayAccount,
) -> Result<()> {

    // No checks are required for relay accounts beyond the merkle proof

    // TODO: verify that this is true

    Ok(())
}

fn verify_timelock_account(
    ctx: &Context<CodeVmVirtualAccountDecompress>,
    vm: &CodeVm,
    _account_index: u16,
    unchecked_vta: VirtualTimelockAccount,
) -> Result<()> {

    let unlock_pda = ctx.accounts.unlock_pda.as_ref();
    let receipt = ctx.accounts.withdraw_receipt.as_ref();

    assert!(unlock_pda.is_some());
    assert!(receipt.is_some());

    let unlock_pda = unlock_pda.unwrap();
    let receipt = receipt.unwrap();

    vm.try_verify_timelock_account(unchecked_vta, unlock_pda, receipt)
}


fn get_message_hash(
    vm: &CodeVm,
    ctx: &Context<CodeVmVirtualAccountDecompress>,
    account_index: u16,
    packed_va: Vec<u8>,
    proof: Vec<Hash>,
    signature: Signature,
) -> Hash {

    let blockhash = vm.get_current_poh();
    let args = instruction::SystemAccountDecompress {
        account_index,
        packed_va,
        proof,
        signature,
    };

    let accounts = ctx.accounts.to_account_metas(None);
    let data = args.try_to_vec().unwrap();
    let ix = vec![
        Instruction {
            program_id: program::CodeVm::id(),
            accounts,
            data: [
                instruction::SystemAccountDecompress::DISCRIMINATOR.to_vec(),
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