#![cfg(test)]
pub mod utils;
use solana_sdk::signer::Signer;
use utils::*;

use code_vm_api::prelude::*;

#[test]
fn run_transfer_for_swap() {
    let (mut svm, payer, mint_owner, mint_pk, vm_address) =
        setup_svm_with_payer_and_vm(21);

    let name = "test";
    let capacity = 100;
    let account_size = VirtualTimelockAccount::LEN+1;

    let (vm_memory, _) =
        create_and_resize_memory(&mut svm, &payer, vm_address, capacity, account_size, name);

    let amount = 1000;
    let account_index = 7;

    let (_, vta_key) = 
        create_timelock(&mut svm, &payer, vm_address, vm_memory, account_index);
    let swapper = vta_key.pubkey();
    let (swap_pda, bump) = find_timelock_swap_pda(&vm_address, &swapper);
    let swap_ata = create_ata(&mut svm, &payer, &mint_pk, &swap_pda);

    mint_to(&mut svm, &payer, &mint_pk, &mint_owner, &swap_ata, amount).unwrap();

    let destination = create_ata(&mut svm, &payer, &mint_pk, &swapper);

    assert!(tx_transfer_for_swap(
        &mut svm, 
        &payer,
        &vta_key,
        vm_address, 
        swap_pda, 
        swap_ata,
        destination,
        amount, 
        bump
    ).is_ok());

    let swap_ata_balance = get_ata_balance(&svm, &swap_ata);
    assert_eq!(0, swap_ata_balance);

    let destination_balance = get_ata_balance(&svm, &destination);
    assert_eq!(amount, destination_balance);
}

#[test]
fn run_cancel_swap() {
    let (mut svm, payer, mint_owner, mint_pk, vm_address) =
        setup_svm_with_payer_and_vm(21);

    let name = "test";
    let capacity = 100;
    let account_size = VirtualTimelockAccount::LEN+1;

    let (vm_memory, _) =
        create_and_resize_memory(&mut svm, &payer, vm_address, capacity, account_size, name);

    let amount = 1000;
    let account_index = 7;

    let (_, vta_key) = 
        create_timelock(&mut svm, &payer, vm_address, vm_memory, account_index);
    let swapper = vta_key.pubkey();
    let (swap_pda, bump) = find_timelock_swap_pda(&vm_address, &swapper);
    let swap_ata = create_ata(&mut svm, &payer, &mint_pk, &swap_pda);

    mint_to(&mut svm, &payer, &mint_pk, &mint_owner, &swap_ata, amount).unwrap();

    let vm = get_vm_account(&svm, vm_address);

    assert!(tx_cancel_swap(
        &mut svm, 
        &payer, 
        vm_address, 
        vm_memory, 
        swapper, 
        swap_pda, 
        swap_ata, 
        vm.omnibus.vault, 
        account_index, 
        amount, 
        bump
    ).is_ok());

    let vta = get_virtual_timelock(&svm, vm_memory, account_index);

    assert_eq!(vta.balance, amount);
}

#[test]
fn run_close_swap_account_if_empty() {
    let (mut svm, payer, mint_owner, mint_pk, vm_address) =
        setup_svm_with_payer_and_vm(21);

    let name = "test";
    let capacity = 100;
    let account_size = VirtualTimelockAccount::LEN+1;

    let (vm_memory, _) =
        create_and_resize_memory(&mut svm, &payer, vm_address, capacity, account_size, name);

    let (_, vta_key1) = 
        create_timelock(&mut svm, &payer, vm_address, vm_memory, 0);
    let swapper1 = vta_key1.pubkey();
    let (swap_pda1, bump1) = find_timelock_swap_pda(&vm_address, &swapper1);
    let swap_ata1 = create_ata(&mut svm, &payer, &mint_pk, &swap_pda1);

    let (_, vta_key2) = 
        create_timelock(&mut svm, &payer, vm_address, vm_memory, 1);
    let swapper2 = vta_key2.pubkey();
    let (swap_pda2, bump2) = find_timelock_swap_pda(&vm_address, &swapper2);
    let swap_ata2 = create_ata(&mut svm, &payer, &mint_pk, &swap_pda2);
    mint_to(&mut svm, &payer, &mint_pk, &mint_owner, &swap_ata2, 1000).unwrap();

    let destination = create_keypair().pubkey();

    assert!(tx_close_swap_account_if_empty(
        &mut svm,
        &payer,
        vm_address, 
        swapper1,
        swap_pda1,
        swap_ata1,
        destination,
        bump1
    ).is_ok());

    assert!(tx_close_swap_account_if_empty(
        &mut svm,
        &payer,
        vm_address,
        swapper2,
        swap_pda2,
        swap_ata2,
        destination,
        bump2
    ).is_ok());

    let destination_lamports = svm.get_account(&destination).unwrap().lamports;
    assert!(destination_lamports > 0);

    let swap_ata1_lamports = svm.get_account(&swap_ata1).unwrap().lamports;
    assert_eq!(0, swap_ata1_lamports);

    let swap_ata2_balance = get_ata_balance(&svm, &swap_ata2);
    assert_eq!(1000, swap_ata2_balance);
}