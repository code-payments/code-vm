#![cfg(test)]
pub mod utils;
use utils::*;

//use code_vm_api::prelude::*;

#[test]
fn run_relay_save_root_test() {
    let (mut svm, payer, _mint_owner, mint_pk, vm_address) =
        setup_svm_with_payer_and_vm(21);

    let name = "test";

    let (relay_address, _) =
        create_relay_account(&mut svm, &payer, &mint_pk, vm_address, name);

    assert!(tx_save_root(&mut svm, &payer, vm_address, relay_address).is_ok());

}