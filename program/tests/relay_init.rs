#![cfg(test)]
pub mod utils;
use utils::*;

use code_vm_api::prelude::*;

#[test]
fn run_relay_init_test() {
    let (mut svm, payer, _mint_owner, mint_pk, vm_address) =
        setup_svm_with_payer_and_vm(21);

    let name = "test";

    let (relay_address, relay_bump) =
        create_relay_account(&mut svm, &payer, &mint_pk, vm_address, name);
    let (relay_vault_address, relay_vault_bump) =
        find_vm_relay_vault_pda(&relay_address);

    let relay_account = svm.get_account(&relay_address).unwrap();
    assert!(relay_account.data.len() == RelayAccount::get_size());

    let relay = get_relay_account(&svm, relay_address);
    assert_eq!(relay.vm, vm_address);
    assert_eq!(relay.bump, relay_bump);
    assert_eq!(relay.name, create_name(name));

    assert_eq!(relay.treasury.vault, relay_vault_address);
    assert_eq!(relay.treasury.vault_bump, relay_vault_bump);

    assert_eq!(relay.history.get_depth(), RELAY_STATE_DEPTH as u8);
    assert_eq!(relay.recent_roots.capacity(), RELAY_HISTORY_ITEMS);
    assert_eq!(relay.recent_roots.num_items, 1);
    assert_eq!(relay.recent_roots.first().unwrap(), relay.history.get_root().as_ref());

}