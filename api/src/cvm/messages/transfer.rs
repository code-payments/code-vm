use steel::*;

use crate::utils;
use crate::types::Hash;
use crate::cvm::{
  CodeVmAccount,
  VirtualDurableNonce, 
  VirtualTimelockAccount
};

pub fn compact_transfer_message(
    src_timelock_address: &Pubkey,
    dst_timelock_address: &Pubkey,
    amount: u64,
    vdn: &VirtualDurableNonce,
) -> Hash {
    let message = &[
        b"transfer",
        src_timelock_address.as_ref(),
        dst_timelock_address.as_ref(),
        &amount.to_le_bytes(),
        vdn.address.as_ref(),
        vdn.value.as_ref(), // this value is auto-advanced upon use
    ];

    utils::hashv(message)
}

pub fn create_transfer_message(
    vm: &CodeVmAccount,
    src_vta: &VirtualTimelockAccount,
    dst_vta: &VirtualTimelockAccount,
    vdn: &VirtualDurableNonce,
    amount: u64,
) -> Hash {

    let src_timelock_address = src_vta.get_timelock_address(
        &vm.get_mint(),
        &vm.get_authority(),
        vm.get_lock_duration(),
    );
    let src_token_address = src_vta.get_token_address(
        &src_timelock_address,
    );

    let dst_timelock_address = dst_vta.get_timelock_address(
        &vm.get_mint(),
        &vm.get_authority(),
        vm.get_lock_duration(),
    );
    let dst_token_address = dst_vta.get_token_address(
        &dst_timelock_address,
    );

    compact_transfer_message(
        &src_token_address,
        &dst_token_address,
        amount,
        vdn,
    )
}

pub fn create_transfer_message_to_external(
    vm: &CodeVmAccount,
    src_vta: &VirtualTimelockAccount,
    dst_pubkey: &Pubkey,
    vdn: &VirtualDurableNonce,
    amount: u64,
) -> Hash {

    let src_timelock_address = src_vta.get_timelock_address(
        &vm.get_mint(),
        &vm.get_authority(),
        vm.get_lock_duration(),
    );
    let src_token_address = src_vta.get_token_address(
        &src_timelock_address,
    );

    compact_transfer_message(
        &src_token_address,
        dst_pubkey,
        amount,
        vdn,
    )
}


