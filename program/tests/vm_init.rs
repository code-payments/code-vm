#![cfg(test)]
pub mod utils;
use utils::*;

use code_vm_api::prelude::*;
use solana_sdk::signer::Signer;

#[test]
fn run_vm_init_test() {
    let (svm, payer, _mint_owner, mint_pk, _vm_address) =
        setup_svm_with_payer_and_vm(21);

    let (vm_address, vm_bump) = find_vm_pda(&mint_pk, &payer.pubkey(), 21);
    let (omnibus_address, omnibus_bump) = find_vm_omnibus_pda(&vm_address);

    let vm_account = svm.get_account(&vm_address).unwrap();
    assert!(vm_account.data.len() == CodeVmAccount::get_size());

    let vm = get_vm_account(&svm, vm_address);
    assert!(vm.lock_duration == 21);
    assert!(vm.mint == mint_pk);
    assert!(vm.authority == payer.pubkey());
    assert!(vm.bump == vm_bump);
    assert!(vm.omnibus.vault == omnibus_address);
    assert!(vm.omnibus.vault_bump == omnibus_bump);
    assert!(vm.poh != Hash::default());
    assert!(vm.slot == 1);
}