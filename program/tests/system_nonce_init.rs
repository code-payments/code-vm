#![cfg(test)]
pub mod utils;
use utils::*;

use code_vm_api::prelude::*;
use solana_sdk::signer::Signer;

#[test]
fn run_system_nonce_init() {
    let (mut svm, payer, _mint_owner, _mint_pk, vm_address) =
        setup_svm_with_payer_and_vm(21);

    let name = "test";
    let layout = MemoryLayout::Nonce;

    let (vm_mem_address, _) =
        create_and_resize_memory(&mut svm, &payer, vm_address, layout, name);

    let vm = get_vm_account(&svm, vm_address);

    let virtual_account_owner = create_keypair().pubkey();
    let account_index = 0;
    assert!(tx_create_virtual_nonce(&mut svm, &payer, vm_address, vm_mem_address, virtual_account_owner, account_index).is_ok());

    let mem_account = svm.get_account(&vm_mem_address).unwrap();
    let paged_mem = MemoryAccount::into_indexed_memory(&mem_account.data);

    // Actual nonce values
    let data = paged_mem.read_item(account_index).unwrap();
    let va = VirtualAccount::unpack(&data).unwrap();
    let vdn = va.into_inner_nonce().unwrap();

    // Expected nonce values
    let seed = virtual_account_owner;
    let (nonce_address, _) = find_virtual_nonce_pda(
        &vm_address, &seed, &vm.get_current_poh()
    );

    assert_eq!(vdn.address, nonce_address);
    assert_eq!(vdn.value, vm.get_current_poh());

}