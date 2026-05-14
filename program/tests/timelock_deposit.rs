#![cfg(test)]
pub mod utils;
use solana_sdk::signer::Signer;
use utils::*;

use code_vm_api::prelude::*;

#[test]
fn run_deposit_from_pda() {
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
    let depositor = vta_key.pubkey();
    let (deposit_pda, bump) = find_timelock_deposit_pda(&vm_address, &depositor);
    let deposit_ata = create_ata(&mut svm, &payer, &mint_pk, &deposit_pda);

    mint_to(&mut svm, &payer, &mint_pk, &mint_owner, &deposit_ata, amount).unwrap();

    let vm = get_vm_account(&svm, vm_address);

    assert!(tx_deposit_from_pda(
        &mut svm, 
        &payer, 
        vm_address, 
        vm_memory, 
        depositor, 
        deposit_pda, 
        deposit_ata, 
        vm.omnibus.vault, 
        account_index, 
        amount, 
        bump
    ).is_ok());

    let vta = get_virtual_timelock(&svm, vm_memory, account_index);

    assert_eq!(vta.balance, amount);
}

#[test]
fn run_deposit_with_authority() {
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
    let destination = vta_key.pubkey();

    let source_key = create_keypair();
    let source = create_ata(&mut svm, &payer, &mint_pk, &source_key.pubkey());

    mint_to(&mut svm, &payer, &mint_pk, &mint_owner, &source, amount).unwrap();

    let vm = get_vm_account(&svm, vm_address);

    assert!(tx_deposit_with_authority(
        &mut svm,
        &payer,
        &source_key,
        vm_address,
        vm_memory,
        source,
        destination,
        vm.omnibus.vault,
        account_index,
        amount,
    ).is_ok());

    let vta = get_virtual_timelock(&svm, vm_memory, account_index);

    assert_eq!(vta.balance, amount);
}

#[test]
fn run_close_deposit_account_if_empty() {
    let (mut svm, payer, mint_owner, mint_pk, vm_address) =
        setup_svm_with_payer_and_vm(21);

    let name = "test";
    let capacity = 100;
    let account_size = VirtualTimelockAccount::LEN+1;

    let (vm_memory, _) =
        create_and_resize_memory(&mut svm, &payer, vm_address, capacity, account_size, name);

    let (_, vta_key1) =
        create_timelock(&mut svm, &payer, vm_address, vm_memory, 0);
    let depositor1 = vta_key1.pubkey();
    let (deposit_pda1, bump1) = find_timelock_deposit_pda(&vm_address, &depositor1);
    let deposit_ata1 = create_ata(&mut svm, &payer, &mint_pk, &deposit_pda1);

    let (_, vta_key2) =
        create_timelock(&mut svm, &payer, vm_address, vm_memory, 1);
    let depositor2 = vta_key2.pubkey();
    let (deposit_pda2, bump2) = find_timelock_deposit_pda(&vm_address, &depositor2);
    let deposit_ata2 = create_ata(&mut svm, &payer, &mint_pk, &deposit_pda2);
    mint_to(&mut svm, &payer, &mint_pk, &mint_owner, &deposit_ata2, 1000).unwrap();

    let destination = create_keypair().pubkey();

    assert!(tx_close_deposit_account_if_empty(
        &mut svm,
        &payer,
        vm_address,
        depositor1,
        deposit_pda1,
        deposit_ata1,
        destination,
        bump1
    ).is_ok());

    assert!(tx_close_deposit_account_if_empty(
        &mut svm,
        &payer,
        vm_address,
        depositor2,
        deposit_pda2,
        deposit_ata2,
        destination,
        bump2
    ).is_ok());

    let destination_lamports = svm.get_account(&destination).unwrap().lamports;
    assert!(destination_lamports > 0);

    let deposit_ata1_lamports = svm.get_account(&deposit_ata1).unwrap().lamports;
    assert_eq!(0, deposit_ata1_lamports);

    let deposit_ata2_balance = get_ata_balance(&svm, &deposit_ata2);
    assert_eq!(1000, deposit_ata2_balance);
}
