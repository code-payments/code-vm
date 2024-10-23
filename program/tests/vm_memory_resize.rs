#![cfg(test)]
pub mod utils;
use utils::*;

use code_vm_api::prelude::*;

#[test]
fn run_mem_resize_test() {
    let (mut svm, payer, _mint_owner, _mint_pk, vm_address) =
        setup_svm_with_payer_and_vm(21);

    let name = "test";
    let layout = MemoryLayout::Mixed;

    let (vm_mem_address, _) =
        create_and_resize_memory(&mut svm, &payer, vm_address, layout, name);

    let required_size = MemoryAccount::get_size_with_data(layout);
    let mem_account = svm.get_account(&vm_mem_address).unwrap();
    assert!(mem_account.data.len() == required_size);
}