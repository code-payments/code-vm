#![cfg(test)]
pub mod utils;
use solana_sdk::signer::Signer;
use utils::*;

use code_vm_api::prelude::*;

#[test]
fn run_deposit() {
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

    assert!(tx_deposit(
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