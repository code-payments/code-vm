#![cfg(test)]
pub mod utils;
use utils::*;

use solana_sdk::signature::Signer;
use code_vm_api::prelude::*;

#[test]
fn run_transfer() {
    // Initialize the test context
    let mut ctx = TestContext::new(21);

    // Create memory accounts
    let mem_a = ctx.create_memory(100, VirtualDurableNonce::LEN + 1, "mem_nonce_0");
    let mem_b = ctx.create_memory(100, VirtualTimelockAccount::LEN + 1, "mem_timelock_0");

    // Create timelock accounts
    let vta_a_ctx = ctx.create_timelock_account(mem_b, 0);
    let vta_b_ctx = ctx.create_timelock_account(mem_b, 1);

    // Create durable nonce account
    let vdn_ctx = ctx.create_durable_nonce_account(mem_a, 0);

    // -- 1) Deposit tokens into `vta_a_ctx` so we have something to send
    let deposit_amount = 100;
    ctx.deposit_tokens_to_timelock(mem_b, &vta_a_ctx, deposit_amount)
        .unwrap();

    // -- 2) Set a non-zero transfer amount
    let amount = 42;

    // Create the transfer message and signature
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
    let data = TransferOp::from_struct(ParsedTransferOp { amount, signature }).to_bytes();

    // -- 3) Execute the transfer opcode
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

    // -- 4) Verify final balances
    let src_vta = ctx.get_virtual_timelock(mem_b, vta_a_ctx.index);
    let dst_vta = ctx.get_virtual_timelock(mem_b, vta_b_ctx.index);
    assert_eq!(src_vta.balance, deposit_amount - amount);
    assert_eq!(dst_vta.balance, amount);
}

#[test]
fn run_transfer_to_external() {
    // Initialize the test context
    let mut ctx = TestContext::new(21);

    // Create memory accounts
    let mem_a = ctx.create_memory(100, VirtualDurableNonce::LEN + 1, "mem_nonce_0");
    let mem_b = ctx.create_memory(100, VirtualTimelockAccount::LEN + 1, "mem_timelock_0");

    // Create timelock account
    let vta_a_ctx = ctx.create_timelock_account(mem_b, 0);

    // Create durable nonce account
    let vdn_ctx = ctx.create_durable_nonce_account(mem_a, 0);

    // Prepare the destination pubkey
    let dst_pubkey = ctx.vm.omnibus.vault; // e.g. the VM's omnibus vault

    // -- 1) Deposit tokens into `vta_a_ctx`
    let deposit_amount = 50;
    ctx.deposit_tokens_to_timelock(mem_b, &vta_a_ctx, deposit_amount)
        .unwrap();

    // -- 2) Set a non-zero transfer amount
    let amount = 10;

    // Create the transfer message and signature
    let hash = create_transfer_message_to_external(
        &ctx.vm,
        &vta_a_ctx.account,
        &dst_pubkey,
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
    let mem_indices = vec![vdn_ctx.index, vta_a_ctx.index];
    let mem_banks = vec![0, 1];
    let data = ExternalTransferOp::from_struct(
        ParsedExternalTransferOp { amount, signature }
    ).to_bytes();

    // -- 3) Execute the transfer opcode
    ctx.exec_opcode(
        [Some(mem_a), Some(mem_b), None, None],
        Some(ctx.vm.omnibus.vault), // vm_omnibus
        None,                       // relay
        None,                       // relay_vault
        Some(dst_pubkey),           // external_address
        Some(spl_token::id()),      // token_program
        data,
        mem_indices,
        mem_banks,
    )
    .unwrap();

    // -- 4) Verify final balance in the timelock
    let src_vta = ctx.get_virtual_timelock(mem_b, vta_a_ctx.index);
    assert_eq!(src_vta.balance, deposit_amount - amount);

    // Optionally, if you want to verify the omnibus vault gained tokens,
    // you'd look up the vault’s balance in the test context (implementation dependent).
}

#[test]
fn run_withdraw() {
    // Initialize the test context
    let mut ctx = TestContext::new(21);

    // Create memory accounts
    let mem_a = ctx.create_memory(100, VirtualDurableNonce::LEN + 1, "mem_nonce_0");
    let mem_b = ctx.create_memory(100, VirtualTimelockAccount::LEN + 1, "mem_timelock_0");

    // Create timelock accounts
    let vta_a_ctx = ctx.create_timelock_account(mem_b, 0);
    let vta_b_ctx = ctx.create_timelock_account(mem_b, 1);

    // Create durable nonce account
    let vdn_ctx = ctx.create_durable_nonce_account(mem_a, 0);

    // -- 1) Deposit tokens into vta_a
    let deposit_amount = 100;
    ctx.deposit_tokens_to_timelock(mem_b, &vta_a_ctx, deposit_amount)
        .unwrap();

    // Create the transfer message and signature
    let hash = create_withdraw_message(
        &ctx.vm,
        &vta_a_ctx.account,
        &vta_b_ctx.account,
        &vdn_ctx.account,
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
    let data = WithdrawOp { signature }.to_bytes();

    // -- 2) Execute the withdraw opcode
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

    // -- 3) Verify final balances
    let dst_vta = ctx.get_virtual_timelock(mem_b, vta_b_ctx.index);
    assert_eq!(dst_vta.balance, deposit_amount);

    // We expect the source account to be deleted after the withdraw
    let src_exists = ctx.has_virtual_account(mem_b, vta_a_ctx.index);
    assert_eq!(src_exists, false);
}

#[test]
fn run_withdraw_to_external() {
    // Initialize the test context
    let mut ctx = TestContext::new(21);

    // Create memory accounts
    let mem_a = ctx.create_memory(100, VirtualDurableNonce::LEN + 1, "mem_nonce_0");
    let mem_b = ctx.create_memory(100, VirtualTimelockAccount::LEN + 1, "mem_timelock_0");

    // Create timelock account
    let vta_a_ctx = ctx.create_timelock_account(mem_b, 0);

    // Create durable nonce account
    let vdn_ctx = ctx.create_durable_nonce_account(mem_a, 0);

    // Prepare the destination pubkey
    let dst_pubkey = ctx.vm.omnibus.vault; // e.g. the VM's omnibus vault

    // -- 1) Deposit tokens into `vta_a_ctx`
    let deposit_amount = 100;
    ctx.deposit_tokens_to_timelock(mem_b, &vta_a_ctx, deposit_amount)
        .unwrap();

    // Create the withdraw message and signature
    let hash = create_withdraw_message_to_external(
        &ctx.vm,
        &vta_a_ctx.account,
        &dst_pubkey,
        &vdn_ctx.account,
    );
    let signature = vta_a_ctx
        .key
        .sign_message(hash.as_ref())
        .as_ref()
        .try_into()
        .unwrap();

    // Prepare the opcode data
    let mem_indices = vec![vdn_ctx.index, vta_a_ctx.index];
    let mem_banks = vec![0, 1];
    let data = ExternalWithdrawOp { signature }.to_bytes();

    // -- 2) Execute the withdraw-to-external opcode
    ctx.exec_opcode(
        [Some(mem_a), Some(mem_b), None, None],
        Some(ctx.vm.omnibus.vault), // vm_omnibus
        None,                       // relay
        None,                       // relay_vault
        Some(dst_pubkey),           // external_address
        Some(spl_token::id()),      // token_program
        data,
        mem_indices,
        mem_banks,
    )
    .unwrap();

    // -- 3) Verify final balances

    let src_exists = ctx.has_virtual_account(mem_b, vta_a_ctx.index);
    assert_eq!(src_exists, false);

    let dst_balance = ctx.get_ata_balance(dst_pubkey);
    assert_eq!(dst_balance, deposit_amount);
}
