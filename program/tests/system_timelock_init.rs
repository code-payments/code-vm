#![cfg(test)]
pub mod utils;
use utils::*;

use code_vm_api::prelude::*;
use solana_sdk::signer::Signer;

#[test]
fn run_system_timelock_init() {
    let (mut svm, payer, _mint_owner, _mint_pk, vm_address) =
        setup_svm_with_payer_and_vm(21);

    let name = "test";
    let layout = MemoryLayout::Mixed;

    let (vm_mem_address, _) =
        create_and_resize_memory(&mut svm, &payer, vm_address, layout, name);

    let vm = get_vm_account(&svm, vm_address);

    let virtual_account_owner = create_keypair().pubkey();
    let account_index = 0;
    let nonce = vm.get_current_poh();

    let (timelock_address, virtual_timelock_bump) = find_virtual_timelock_address(
        &vm.get_mint(), 
        &vm.get_authority(), 
        &virtual_account_owner, 
        vm.get_lock_duration()
    );

    let (_, virtual_vault_bump) = find_virtual_timelock_vault_address(
        &timelock_address
    );

    let (unlock_address, unlock_pda_bump)  = find_unlock_address(
        &virtual_account_owner, 
        &timelock_address, 
        &vm_address, 
    );

    let (_, withdraw_bump) = find_withdraw_receipt_address(
        &unlock_address, 
        &nonce, 
        &vm_address
    );

    assert!(tx_create_virtual_timelock(
        &mut svm, 
        &payer, 
        vm_address, 
        vm_mem_address, 
        virtual_account_owner, 
        account_index,
        virtual_timelock_bump,
        virtual_vault_bump,
        unlock_pda_bump,
    ).is_ok());

    let mem_account = svm.get_account(&vm_mem_address).unwrap();
    let paged_mem = MemoryAccount::into_indexed_memory(&mem_account.data);

    // Expected values
    let vta = VirtualTimelockAccount {
        owner: virtual_account_owner,
        nonce,
        bump: virtual_timelock_bump,
        token_bump: virtual_vault_bump,
        unlock_bump: unlock_pda_bump,
        withdraw_bump,
        balance: 0,
    };
    let expected = VirtualAccount::Timelock(vta);

    // Actual nonce values
    let data = paged_mem.read_item(account_index).unwrap();
    let actual = VirtualAccount::unpack(&data).unwrap();

    assert_eq!(expected, actual);

}

