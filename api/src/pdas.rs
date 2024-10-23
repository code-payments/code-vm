use crate::consts::*;
use crate::external::*;
use crate::types::Hash;
use steel::*;

#[cfg(not(target_os = "solana"))]
pub fn find_vm_pda(mint: &Pubkey, authority: &Pubkey, lock_duration: u8) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[CODE_VM, mint.as_ref(), authority.as_ref(), &[lock_duration]],
        &crate::id(),
    )
}

#[cfg(not(target_os = "solana"))]
pub fn find_vm_omnibus_pda(vm: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[CODE_VM, VM_OMNIBUS, vm.as_ref()], &crate::id())
}

#[cfg(not(target_os = "solana"))]
pub fn find_vm_memory_pda(vm: &Pubkey, name: &[u8; MAX_NAME_LEN]) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[CODE_VM, VM_MEMORY_ACCOUNT, name.as_ref(), vm.as_ref()],
        &crate::id(),
    )
}

#[cfg(not(target_os = "solana"))]
pub fn find_vm_storage_pda(vm: &Pubkey, name: &[u8; MAX_NAME_LEN]) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[CODE_VM, VM_STORAGE_ACCOUNT, name.as_ref(), vm.as_ref()],
        &crate::id(),
    )
}

#[cfg(not(target_os = "solana"))]
pub fn find_vm_relay_pda(vm: &Pubkey, name: &[u8; MAX_NAME_LEN]) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[CODE_VM, VM_RELAY_ACCOUNT, name.as_ref(), vm.as_ref()],
        &crate::id(),
    )
}

#[cfg(not(target_os = "solana"))]
pub fn find_vm_relay_vault_pda(relay: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[CODE_VM, VM_RELAY_VAULT, relay.as_ref()], &crate::id())
}

#[cfg(not(target_os = "solana"))]
pub fn find_timelock_deposit_pda(vm: &Pubkey, depositor: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[CODE_VM, VM_DEPOSIT_PDA, depositor.as_ref(), vm.as_ref()],
        &crate::id(),
    )
}

pub fn create_timelock_deposit_pda(vm: &Pubkey, depositor: &Pubkey, bump: u8) -> Pubkey {
    Pubkey::create_program_address(
        &[
            CODE_VM,
            VM_DEPOSIT_PDA,
            depositor.as_ref(),
            vm.as_ref(),
            &[bump],
        ],
        &crate::id(),
    )
    .unwrap()
}

pub fn find_virtual_nonce_pda(vm: &Pubkey, seed: &Pubkey, poh: &Hash) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            CODE_VM,
            VM_DURABLE_NONCE,
            seed.as_ref(),
            poh.as_ref(),
            vm.as_ref(),
        ],
        &crate::id(),
    )
}

pub fn create_virtual_nonce_pda(vm: &Pubkey, seed: &Pubkey, poh: &Hash, bump: u8) -> Pubkey {
    Pubkey::create_program_address(
        &[
            CODE_VM,
            VM_DURABLE_NONCE,
            seed.as_ref(),
            poh.as_ref(),
            vm.as_ref(),
            &[bump],
        ],
        &crate::id(),
    )
    .unwrap()
}

pub fn find_virtual_timelock_address(
    mint: &Pubkey,
    authority: &Pubkey,
    owner: &Pubkey,
    lock_duration: u8,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            VM_TIMELOCK_STATE,
            mint.as_ref(),
            authority.as_ref(),
            owner.as_ref(),
            lock_duration.to_le_bytes().as_ref(),
        ],
        &timelock::ID,
    )
}

pub fn create_virtual_timelock_address(
    mint: &Pubkey,
    authority: &Pubkey,
    owner: &Pubkey,
    lock_duration: u8,
    timelock_bump: u8,
) -> Pubkey {
    Pubkey::create_program_address(
        &[
            VM_TIMELOCK_STATE,
            mint.as_ref(),
            authority.as_ref(),
            owner.as_ref(),
            lock_duration.to_le_bytes().as_ref(),
            &[timelock_bump][..],
        ],
        &timelock::ID,
    )
    .unwrap()
}

#[cfg(not(target_os = "solana"))]
pub fn find_virtual_timelock_vault_address(timelock_address: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[VM_TIMELOCK_VAULT, timelock_address.as_ref(), &[3]],
        &timelock::ID,
    )
}

pub fn create_virtual_timelock_vault_address(timelock_address: &Pubkey, vault_bump: u8) -> Pubkey {
    let version = 3;
    Pubkey::create_program_address(
        &[
            VM_TIMELOCK_VAULT,
            timelock_address.as_ref(),
            &[version],
            &[vault_bump][..],
        ],
        &timelock::ID,
    )
    .unwrap()
}

pub fn find_unlock_address(
    virtual_account_owner: &Pubkey,
    virtual_account: &Pubkey,
    vm: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            CODE_VM,
            VM_UNLOCK_ACCOUNT,
            virtual_account_owner.as_ref(),
            virtual_account.as_ref(),
            vm.as_ref(),
        ],
        &crate::id(),
    )
}

pub fn create_unlock_address(
    virtual_account_owner: &Pubkey,
    virtual_account: &Pubkey,
    vm: &Pubkey,
    bump: u8,
) -> Pubkey {
    Pubkey::create_program_address(
        &[
            CODE_VM,
            VM_UNLOCK_ACCOUNT,
            virtual_account_owner.as_ref(),
            virtual_account.as_ref(),
            vm.as_ref(),
            &[bump][..],
        ],
        &crate::id(),
    )
    .unwrap()
}

pub fn find_withdraw_receipt_address(
    unlock_pda: &Pubkey,
    nonce: &Hash,
    vm: &Pubkey,
) -> (Pubkey, u8) {
    // The VM can have multiple uncompressed or compressed records for
    // the same address at any given time. However, each one has a
    // unique nonce value. It should be set to the nonce value of the
    // record that the user wants to unlock.

    Pubkey::find_program_address(
        &[
            CODE_VM,
            VM_WITHDRAW_RECEIPT,
            unlock_pda.as_ref(),
            nonce.as_ref(),
            vm.as_ref(),
        ],
        &crate::id(),
    )
}

pub fn create_withdraw_receipt_address(
    unlock_pda: &Pubkey,
    nonce: &Hash,
    vm: &Pubkey,
    bump: u8,
) -> Pubkey {
    Pubkey::create_program_address(
        &[
            CODE_VM,
            VM_WITHDRAW_RECEIPT,
            unlock_pda.as_ref(),
            nonce.as_ref(),
            vm.as_ref(),
            &[bump][..],
        ],
        &crate::id(),
    )
    .unwrap()
}

pub fn find_relay_proof_address(
    relay: &Pubkey,
    merkle_root: &Hash,
    commitment: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            CODE_VM,
            VM_RELAY_PROOF,
            relay.as_ref(),
            merkle_root.as_ref(),
            commitment.as_ref(),
        ],
        &crate::id(),
    )
}

pub fn find_relay_commitment_address(
    relay: &Pubkey,
    merkle_root: &Hash,
    transcript: &Hash,
    destination: &Pubkey,
    amount: u64,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            CODE_VM,
            VM_RELAY_COMMITMENT,
            relay.as_ref(),
            merkle_root.as_ref(),
            transcript.as_ref(),
            destination.as_ref(),
            amount.to_le_bytes().as_ref(),
        ],
        &crate::id(),
    )
}

pub fn find_relay_destination(proof: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[CODE_VM, VM_RELAY_VAULT, proof.as_ref()], &splitter::ID)
}
