use anchor_lang::prelude::*;

use crate::{program, types::Hash, CODE_VM_PREFIX};

pub fn create_memory_address(
    vm: Pubkey,
    name: &[u8; 32],
    bump: u8,
) -> Pubkey {
    Pubkey::create_program_address(
        &[
            CODE_VM_PREFIX.as_bytes(),
            b"vm_memory_account",
            name,
            vm.as_ref(),
            &[bump][..],
        ],
        &program::CodeVm::id(),
    ).unwrap()
}

pub fn create_unlock_address(
    virtual_account_owner: Pubkey,
    virtual_account: Pubkey,
    vm: Pubkey,
    bump: u8,
) -> Pubkey {
    Pubkey::create_program_address(
        &[
            CODE_VM_PREFIX.as_bytes(),
            b"vm_unlock_pda_account",
            virtual_account_owner.as_ref(),
            virtual_account.as_ref(),
            vm.as_ref(),
            &[bump][..],
        ],
        &program::CodeVm::id(),
    ).unwrap()
}

pub fn create_virtual_timelock_address(
    mint: Pubkey,
    authority: Pubkey,
    owner: Pubkey,
    lock_duration: u8,
    timelock_bump: u8,
) -> Pubkey {
    Pubkey::create_program_address(
        &[
            b"timelock_state",
            mint.as_ref(),
            authority.as_ref(),
            owner.as_ref(),
            lock_duration.to_le_bytes().as_ref(),
            &[timelock_bump][..],
        ],
        &timelock::ID,
    ).unwrap()
}

pub fn create_virtual_timelock_vault_address(
    timelock_address: Pubkey,
    vault_bump: u8,
) -> Pubkey {
    let version = 3;
    Pubkey::create_program_address(
        &[
            b"timelock_vault",
            timelock_address.as_ref(),
            &[version],
            &[vault_bump][..],
        ],
        &timelock::ID,
    ).unwrap()
}

pub fn create_withdraw_receipt_address(
    unlock_pda: Pubkey,
    nonce: Hash,
    vm: Pubkey,
    bump: u8,
) -> Pubkey {
    Pubkey::create_program_address(
        &[
            CODE_VM_PREFIX.as_bytes(),
            b"vm_withdraw_receipt_account",
            unlock_pda.as_ref(),

            // The VM can have multiple uncompressed or compressed records for
            // the same address at any given time. However, each one has a
            // unique nonce value. It should be set to the nonce value of the
            // record that the user wants to unlock.

            nonce.as_ref(), 
            vm.as_ref(),
            &[bump][..],
        ],
        &program::CodeVm::id(),
    ).unwrap()
}

pub fn find_withdraw_receipt_address(
    unlock_pda: Pubkey,
    nonce: Hash,
    vm: Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            CODE_VM_PREFIX.as_bytes(),
            b"vm_withdraw_receipt_account",
            unlock_pda.as_ref(),

            // The VM can have multiple uncompressed or compressed records for
            // the same address at any given time. However, each one has a
            // unique nonce value. It should be set to the nonce value of the
            // record that the user wants to unlock.

            nonce.as_ref(), 
            vm.as_ref(),
        ],
        &program::CodeVm::id(),
    )
}

pub fn find_relay_proof_address(
    relay: Pubkey,
    merkle_root: Hash,
    commitment: Hash,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            CODE_VM_PREFIX.as_bytes(),
            b"vm_proof_account",
            relay.as_ref(),
            merkle_root.as_ref(),
            commitment.as_ref(),
        ],
        &program::CodeVm::id(),
    )
}

pub fn find_relay_vault_address(
    proof: Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            CODE_VM_PREFIX.as_bytes(),
            b"vm_relay_vault",
            proof.as_ref(),
        ],
        &program::CodeVm::id(),
    )
}

pub fn find_relay_commitment_address(
    relay: Pubkey,
    merkle_root: Hash,
    transcript: Hash,
    destination: Pubkey,
    amount: u64,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            CODE_VM_PREFIX.as_bytes(),
            b"relay_commitment",
            relay.as_ref(),
            merkle_root.as_ref(),
            transcript.as_ref(),
            destination.as_ref(),
            amount.to_le_bytes().as_ref(),
        ],
        &program::CodeVm::id(),
    )
}
