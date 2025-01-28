#![cfg(test)]
pub mod utils;
use utils::*;

use solana_sdk::signature::Signer;
use code_vm_api::prelude::*;

#[test]
fn run_relay_transfer() {
    // Initialize the test context
    let mut ctx = TestContext::new(21);

    // Setup a relay and treasury (with tokens)
    let relay_ctx = ctx.create_relay("relay_0", 10_00);

    // Create our virtual memory accounts
    let mem_a = ctx.create_memory(100, VirtualDurableNonce::LEN + 1, "mem_nonce_0");
    let mem_b = ctx.create_memory(100, VirtualTimelockAccount::LEN + 1, "mem_timelock_0");
    let mem_c = ctx.create_memory(100, VirtualRelayAccount::LEN + 1, "mem_relay_0");

    // Create some virtual accounts
    let vta_a_index = 7;
    let vta_b_index = 15;
    let vdn_index = 8;
    let vra_index = 3;

    let vta_a_ctx = ctx.create_timelock_account(mem_b, vta_a_index);
    let vta_b_ctx = ctx.create_timelock_account(mem_b, vta_b_index);
    let vdn_ctx = ctx.create_durable_nonce_account(mem_a, vdn_index);

    // Deposit 100 tokens into vta_b
    ctx.deposit_tokens_to_timelock(mem_b, &vta_b_ctx, 100)
        .unwrap();

    // We're going to do a relay transfer from vta_b to vta_a, and then a
    // conditional transfer from vta_b to the relay treasury. The net result is
    // that vta_a gets 42 tokens.

    // First, we need to calculate the commitment value
    let amount: u64 = 42;
    let recent_root = relay_ctx.relay.get_recent_root();
    let transcript = hashv(&[b"transfer", &amount.to_le_bytes()]);

    let timelock_address = vta_a_ctx.account.get_timelock_address(
        &ctx.vm.get_mint(),
        &ctx.vm.get_authority(),
        ctx.vm.get_lock_duration(),
    );
    let destination = vta_a_ctx.account.get_token_address(&timelock_address);

    let (commitment, _) = find_relay_commitment_address(
        &relay_ctx.relay_address,
        &recent_root,
        &transcript,
        &destination,
        amount,
    );

    // Next, we're going to calculate the target address and sign a transaction
    let (proof_address, _) = find_relay_proof_address(
        &relay_ctx.relay_address,
        &recent_root,
        &commitment,
    );
    let (target, _) = find_relay_destination(&proof_address);
    let conditional_payment = create_transfer_message_to_external(
        &ctx.vm,
        &vta_b_ctx.account,
        &target,
        &vdn_ctx.account,
        amount,
    );
    let conditional_sig = vta_b_ctx
        .key
        .sign_message(conditional_payment.as_ref())
        .as_ref()
        .try_into()
        .unwrap();

    // Run the relay to vta_a transfer
    let mem_indices = vec![vta_a_index, vra_index]; // dst, vra
    let mem_banks = vec![1, 2]; // mem_b, mem_c
    let data = RelayOp::from_struct( 
        ParsedRelayOp {
        amount,
        transcript,
        recent_root,
        commitment,
    }).to_bytes();

    ctx.exec_relay_op(
        &relay_ctx,
        [None, Some(mem_b), Some(mem_c), None],
        mem_indices,
        mem_banks,
        data,
    )
    .unwrap();

    let vta = ctx.get_virtual_timelock(mem_b, vta_a_index);
    assert_eq!(vta.balance, amount);

    let vta = ctx.get_virtual_timelock(mem_b, vta_b_index);
    assert_eq!(vta.balance, 100);

    let vra = get_virtual_relay(&ctx.svm, mem_c, vra_index);
    assert_eq!(vra.target, target);
    assert_eq!(vra.destination, relay_ctx.relay.treasury.vault);

    // Now, we're going to run the conditional transfer from vta_b to the relay
    let mem_indices = vec![vdn_index, vta_b_index, vra_index];
    let mem_banks = vec![0, 1, 2];
    let data = ConditionalTransferOp::from_struct(
        ParsedConditionalTransferOp {
        amount,
        signature: conditional_sig,
    }).to_bytes();

    ctx.exec_conditional_transfer(
        relay_ctx.relay.treasury.vault,
        [Some(mem_a), Some(mem_b), Some(mem_c), None],
        mem_indices,
        mem_banks,
        data,
    )
    .unwrap();

    // Let's confirm tokens left vta_b
    let vta = ctx.get_virtual_timelock(mem_b, vta_a_index);
    assert_eq!(vta.balance, 42);

    let vta = ctx.get_virtual_timelock(mem_b, vta_b_index);
    assert_eq!(vta.balance, 100 - 42);
}
