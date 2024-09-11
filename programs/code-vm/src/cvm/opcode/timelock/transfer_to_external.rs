use anchor_lang::prelude::*;
use anchor_spl::token::{Transfer, transfer};
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
    },
    CODE_VM_PREFIX,
};

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[repr(C)]
struct OpcodeData {
    signature: [u8; 64],
    amount: u64,
}

pub fn transfer_to_external(
    ctx: Context<CodeVmExec>,
    mem_indicies: Vec<u16>, 
    mem_banks: Vec<u8>, 
    data: Vec<u8>,
) -> Result<()> {

    // This action requires a virtual durable nonce and a virtual timelock
    // account (the destination is a real account).
    assert_eq!(mem_indicies.len(), 2);
    assert_eq!(mem_banks.len(), 2);

    let nonce_index = mem_indicies[0];
    let nonce_mem = MemoryBank::from(mem_banks[0]);

    let src_index = mem_indicies[1];
    let src_mem = MemoryBank::from(mem_banks[1]);

    // This action requires a real destination token account, the token program,
    // and our vm omnibus.
    assert!(ctx.accounts.external_address.is_some());
    assert!(ctx.accounts.token_program.is_some());
    assert!(ctx.accounts.vm_omnibus.is_some());

    let vm_omnibus = ctx.accounts.vm_omnibus.as_ref().unwrap().to_account_info();
    let token_program: AccountInfo = ctx.accounts.token_program.as_ref().unwrap().to_account_info();
    let destination = ctx.accounts.external_address.as_ref().unwrap().to_account_info();

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

    // Retrieve the virtual account/nonce indicies and check that they have been
    // allocated. The accounts should be allocated.

    require!(vm.is_allocated_using(nonce_mem, nonce_index), CodeVmError::VirtualAccountNotAllocated);
    require!(vm.is_allocated_using(src_mem, src_index), CodeVmError::VirtualAccountNotAllocated);

    // Read the virtual accounts and nonce from their memory banks
    let mut vdn = vm.read_account_using(
        nonce_mem, 
        nonce_index,
    ).unwrap().into_inner_nonce().unwrap();

    let mut src_vta = vm.read_account_using(
        src_mem, 
        src_index,
    ).unwrap().into_inner_timelock().unwrap();

    // Deserialize the OpCode data
    let opcode_data = OpcodeData::try_from_slice(&data).unwrap();
    let hash = get_message_hash(&vm, 
        &src_vta, 
        destination.key(), 
        &vdn, 
        opcode_data.amount);

    // This action requires a signature from the source account
    sig_verify(
        src_vta.owner.as_ref(), 
        opcode_data.signature.as_ref(), 
        hash.as_ref(),
    )?;

    let mint = vm.get_mint();
    let authority = vm.get_authority();
    let lock_duration = vm.get_lock_duration().to_le_bytes();

    let seeds = [
        CODE_VM_PREFIX.as_bytes(),
        b"vm_omnibus",
        mint.as_ref(),
        authority.as_ref(),
        lock_duration.as_ref(),
        &[vm.get_omnibus_bump()],
    ];
    let signer_seeds = &[&seeds[..]];

    let cpi_accounts = Transfer {
        authority: vm_omnibus.clone(),
        from: vm_omnibus.clone(),
        to: destination.clone(),
    };
    let cpi_program = token_program;
    let cpi_ctx = CpiContext::new_with_signer(
        cpi_program, 
        cpi_accounts, 
        signer_seeds
    );

    transfer(cpi_ctx, opcode_data.amount)?;

    // Advance the nonce
    vdn.nonce = vm.get_current_poh();

    // TODO: add bounds checking
    src_vta.balance -= opcode_data.amount;

    let src_timelock_address = src_vta.get_timelock_address(
        vm.get_mint(),
        vm.get_authority(),
        vm.get_lock_duration(),
    );
    let src_token_address = src_vta.get_token_address(
        src_timelock_address,
    );

    vm.try_write_account_using(
        src_mem, 
        src_index, 
        VirtualAccount::Timelock(src_vta)
    )?;

    vm.try_write_account_using(
        nonce_mem, 
        nonce_index, 
        VirtualAccount::Nonce(vdn)
    )?;

    log_event!(ctx, vm, 
        Some(ChangeLogData::Transfer {
            src: src_token_address,
            dst: destination.to_account_info().key(),
            amount: opcode_data.amount,
        })
    );

    Ok(())
}

fn get_message_hash(
    vm: &CodeVm,
    src_vta: &VirtualTimelockAccount,
    dst_pubkey: Pubkey,
    vdn: &VirtualDurableNonce,
    amount: u64,
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

    let ix = vec![
        system_instruction::advance_nonce_account(
            &vdn.address,
            &vm.get_authority(),
        ),
        utils::memo::build_kre_memo(),
        timelock::transfer_with_authority_ix(
            timelock::TransferWithAuthorityKeys {
                timelock: src_timelock_address,
                vault: src_token_address,
                vault_owner: src_vta.owner,
                time_authority: vm.get_authority(),
                destination: dst_pubkey,
                payer: vm.get_authority(),
                token_program: anchor_spl::token::ID,
                system_program: system_program::ID,
            },
            timelock::TransferWithAuthorityIxArgs {
                amount,
                timelock_bump: src_vta.bump,
            }
        ).unwrap()
    ];

    let message = utils::message_with_sorted_keys(
        &ix,
        Some(&vm.get_authority()),
        &blockhash,
    );

    let message = message.serialize();
    utils::hash(&message)
}