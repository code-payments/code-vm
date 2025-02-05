use steel::*;

use crate::utils;
use crate::types::Hash;
use crate::cvm::{
    CodeVmAccount,
    VirtualDurableNonce, 
    VirtualTimelockAccount
};

pub fn compact_airdrop_message(
    src_timelock_address: &Pubkey,
    dst_timelock_addresses: &[Pubkey],
    amount: u64,
    vdn: &VirtualDurableNonce,
) -> Hash {
    let mut msg = Vec::new();

    msg.push(b"airdrop" as &[u8]);
    msg.push(src_timelock_address.as_ref());
    msg.push(vdn.address.as_ref());
    msg.push(vdn.value.as_ref());

    // Store the little-endian bytes in a local variable so it won't go out of scope
    let amount_bytes = amount.to_le_bytes();
    msg.push(&amount_bytes);

    // Push each destination pubkey
    for dst_pubkey in dst_timelock_addresses {
        msg.push(dst_pubkey.as_ref());
    }

    utils::hashv(&msg)
}

pub fn create_airdrop_message(
    vm: &CodeVmAccount,
    src_vta: &VirtualTimelockAccount,
    destinations: &[Pubkey],
    amount: u64,
    vdn: &VirtualDurableNonce,
) -> Hash {

    let src_timelock_address = src_vta.get_timelock_address(
        &vm.get_mint(),
        &vm.get_authority(),
        vm.get_lock_duration(),
    );

    let src_token_address = src_vta.get_token_address(
        &src_timelock_address,
    );

    compact_airdrop_message(
        &src_token_address,
        destinations,
        amount,
        vdn,
    )
}