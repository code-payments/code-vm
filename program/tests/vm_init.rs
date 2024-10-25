#![cfg(test)]
pub mod utils;
use utils::*;

use code_vm_api::prelude::*;
use solana_sdk::signer::Signer;

#[test]
fn run_vm_init_test() {
    // Initialize the test context with a lock duration of 21 days
    let ctx = TestContext::new(21);

    // Find the expected VM and omnibus addresses and their bumps
    let (expected_vm_address, expected_vm_bump) =
        find_vm_pda(&ctx.mint_pk, &ctx.payer.pubkey(), 21);
    let (expected_omnibus_address, expected_omnibus_bump) =
        find_vm_omnibus_pda(&expected_vm_address);

    // Retrieve the VM account from the SVM
    let vm_account = ctx.svm.get_account(&ctx.vm_address).unwrap();
    assert_eq!(vm_account.data.len(), CodeVmAccount::get_size());

    // Use the VM account from the context
    let vm = &ctx.vm;

    // Perform assertions to verify the VM initialization
    assert_eq!(vm.lock_duration, 21);
    assert_eq!(vm.mint, ctx.mint_pk);
    assert_eq!(vm.authority, ctx.payer.pubkey());
    assert_eq!(vm.bump, expected_vm_bump);
    assert_eq!(vm.omnibus.vault, expected_omnibus_address);
    assert_eq!(vm.omnibus.vault_bump, expected_omnibus_bump);
    assert_ne!(vm.poh, Hash::default());
    assert_eq!(vm.slot, 1);
}