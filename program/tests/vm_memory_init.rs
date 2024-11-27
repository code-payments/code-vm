#![cfg(test)]
pub mod utils;
use utils::*;

use code_vm_api::prelude::*;

#[test]
fn run_mem_init_test() {
    let (mut svm, payer, _mint_owner, _mint_pk, vm_address) =
        setup_svm_with_payer_and_vm(21);

    let name = create_name("test");
    let capacity = 100;
    let account_size = VirtualDurableNonce::LEN+1;

    let (vm_mem_address, vm_mem_bump) = find_vm_memory_pda(&vm_address, &name);

    assert!(tx_create_memory(&mut svm, &payer, vm_address, capacity, account_size, "test").is_ok());

    let mem_account = svm.get_account(&vm_mem_address).unwrap();
    assert!(mem_account.data.len() == MemoryAccount::get_size());

    let memory = get_memory_account(&svm, vm_mem_address);
    assert!(memory.vm == vm_address);
    assert!(memory.bump == vm_mem_bump);
    assert!(memory.version == 1);
    assert!(memory.num_accounts == capacity as u32);
    assert!(memory.account_size == account_size as u16);
    assert!(memory.name == name);

    let vm = get_vm_account(&svm, vm_address);
    assert!(vm.slot == 2);
    assert!(vm.poh != Hash::default());
}