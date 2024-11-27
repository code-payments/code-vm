#![cfg(test)]
pub mod utils;
use steel::Clock;
use utils::*;

use code_vm_api::prelude::*;

#[test]
fn run_unlock() {
    let (mut svm, payer, _mint_owner, _mint_pk, vm_address) =
        setup_svm_with_payer_and_vm(21);

    let name = "test";
    let capacity = 100;
    let account_size = VirtualTimelockAccount::LEN+1;

    let (vm_memory, _) =
        create_and_resize_memory(&mut svm, &payer, vm_address, capacity, account_size, name);

    let (vta, vta_key) = 
        create_timelock(&mut svm, &payer, vm_address, vm_memory, 0);

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

    assert_eq!(unlock.owner, vta.owner);
    assert_eq!(unlock.address, timelock_address);
    assert_eq!(unlock.vm, vm_address);
    assert_eq!(unlock.state, TimelockState::WaitingForTimeout as u8);
    assert!(unlock.bump > 0);
    assert!(unlock.unlock_at > 0);

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
}