use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    system_program,
    system_instruction,
};
use borsh::{ BorshDeserialize, BorshSerialize };

use crate::log_event;
use crate::error::CodeVmError;
use crate::{ 
    types::Hash, 
    utils::{self, sig_verify}, 
    instructions::CodeVmExec,
    cvm::{
        CodeVm, 
        MemoryBank,
        VirtualAccount,
        VirtualDurableNonce,
        VirtualTimelockAccount,
        ChangeLogData,
    }
};

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[repr(C)]
struct OpcodeData {
    signature: [u8; 64],
}

pub fn withdraw_to_internal (
    ctx: Context<CodeVmExec>,
    mem_indicies: Vec<u16>,
    mem_banks: Vec<u8>,
    data: Vec<u8>,
) -> Result<()> {

    // This action requires a virtual durable nonce and two virtual timelock
    // accounts.
    assert_eq!(mem_indicies.len(), 3);
    assert_eq!(mem_banks.len(), 3);

    let nonce_index = mem_indicies[0];
    let nonce_mem = MemoryBank::from(mem_banks[0]);

    let src_index = mem_indicies[1];
    let src_mem = MemoryBank::from(mem_banks[1]);

    let dst_index = mem_indicies[2];
    let dst_mem = MemoryBank::from(mem_banks[2]);

    let mut vm = CodeVm::new(ctx.accounts.vm.to_owned());

    // Use the memory banks if they are provided
    if let Some(info) = ctx.accounts.mem_a.as_ref() {
        vm.try_use_memory_bank(MemoryBank::A, info)?;
    }

    if let Some(info) = ctx.accounts.mem_b.as_ref() {
        vm.try_use_memory_bank(MemoryBank::B, info)?;
    }

    if let Some(info) = ctx.accounts.mem_c.as_ref() {
        vm.try_use_memory_bank(MemoryBank::C, info)?;
    }

    if let Some(info) = ctx.accounts.mem_d.as_ref() {
        vm.try_use_memory_bank(MemoryBank::D, info)?;
    }

    // Check that the correct memory banks are properly set for the accounts
    // that need them
    require!(vm.has_memory_bank(nonce_mem), CodeVmError::InvalidMemoryBank);
    require!(vm.has_memory_bank(src_mem), CodeVmError::InvalidMemoryBank);
    require!(vm.has_memory_bank(dst_mem), CodeVmError::InvalidMemoryBank);

    // Retrieve the virtual account/nonce indicies and check that they have been
    // allocated. The accounts should be allocated.

    require!(vm.is_allocated_using(nonce_mem, nonce_index), CodeVmError::VirtualAccountNotAllocated);
    require!(vm.is_allocated_using(src_mem, src_index), CodeVmError::VirtualAccountNotAllocated);
    require!(vm.is_allocated_using(dst_mem, dst_index), CodeVmError::VirtualAccountNotAllocated);

    // Read the virtual accounts and nonce from their memory banks
    let mut vdn = vm.read_account_using(
        nonce_mem, 
        nonce_index,
    ).unwrap().into_inner_nonce().unwrap();

    let src_vta = vm.read_account_using(
        src_mem, 
        src_index,
    ).unwrap().into_inner_timelock().unwrap();

    let mut dst_vta = vm.read_account_using(
        dst_mem, 
        dst_index,
    ).unwrap().into_inner_timelock().unwrap();

    // Deserialize the OpCode data
    let opcode_data = OpcodeData::try_from_slice(&data).unwrap();
    let hash = get_message_hash(&vm, 
        &src_vta, 
        &dst_vta, 
        &vdn
    );

    let amount = src_vta.balance;

    // This action requires a signature from the source account
    sig_verify(
        src_vta.owner.as_ref(), 
        opcode_data.signature.as_ref(), 
        hash.as_ref(),
    )?;

    // Advance the nonce
    vdn.nonce = vm.get_current_poh();

    if src_index == dst_index {
        // No need to transfer
    } else {
        dst_vta.balance += amount;
    }

    vm.try_write_account_using(
        dst_mem, 
        dst_index, 
        VirtualAccount::Timelock(dst_vta)
    )?;

    vm.try_write_account_using(
        nonce_mem, 
        nonce_index, 
        VirtualAccount::Nonce(vdn)
    )?;

    vm.try_delete_account_using(
        src_mem, 
        src_index
    )?;

    let src_timelock_address = src_vta.get_timelock_address(
        vm.get_mint(),
        vm.get_authority(),
        vm.get_lock_duration(),
    );
    let src_token_address = src_vta.get_token_address(
        src_timelock_address,
    );

    let dst_timelock_address = dst_vta.get_timelock_address(
        vm.get_mint(),
        vm.get_authority(),
        vm.get_lock_duration(),
    );
    let dst_token_address = dst_vta.get_token_address(
        dst_timelock_address,
    );

    log_event!(ctx, vm, 
        Some(ChangeLogData::Transfer {
            src: src_token_address,
            dst: dst_token_address,
            amount,
        })
    );

    Ok(())
}

fn get_message_hash(
    vm: &CodeVm,
    src_vta: &VirtualTimelockAccount,
    dst_vta: &VirtualTimelockAccount,
    vdn: &VirtualDurableNonce,
) -> Hash {
    let blockhash = vdn.nonce;

    let src_timelock_address = src_vta.get_timelock_address(
        vm.get_mint(),
        vm.get_authority(),
        vm.get_lock_duration(),
    );
    let src_token_address = src_vta.get_token_address(
        src_timelock_address,
    );

    let dst_timelock_address = dst_vta.get_timelock_address(
        vm.get_mint(),
        vm.get_authority(),
        vm.get_lock_duration(),
    );
    let dst_token_address = dst_vta.get_token_address(
        dst_timelock_address,
    );

    let ix = vec![
        system_instruction::advance_nonce_account(
            &vdn.address,
            &vm.get_authority(),
        ),
        utils::memo::build_kre_memo(),
        timelock::revoke_lock_with_authority_ix(
            timelock::RevokeLockWithAuthorityKeys {
                timelock: src_timelock_address,
                vault: src_token_address,
                time_authority: vm.get_authority(),
                payer: vm.get_authority(),
                token_program: anchor_spl::token::ID,
                system_program: system_program::ID,
            },
            timelock::RevokeLockWithAuthorityIxArgs {
                timelock_bump: src_vta.bump,
            }
        ).unwrap(),
        timelock::deactivate_ix(
            timelock::DeactivateKeys {
                timelock: src_timelock_address,
                vault_owner: src_vta.owner,
                payer: vm.get_authority(),
            },
            timelock::DeactivateIxArgs {
                timelock_bump: src_vta.bump,
            }
        ).unwrap(),
        timelock::withdraw_ix(
            timelock::WithdrawKeys {
                timelock: src_timelock_address,
                vault: src_token_address,
                vault_owner: src_vta.owner,
                destination: dst_token_address,
                payer: vm.get_authority(),
                token_program: anchor_spl::token::ID,
                system_program: system_program::ID,
            },
            timelock::WithdrawIxArgs {
                timelock_bump: src_vta.bump,
            }
        ).unwrap(),
        timelock::close_accounts_ix(
            timelock::CloseAccountsKeys {
                timelock: src_timelock_address,
                vault: src_token_address,
                close_authority: vm.get_authority(),
                payer: vm.get_authority(),
                token_program: anchor_spl::token::ID,
                system_program: system_program::ID,
            },
            timelock::CloseAccountsIxArgs {
                timelock_bump: src_vta.bump,
            }
        ).unwrap(),
    ];

    let message = utils::message_with_sorted_keys(
        &ix,
        Some(&vm.get_authority()),
        &blockhash,
    );

    let message = message.serialize();

    utils::hash(&message)
}