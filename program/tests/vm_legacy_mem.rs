#![cfg(test)]
pub mod utils;
use steel::Discriminator;
use utils::*;

use solana_sdk::signature::Signer;
use code_vm_api::prelude::*;

#[test]
fn run_transfer_on_legacy_memory() {
    // Initialize the test context
    let mut ctx = TestContext::new(21);

    // Create memory accounts
    let mem_a = ctx.create_memory(100, VirtualDurableNonce::LEN + 1, "mem_nonce_0");
    let mem_b = ctx.create_memory(NUM_ACCOUNTS, VirtualTimelockAccount::LEN + 1, "mem_timelock_0");

    // Change the memory account to legacy
    let mut info = ctx.svm.get_account(&mem_b).unwrap();
    let mem_data = info.data;
    let mut mem = MemoryAccount::unpack(&mem_data);

    mem.version = MemoryVersion::Legacy as u8;
    mem.packed_info = [
        0, 0, 0, 0, 0,  // _padding
        1               // layout (1 = Timelock)
    ];

    // Assemble the account back together
    let discriminator: &[u8; 8] = &[
        MemoryAccount::discriminator(),
        0, 0, 0, 0, 0, 0, 0,
    ];
    info.data = [
        discriminator, 
        mem.to_bytes(),
        &mem_data[MemoryAccount::get_size()..],
    ].concat();

    // Set the account directly
    ctx.svm
        .set_account(mem_b, info)
        .unwrap();

    // Check that the memory account is now legacy
    let info = ctx.svm.get_account(&mem_b).unwrap();
    let mem = MemoryAccount::unpack(&info.data);
    assert_eq!(mem.get_version(), MemoryVersion::Legacy);
    assert_eq!(mem.get_capacity(), NUM_ACCOUNTS);
    assert_eq!(mem.get_account_size(), VirtualTimelockAccount::LEN + 1);

    // Do a transfer to check that the memory account is still usable

    // Create timelock accounts
    let vta_a_ctx = ctx.create_timelock_account(mem_b, 0);
    let vta_b_ctx = ctx.create_timelock_account(mem_b, 1);

    // Create durable nonce account
    let vdn_ctx = ctx.create_durable_nonce_account(mem_a, 0);

    // Create the transfer message and signature
    let amount = 0; // Sending 0 tokens to keep the test simple
    let hash = create_transfer_message(
        &ctx.vm,
        &vta_a_ctx.account,
        &vta_b_ctx.account,
        &vdn_ctx.account,
        amount,
    );
    let signature = vta_a_ctx
        .key
        .sign_message(hash.as_ref())
        .as_ref()
        .try_into()
        .unwrap();

    // Prepare the opcode data
    let mem_indices = vec![vdn_ctx.index, vta_a_ctx.index, vta_b_ctx.index];
    let mem_banks = vec![0, 1, 1];
    let data = TransferOp::from_struct(
        ParsedTransferOp { amount, signature }
    ).to_bytes();

    // Execute the opcode
    ctx.exec_opcode(
        [Some(mem_a), Some(mem_b), None, None],
        None, // vm_omnibus
        None, // relay
        None, // relay_vault
        None, // external_address
        None, // token_program
        data,
        mem_indices,
        mem_banks,
    )
    .unwrap();
}
