#![cfg(test)]
pub mod utils;
use steel::Clock;
use solana_sdk::signer::Signer;
use utils::*;

use code_vm_api::prelude::*;



#[test]
fn run_withdraw_from_deposit_pda() {
    let (mut svm, payer, mint_owner, mint_pk, vm_address) =
        setup_svm_with_payer_and_vm(21);

    let name = "test";
    let capacity = 100;
    let account_size = VirtualTimelockAccount::LEN+1;

    let (vm_memory, _) =
        create_and_resize_memory(&mut svm, &payer, vm_address, capacity, account_size, name);

    let amount = 1000;
    let account_index = 7;

    let (vta, vta_key) = 
        create_timelock(&mut svm, &payer, vm_address, vm_memory, account_index);

    let depositor = vta_key.pubkey();
    let (deposit_pda, deposit_pda_bump) = find_timelock_deposit_pda(&vm_address, &depositor);
    let deposit_ata = create_ata(&mut svm, &payer, &mint_pk, &deposit_pda);

    let dest_key = create_keypair();
    let destination = create_ata(&mut svm, &payer, &mint_pk, &dest_key.pubkey());

    mint_to(&mut svm, &payer, &mint_pk, &mint_owner, &deposit_ata, amount).unwrap();

    let vm = get_vm_account(&svm, vm_address);
    let timelock_address = vta.get_timelock_address(
        &vm.get_mint(), 
        &vm.get_authority(), 
        vm.get_lock_duration()
    );

    let unlock_address = vta.get_unlock_address(&timelock_address, &vm_address);

    assert!(tx_unlock_init(
        &mut svm, 
        &payer, 
        &vta_key,
        vm_address,
        unlock_address,
    ).is_ok());

    let unlock = get_unlock_state(&svm, unlock_address);
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = unlock.unlock_at + 1;
    svm.set_sysvar::<Clock>(&clock);

    assert!(tx_unlock_finalize(
        &mut svm, 
        &payer, 
        &vta_key,
        vm_address,
        unlock_address,
    ).is_ok());

    assert!(tx_withdraw_from_deposit(
        &mut svm, 
        &payer, 
        &vta_key, 
        vm_address, 
        deposit_pda, 
        deposit_ata, 
        unlock_address, 
        destination, 
        WithdrawIxData::FromDeposit { bump: deposit_pda_bump }
    ).is_ok());
}

#[test]
fn run_withdraw_from_memory() {
    let (mut svm, payer, _mint_owner, mint_pk, vm_address) =
        setup_svm_with_payer_and_vm(21);

    let name = "test";
    let capacity = 100;
    let account_size = VirtualTimelockAccount::LEN+1;

    let (vm_memory, _) =
        create_and_resize_memory(&mut svm, &payer, vm_address, capacity, account_size, name);

    let account_index = 7;

    let (vta, vta_key) = 
        create_timelock(&mut svm, &payer, vm_address, vm_memory, account_index);

    let dest_key = create_keypair();
    let destination = create_ata(&mut svm, &payer, &mint_pk, &dest_key.pubkey());

    let vm = get_vm_account(&svm, vm_address);
    let timelock_address = vta.get_timelock_address(
        &vm.get_mint(), 
        &vm.get_authority(), 
        vm.get_lock_duration()
    );

    let unlock_address = vta.get_unlock_address(&timelock_address, &vm_address);
    let receipt_address = vta.get_withdraw_receipt_address(&unlock_address, &vm_address);

    assert!(tx_unlock_init(
        &mut svm, 
        &payer, 
        &vta_key,
        vm_address,
        unlock_address,
    ).is_ok());

    let unlock = get_unlock_state(&svm, unlock_address);
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = unlock.unlock_at + 1;
    svm.set_sysvar::<Clock>(&clock);

    assert!(tx_unlock_finalize(
        &mut svm, 
        &payer, 
        &vta_key,
        vm_address,
        unlock_address,
    ).is_ok());

    assert!(tx_withdraw_from_memory(
        &mut svm, 
        &payer, 
        &vta_key,
        vm_address,
        vm.omnibus.vault,
        vm_memory,
        unlock_address,
        receipt_address,
        destination,
        WithdrawIxData::FromMemory { account_index }
    ).is_ok());
}


#[test]
fn run_withdraw_from_storage() {
    let (mut svm, payer, _mint_owner, mint_pk, vm_address) =
        setup_svm_with_payer_and_vm(21);

    let name = "test";
    let capacity = 100;
    let account_size = VirtualTimelockAccount::LEN+1;

    let (vm_memory, _) =
        create_and_resize_memory(&mut svm, &payer, vm_address, capacity, account_size, name);

    let (vm_storage, _) =
        create_storage_account(&mut svm, &payer, vm_address, name);

    let account_index = 7;

    let (vta, vta_key) = 
        create_timelock(&mut svm, &payer, vm_address, vm_memory, account_index);

    let dest_key = create_keypair();
    let destination = create_ata(&mut svm, &payer, &mint_pk, &dest_key.pubkey());

    let vm = get_vm_account(&svm, vm_address);
    let timelock_address = vta.get_timelock_address(
        &vm.get_mint(), 
        &vm.get_authority(), 
        vm.get_lock_duration()
    );

    let va = VirtualAccount::Timelock(vta);
    let va_hash = va.get_hash();
    let sig = Signature::new(payer.sign_message(va_hash.as_ref()).as_ref());
    let sig_hash = hashv(&[sig.as_ref(), va_hash.as_ref()]);

    assert!(tx_account_compress(
        &mut svm, 
        &payer,
        vm_address,
        vm_memory,
        vm_storage,
        account_index,
        sig
    ).is_ok());

    let compressed_mem = get_storage_account(&svm, vm_storage).compressed_state;
    let proof = compressed_mem.get_merkle_proof(&[sig_hash], 0);

    let unlock_address = vta.get_unlock_address(&timelock_address, &vm_address);
    let receipt_address = vta.get_withdraw_receipt_address(&unlock_address, &vm_address);

    assert!(tx_unlock_init(
        &mut svm, 
        &payer, 
        &vta_key,
        vm_address,
        unlock_address,
    ).is_ok());

    let unlock = get_unlock_state(&svm, unlock_address);
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = unlock.unlock_at + 1;
    svm.set_sysvar::<Clock>(&clock);

    assert!(tx_unlock_finalize(
        &mut svm, 
        &payer, 
        &vta_key,
        vm_address,
        unlock_address,
    ).is_ok());

    assert!(tx_withdraw_from_storage(
        &mut svm, 
        &payer, 
        &vta_key,
        vm_address,
        vm.omnibus.vault,
        vm_storage,
        unlock_address,
        receipt_address,
        destination,
        WithdrawIxData::FromStorage { 
            packed_va: va.pack(), 
            proof,
            signature: sig,
        } 
    ).is_ok());
}