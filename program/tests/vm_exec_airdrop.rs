#![cfg(test)]

use steel::*;

pub mod utils;
use utils::*;

use solana_sdk::signature::Signer;
use code_vm_api::prelude::*;


#[test]
fn run_airdrop_10() {
    run_airdrop_test(10, 100);
}

#[test]
fn run_airdrop_25() {
    run_airdrop_test(25, 100);
}

#[test]
fn run_airdrop_50() {
    run_airdrop_test(50, 100);
}

#[test]
fn run_airdrop_chunked() {
    run_airdrop_test_chunked(200, 100);
}

#[test]
fn run_airdrop_0() {
    run_airdrop_test(0, 0);
    run_airdrop_test(0, 100);
    run_airdrop_test(10, 0);
}

#[test]
fn run_airdrop_include_self() {
    run_airdrop_with_self(0, 100);
    run_airdrop_with_self(1, 100);
    run_airdrop_with_self(10, 100);
}

#[test]
fn run_airdrop_only_to_self() {
    run_self_edgecase(0, 100);
    run_self_edgecase(10, 100);
}

/// Runs an airdrop test with the specified number of destination accounts.
/// Each destination receives 100 tokens from a single source timelock.
fn run_airdrop_test(count: usize, amount_each: u64) {
    let mut ctx = TestContext::new(21);

    let mem_a = ctx.create_memory(10, VirtualDurableNonce::LEN + 1, "mem_nonce_0");
    let mem_b = ctx.create_memory(count+2, VirtualTimelockAccount::LEN + 1, "mem_timelock_0");

    let vta_source = ctx.create_timelock_account(mem_b, 0); // move occurs because vta_source has type TimelockAccountContext, which does not implement the Copy

    let mut destinations = Vec::with_capacity(count);
    for i in 1..=count {
        let dst_vta = ctx.create_timelock_account(mem_b, i as u16);
        destinations.push(dst_vta);
    }

    let vdn_ctx = ctx.create_durable_nonce_account(mem_a, 0);

    let total_outflow = amount_each.checked_mul(count as u64).unwrap();
    let deposit_amount = total_outflow;
    ctx.deposit_tokens_to_timelock(mem_b, &vta_source, deposit_amount)
        .unwrap();

    let dst_pubkeys: Vec<_> = destinations
        .iter()
        .map(|dst_vta| dst_vta.account.owner)
        .collect();

    let hash = create_airdrop_message(
        &ctx.vm,
        &vta_source.account,
        &dst_pubkeys,
        amount_each,
        &vdn_ctx.account,
    );

    let sig = vta_source
        .key
        .sign_message(hash.as_ref())
        .as_ref()
        .try_into()
        .unwrap();

    let data = AirdropOp::from_struct(ParsedAirdropOp {
        signature: sig,
        amount: amount_each,
        count: count as u8
    }).to_bytes();

    let mut mem_indices = vec![vdn_ctx.index, vta_source.index];
    mem_indices.extend(destinations.iter().map(|d| d.index));

    let mut mem_banks = vec![0, 1]; // 0 for mem_a (nonce), 1 for mem_b (source/dest)
    mem_banks.extend(std::iter::repeat(1).take(count));

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

    let src_after = ctx.get_virtual_timelock(mem_b, vta_source.index);
    assert_eq!(src_after.balance, deposit_amount - total_outflow);

    for (i, dst) in destinations.iter().enumerate() {
        let dst_balance = ctx.get_virtual_timelock(mem_b, dst.index).balance;
        assert_eq!(
            dst_balance,
            amount_each,
            "Destination #{} did not receive 100 tokens",
            i
        );
    }
}

// Same as run_airdrop_test, but includes the source account in the destination list.
fn run_airdrop_with_self(count: usize, amount_each: u64) {
    let mut ctx = TestContext::new(21);

    let final_count = count + 1;
    let mem_a = ctx.create_memory(10, VirtualDurableNonce::LEN + 1, "mem_nonce_0");
    let mem_b = ctx.create_memory(count + 2, VirtualTimelockAccount::LEN + 1, "mem_timelock_0");

    let vta_source = ctx.create_timelock_account(mem_b, 0);

    let mut destinations = Vec::with_capacity(count);
    for i in 1..=count {
        let dst_vta = ctx.create_timelock_account(mem_b, i as u16);
        destinations.push(dst_vta);
    }

    let vdn_ctx = ctx.create_durable_nonce_account(mem_a, 0);
    let total_outflow = amount_each.checked_mul(final_count as u64).unwrap();

    let deposit_amount = total_outflow;
    ctx.deposit_tokens_to_timelock(mem_b, &vta_source, deposit_amount)
        .unwrap();

    let mut dst_pubkeys: Vec<_> = destinations
        .iter()
        .map(|d| d.account.owner)
        .collect();

    dst_pubkeys.push(vta_source.account.owner);

    let hash = create_airdrop_message(
        &ctx.vm,
        &vta_source.account,
        &dst_pubkeys,
        amount_each,
        &vdn_ctx.account,
    );

    let sig = vta_source
        .key
        .sign_message(hash.as_ref())
        .as_ref()
        .try_into()
        .unwrap();

    destinations.push(vta_source);

    let data = AirdropOp::from_struct(ParsedAirdropOp {
        signature: sig,
        amount: amount_each,
        count: final_count as u8,
    })
    .to_bytes();

    let mut mem_indices = vec![vdn_ctx.index, 0]; // 0 for source
    let mut mem_banks = vec![0, 1]; // 0 for mem_a, 1 for mem_b

    // For all destinations, push their memory index and bank=1
    for dst in &destinations {
        mem_indices.push(dst.index);
        mem_banks.push(1);
    }

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

    let src_after = ctx.get_virtual_timelock(mem_b, 0);
    assert_eq!(src_after.balance, deposit_amount - total_outflow + amount_each);

    for (i, dst) in destinations.iter().enumerate() {
        let dst_vta = ctx.get_virtual_timelock(mem_b, dst.index);
        let dst_balance = dst_vta.balance;

        if i == count {
            assert_eq!(dst_balance, deposit_amount - total_outflow + amount_each);
        } else {
            assert_eq!(
                dst_balance,
                amount_each,
                "Destination #{} did not receive {} tokens",
                i,
                amount_each
            );
        }
    }
}

/// Airdrops exclusively to the same source `count` times, each worth `amount_each`.
/// (This is an edge case, we're testing that the source account doesn't get more
/// or less tokens than what it started with)
fn run_self_edgecase(count: usize, amount_each: u64) {
    let mut ctx = TestContext::new(21);

    let mem_a = ctx.create_memory(1, VirtualDurableNonce::LEN + 1, "mem_nonce_0");
    let mem_b = ctx.create_memory(1, VirtualTimelockAccount::LEN + 1, "mem_timelock_0");

    let vta_source = ctx.create_timelock_account(mem_b, 0);

    let vdn_ctx = ctx.create_durable_nonce_account(mem_a, 0);

    let total_outflow = amount_each
        .checked_mul(count as u64)
        .expect("overflow computing total_outflow");

    let deposit_amount = total_outflow;
    ctx.deposit_tokens_to_timelock(mem_b, &vta_source, deposit_amount)
        .unwrap();

    let dst_pubkeys = vec![vta_source.account.owner; count];

    let hash = create_airdrop_message(
        &ctx.vm,
        &vta_source.account,
        &dst_pubkeys,
        amount_each,
        &vdn_ctx.account,
    );
    let signature = vta_source
        .key
        .sign_message(hash.as_ref())
        .as_ref()
        .try_into()
        .unwrap();

    let data = AirdropOp::from_struct(ParsedAirdropOp {
        signature,
        amount: amount_each,
        count: count as u8,
    })
    .to_bytes();

    let mut mem_indices = vec![vdn_ctx.index, vta_source.index];
    mem_indices.extend(std::iter::repeat(vta_source.index).take(count));

    let mut mem_banks = vec![0, 1];
    mem_banks.extend(std::iter::repeat(1).take(count));

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

    let src_after = ctx.get_virtual_timelock(mem_b, vta_source.index);
    assert_eq!(
        src_after.balance, 
        deposit_amount,
        "Source final balance mismatch after repeated self-airdrop"
    );
}

/// Runs an airdrop test with the specified number of destination accounts,
/// grouping them in chunks of up to 50. Each destination receives `amount_each`
/// tokens from a single source account.
fn run_airdrop_test_chunked(count: usize, amount_each: u64) {
    let mut ctx = TestContext::new(21);

    let mem_a = ctx.create_memory(10, VirtualDurableNonce::LEN + 1, "mem_nonce_0");
    let mem_b = ctx.create_memory(count+2, VirtualTimelockAccount::LEN + 1, "mem_timelock_0");

    let vta_source = ctx.create_timelock_account(mem_b, 0);

    let mut destinations = Vec::with_capacity(count);
    for i in 1..=count {
        let dst_vta = ctx.create_timelock_account(mem_b, i as u16);
        destinations.push(dst_vta);
    }

    let total_outflow = amount_each.checked_mul(count as u64).unwrap();
    ctx.deposit_tokens_to_timelock(mem_b, &vta_source, total_outflow)
        .unwrap();

    let chunk_size = 50;
    let mut instructions = Vec::new();

    let num_chunks = (count + chunk_size - 1) / chunk_size;

    for i in 0..num_chunks {
        let start = i * chunk_size;
        let end = std::cmp::min(start + chunk_size, count);
        let chunk = &destinations[start..end];

        // Create a fresh durable nonce account for this chunk (in prod, these
        // would be pre-created and reused)
        let vdn_ctx = ctx.create_durable_nonce_account(mem_a, i as u16);

        // Build the instruction for this chunk
        let ix = create_airdrop_ix(
            &mut ctx,
            mem_a,       // The memory account for the nonce
            mem_b,       // The memory account for source/dest
            &vdn_ctx,        // This chunk's nonce
            &vta_source,     // Source Timelock
            chunk,           // slice of up to 50
            amount_each,
        );
        instructions.push(ix);
    }

    ctx.ix_send(&instructions).unwrap();
    
    let src_after = ctx.get_virtual_timelock(mem_b, vta_source.index);
    assert_eq!(
        src_after.balance, 
        total_outflow - total_outflow,
        "Source did not properly deduct the outflow"
    );

    for (i, dst) in destinations.iter().enumerate() {
        let dst_balance = ctx.get_virtual_timelock(mem_b, dst.index).balance;
        assert_eq!(
            dst_balance,
            amount_each,
            "Destination #{} did not receive the expected tokens",
            i
        );
    }
}

/// Creates a single `Instruction` for a batch (chunk) of destinations.
/// This instructs the VM to execute the bulk airdrop for all `destinations`.
fn create_airdrop_ix(
    ctx: &mut TestContext,
    mem_a_key: Pubkey,                        // The memory account for the DurableNonce
    mem_b_key: Pubkey,                        // The memory account for Source & Dest accounts
    vdn_ctx: &DurableNonceContext,            // Contains .index and .account (nonce data)
    vta_source: &TimelockAccountContext,      // Source Timelock (has .index, .account, .key)
    destinations: &[TimelockAccountContext],  // A slice of up to ~50 destinations
    amount_each: u64,
) -> Instruction 
{
    let dst_pubkeys: Vec<_> = destinations
        .iter()
        .map(|dst| dst.account.owner) // or whatever pubkey you want to reference
        .collect();

    let hash = create_airdrop_message(
        &ctx.vm,
        &vta_source.account,
        &dst_pubkeys,
        amount_each,
        &vdn_ctx.account,
    );

    let sig = vta_source
        .key
        .sign_message(hash.as_ref())
        .as_ref()
        .try_into()
        .unwrap();

    let data = AirdropOp::from_struct(ParsedAirdropOp {
        signature: sig,
        amount: amount_each,
        count: destinations.len() as u8,
    })
    .to_bytes();

    let mut mem_indices = vec![vdn_ctx.index, vta_source.index];
    for d in destinations {
        mem_indices.push(d.index);
    }

    let mut mem_banks = vec![0u8, 1u8]; // 0 => mem_a, 1 => mem_b 
    mem_banks.extend(std::iter::repeat(1u8).take(destinations.len()));

    ctx.get_exec_ix(
        [Some(mem_a_key), Some(mem_b_key), None, None], // up to 4 memory accounts
        None, // vm_omnibus
        None, // relay
        None, // relay_vault
        None, // external_address
        None, // token_program
        data,
        mem_indices,
        mem_banks,
    )
}