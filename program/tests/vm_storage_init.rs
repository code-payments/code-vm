#![cfg(test)]
pub mod utils;
use utils::*;

use code_vm_api::prelude::*;

#[test]
fn run_storage_init_test() {
    let (mut svm, payer, _mint_owner, _mint_pk, vm_address) =
        setup_svm_with_payer_and_vm(21);

    let name = "test";

    let (vm_storage_address, vm_storage_bump) =
        create_storage_account(&mut svm, &payer, vm_address, name);

    let storage_account = svm.get_account(&vm_storage_address).unwrap();
    assert!(storage_account.data.len() == StorageAccount::get_size());

    let storage = get_storage_account(&svm, vm_storage_address);
    assert!(storage.vm == vm_address);
    assert!(storage.bump == vm_storage_bump);
    assert!(storage.name == create_name(name));

    let actual = storage.compressed_state;

    assert_eq!(actual.get_depth(), StorageAccount::MERKLE_TREE_DEPTH as u8);
    assert_ne!(actual.get_root(), Hash::default());
    assert_ne!(actual.get_empty_leaf(), Hash::default());

    let expected = MerkleTree::<{StorageAccount::MERKLE_TREE_DEPTH}>::new(&[
        MERKLE_TREE_SEED,
        create_name(name).as_ref(),
        vm_address.as_ref()
    ]);

    assert_eq!(actual.get_root(), expected.get_root());

    let vm = get_vm_account(&svm, vm_address);
    assert!(vm.slot == 2);
    assert!(vm.poh != Hash::default());
}