#![cfg(test)]
pub mod utils;
use utils::*;

use code_vm_api::{prelude::*, utils::hashv};
use solana_sdk::signer::Signer;

#[test]
fn run_system_account_decompress() {
    let (mut svm, payer, _mint_owner, _mint_pk, vm_address) =
        setup_svm_with_payer_and_vm(21);

    let name = "test";
    let capacity = 100;
    let account_size = VirtualDurableNonce::LEN+1;

    let (vm_mem_address, _) =
        create_and_resize_memory(&mut svm, &payer, vm_address, capacity, account_size, name);

    let (vm_storage_address, _) =
        create_storage_account(&mut svm, &payer, vm_address, name);

    let virtual_account_owner = create_keypair().pubkey();
    let account_index = 0;
    assert!(tx_create_virtual_nonce(&mut svm, &payer, vm_address, vm_mem_address, virtual_account_owner, account_index).is_ok());

    let va = get_virtual_account(&svm, vm_mem_address, account_index);
    let va_hash = va.get_hash();

    let sig = Signature::new(payer.sign_message(va_hash.as_ref()).as_ref());
    let sig_hash = hashv(&[sig.as_ref(), va_hash.as_ref()]);
    
    assert!(tx_account_compress(
        &mut svm, 
        &payer,
        vm_address,
        vm_mem_address,
        vm_storage_address,
        account_index,
        sig
    ).is_ok());

    let data = get_virtual_account_data(&svm, vm_mem_address, account_index);
    assert!(data.is_none());

    let compressed_mem = get_storage_account(&svm, vm_storage_address).compressed_state;
    let mut expected = MerkleTree::<{StorageAccount::MERKLE_TREE_DEPTH}>::new(&[
        MERKLE_TREE_SEED,
        create_name(name).as_ref(),
        vm_address.as_ref()
    ]);
    assert!(expected.try_insert(sig_hash).is_ok());
    assert_eq!(expected.get_root(), compressed_mem.get_root());

    let packed_va = va.pack();
    let proof = expected.get_merkle_proof(&[sig_hash], 0);
    let account_index = 42;

    assert!(tx_account_decompress(
        &mut svm, 
        &payer, 
        vm_address,
        vm_mem_address,
        vm_storage_address,
        None,
        None,
        account_index,
        packed_va,
        proof.clone(),
        sig
    ).is_ok());

    let compressed_mem = get_storage_account(&svm, vm_storage_address).compressed_state;

    assert!(expected.try_remove(&proof, sig_hash).is_ok());
    assert_eq!(expected.get_root(), compressed_mem.get_root());

    let old_index = get_virtual_account_data(&svm, vm_mem_address, 0);
    let new_index = get_virtual_account_data(&svm, vm_mem_address, account_index);

    assert!(old_index.is_none());
    assert!(new_index.is_some());

    let va = get_virtual_account(&svm, vm_mem_address, account_index);
    assert!(va.is_nonce());
}