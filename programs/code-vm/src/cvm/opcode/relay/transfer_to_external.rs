use anchor_lang::prelude::*;
use anchor_spl::token::{Transfer, transfer};
use borsh::{ BorshDeserialize, BorshSerialize };

use crate::log_event;
use crate::error::CodeVmError;
use crate::{ 
    utils,
    types::Hash, 
    instructions::CodeVmExec,
    cvm::{
        CodeVm, 
        MemoryBank,
        VirtualAccount,
        VirtualRelayAccount,
        ChangeLogData,
    },
    CODE_VM_PREFIX,
};

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[repr(C)]
struct OpcodeData {
    amount: u64,
    transcript: Hash,
    recent_root: Hash,
    commitment: Hash,
}

pub fn transfer_to_external (
    ctx: Context<CodeVmExec>,
    mem_indicies: Vec<u16>, 
    mem_banks: Vec<u8>, 
    data: Vec<u8>,
) -> Result<()> {

    // This action requires a virtual relay account.
    assert_eq!(mem_indicies.len(), 1);
    assert_eq!(mem_banks.len(), 1);

    let vra_index = mem_indicies[0];
    let vra_mem = MemoryBank::from(mem_banks[0]);

    // This action requires a real destination token account, the token program,
    // the relay, and its treasury.
    assert!(ctx.accounts.external_address.is_some());
    assert!(ctx.accounts.token_program.is_some());
    assert!(ctx.accounts.relay.is_some());
    assert!(ctx.accounts.relay_vault.is_some());

    let destination = ctx.accounts.external_address.as_ref().unwrap().to_account_info();
    let token_program: AccountInfo = ctx.accounts.token_program.as_ref().unwrap().to_account_info();
    let relay_vault = ctx.accounts.relay_vault.as_ref().unwrap().to_account_info();

    let mut vm = CodeVm::new(ctx.accounts.vm.to_owned());
    let relay = ctx.accounts.relay.as_mut().unwrap();

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

    require!(vm.has_memory_bank(vra_mem), CodeVmError::InvalidMemoryBank);
    require!(vm.is_empty_using(vra_mem, vra_index), CodeVmError::VirtualAccountAlreadyAllocated);

    // Deserialize the OpCode data
    let opcode_data = OpcodeData::try_from_slice(&data).unwrap();

    // Check that the provided relay recent_root is valid (it is in the
    // recent history of the relay)

    assert!(relay.recent_roots.contains(&opcode_data.recent_root.as_ref()));

    let relay_address = relay.key();
    let seeds = [
        CODE_VM_PREFIX.as_bytes(),
        b"vm_relay_vault",
        relay_address.as_ref(),
        &[relay.treasury.vault_bump],
    ];
    let signer = &[&seeds[..]];

    let cpi_accounts = Transfer {
        from: relay_vault.clone(),
        to: destination.clone(),
        authority: relay_vault.clone(),
    };

    let cpi_program = token_program.to_account_info().clone();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

    transfer(cpi_ctx, opcode_data.amount)?;

    let destination_address = destination.key();
    let (commitment, _) = utils::find_relay_commitment_address( // <- expensive
        relay.key(),
        opcode_data.recent_root,
        opcode_data.transcript,
        destination_address,
        opcode_data.amount,
    );

    // Whatever was passed in as the commitment should match what we calculated above
    assert_eq!(commitment.as_ref(), opcode_data.commitment.as_ref());

    // Add the commitment address to the merkle tree
    msg!("Adding val to merkle tree: {:?}", commitment.to_bytes().as_ref());
    relay.history.try_insert(commitment.to_bytes().into())?;

    // The vault_address below is where the mobile app will try to send tokens
    // to but our relay account will be used to redirect tokens to the treasury
    // instead (removing the need for the proof init, proof upload, proof
    // verify, token init, and token close instructions from the original
    // splitter program)

    // Find the virtual relay address
    let (proof_address, _) = utils::find_relay_proof_address( // <- expensive
        relay.key(),
        opcode_data.recent_root,
        opcode_data.commitment,
    );
    
    let (vault_address, _) = utils::find_relay_vault_address( // <- expensive
        proof_address,
    );

    let vra = VirtualRelayAccount {
        address: vault_address,
        commitment: opcode_data.commitment,
        recent_root: opcode_data.recent_root,
        destination: relay.treasury.vault.key(),
    };

    vm.try_write_account_using(
        vra_mem,
        vra_index, 
        VirtualAccount::Relay(vra),
    )?;

    log_event!(ctx, vm,
        Some(ChangeLogData::Transfer {
            src: relay_vault.to_account_info().key(),
            dst: destination_address,
            amount: opcode_data.amount,
        })
    );

    Ok(())
}
