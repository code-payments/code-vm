#![cfg(test)]
use code_vm_api::prelude::*;
use litesvm::{types::TransactionResult, LiteSVM};
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};
use solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE;

use super::svm::*;

pub fn setup_svm_with_payer_and_vm(
    lock_duration: u8,
) -> (LiteSVM, Keypair, Keypair, Pubkey, Pubkey) {
    let mut svm = setup_svm();

    let payer = create_payer(&mut svm);
    let mint_owner = create_keypair();
    let mint_pk = create_mint(&mut svm, &payer, &mint_owner.pubkey());
    assert!(tx_create_vm(&mut svm, &payer, &mint_pk, lock_duration).is_ok());

    let (vm_address, _) = find_vm_pda(&mint_pk, &payer.pubkey(), lock_duration);

    (svm, payer, mint_owner, mint_pk, vm_address)
}

pub fn get_vm_account(svm: &LiteSVM, vm_address: Pubkey) -> CodeVmAccount {
    let account = svm.get_account(&vm_address).unwrap();
    CodeVmAccount::unpack(&account.data)
}

pub fn get_memory_account(svm: &LiteSVM, memory_address: Pubkey) -> MemoryAccount {
    let account = svm.get_account(&memory_address).unwrap();
    MemoryAccount::unpack(&account.data)
}

pub fn get_storage_account(svm: &LiteSVM, storage_address: Pubkey) -> StorageAccount {
    let account = svm.get_account(&storage_address).unwrap();
    StorageAccount::unpack(&account.data)
}

pub fn get_relay_account(svm: &LiteSVM, relay_address: Pubkey) -> RelayAccount {
    let account = svm.get_account(&relay_address).unwrap();
    RelayAccount::unpack(&account.data)
}

pub fn get_unlock_state(svm: &LiteSVM, unlock_address: Pubkey) -> UnlockStateAccount {
    let account = svm.get_account(&unlock_address).unwrap();
    UnlockStateAccount::unpack(&account.data)
}

pub fn get_virtual_account_data(svm: &LiteSVM, vm_memory: Pubkey, account_index: u16) -> Option<Vec<u8>> {
    let info = svm.get_account(&vm_memory).unwrap();
    let mem_account = MemoryAccount::unpack(&info.data);
    let capacity = mem_account.num_accounts as usize;
    let max_item_size = mem_account.account_size as usize;

    let offset = MemoryAccount::get_size();
    let data = info.data.split_at(offset).1;
    let mem = SliceAllocator::try_from_slice(data, capacity, max_item_size).unwrap();
    mem.read_item(account_index)
}

pub fn get_virtual_account(svm: &LiteSVM, vm_memory: Pubkey, account_index: u16) -> VirtualAccount {
    let data = get_virtual_account_data(svm, vm_memory, account_index).unwrap();
    VirtualAccount::unpack(&data).unwrap()
}

pub fn get_virtual_nonce(svm: &LiteSVM, vm_memory: Pubkey, account_index: u16) -> VirtualDurableNonce {
    let va = get_virtual_account(svm, vm_memory, account_index);
    va.into_inner_nonce().unwrap()
}

pub fn get_virtual_timelock(svm: &LiteSVM, vm_memory: Pubkey, account_index: u16) -> VirtualTimelockAccount {
    let va = get_virtual_account(svm, vm_memory, account_index);
    va.into_inner_timelock().unwrap()
}

pub fn get_virtual_relay(svm: &LiteSVM, vm_memory: Pubkey, account_index: u16) -> VirtualRelayAccount {
    let va = get_virtual_account(svm, vm_memory, account_index);
    va.into_inner_relay().unwrap()
}

pub fn create_durable_nonce(
    svm: &mut LiteSVM,
    payer: &Keypair,
    vm_address: Pubkey,
    vm_memory: Pubkey,
    account_index: u16,
) -> (VirtualDurableNonce, Keypair) {
    let signer = create_keypair();
    let owner = signer.pubkey();

    assert!(tx_create_virtual_nonce(
        svm, 
        &payer, 
        vm_address, 
        vm_memory, 
        owner, 
        account_index,
    ).is_ok());

    let vdn = get_virtual_nonce(svm, vm_memory, account_index);

    (vdn, signer)
}

pub fn create_timelock(
    svm: &mut LiteSVM,
    payer: &Keypair,
    vm_address: Pubkey,
    vm_memory: Pubkey,
    account_index: u16,
) -> (VirtualTimelockAccount, Keypair) {
    let vm = get_vm_account(&svm, vm_address);
    let signer = create_keypair();
    let owner = signer.pubkey();

    let (timelock_address, virtual_timelock_bump) = find_virtual_timelock_address(
        &vm.get_mint(), 
        &vm.get_authority(), 
        &owner, 
        vm.get_lock_duration()
    );

    let (_, virtual_vault_bump) = find_virtual_timelock_vault_address(
        &timelock_address
    );

    let (_, unlock_pda_bump) = find_unlock_address(
        &owner, 
        &timelock_address, 
        &vm_address, 
    );

    // Create the virtual timelock account
    assert!(tx_create_virtual_timelock(
        svm, 
        &payer, 
        vm_address, 
        vm_memory, 
        owner, 
        account_index,
        virtual_timelock_bump,
        virtual_vault_bump,
        unlock_pda_bump,
    ).is_ok());

    // Grab the virtual account data from the memory account
    let vta = get_virtual_timelock(svm, vm_memory, account_index);

    (vta, signer)
}

pub fn create_and_resize_memory(
    svm: &mut LiteSVM,
    payer: &Keypair,
    vm_address: Pubkey,
    capacity: usize,
    account_size: usize,
    name: &str,
) -> (Pubkey, u8) {
    assert!(tx_create_memory(svm, payer, vm_address, capacity, account_size, name).is_ok());

    let (mem_address, mem_bump) = find_vm_memory_pda(&vm_address, &create_name(name));

    // Increase account size until it reaches the required size for the layout
    let required_size = MemoryAccount::get_size_with_data(capacity, account_size);
    loop {
        let mem_account = svm.get_account(&mem_address).unwrap();
        let current = mem_account.data.len();

        if current == required_size {
            break;
        }

        // Keep in mind we can only increase the account size by MAX_PERMITTED_DATA_INCREASE.
        // Let's figure out how much more we need to increase the account size
        let remaining = required_size - current;
        let increase = std::cmp::min(remaining, MAX_PERMITTED_DATA_INCREASE) + current;

        assert!(tx_resize_memory(
            svm,
            payer,
            vm_address,
            mem_address,
            increase as u32
        )
        .is_ok());
    }

    (mem_address, mem_bump)
}

pub fn create_storage_account(
    svm: &mut LiteSVM,
    payer: &Keypair,
    vm_address: Pubkey,
    name: &str,
) -> (Pubkey, u8) {
    assert!(tx_create_storage(svm, payer, vm_address, name).is_ok());

    let (storage_address, storage_bump) =
        find_vm_storage_pda(&vm_address, &create_name(name));

    (storage_address, storage_bump)
}

pub fn create_relay_account(
    svm: &mut LiteSVM,
    payer: &Keypair,
    mint: &Pubkey,
    vm_address: Pubkey,
    name: &str,
) -> (Pubkey, u8) {
    assert!(tx_create_relay(svm, payer, mint, vm_address, name).is_ok());

    let (relay_address, relay_bump) =
        find_vm_relay_pda(&vm_address, &create_name(name));

    (relay_address, relay_bump)
}

pub fn tx_create_vm(
    svm: &mut LiteSVM,
    payer: &Keypair,
    mint: &Pubkey,
    lock_duration: u8,
) -> TransactionResult {
    let payer_pk = payer.pubkey();
    let blockhash = svm.latest_blockhash();
    let ix = vm_init(payer_pk, *mint, lock_duration);
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer_pk), &[payer], blockhash);

    send_tx(svm, tx)
}

pub fn tx_create_storage(
    svm: &mut LiteSVM,
    payer: &Keypair,
    vm_address: Pubkey,
    name: &str,
) -> TransactionResult {
    let payer_pk = payer.pubkey();
    let blockhash = svm.latest_blockhash();
    let ix = vm_storage_init(payer_pk, vm_address, name);
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer_pk), &[payer], blockhash);

    send_tx(svm, tx)
}

pub fn tx_create_memory(
    svm: &mut LiteSVM,
    payer: &Keypair,
    vm_address: Pubkey,
    capacity: usize,
    account_size: usize,
    name: &str,
) -> TransactionResult {
    let payer_pk = payer.pubkey();
    let blockhash = svm.latest_blockhash();
    let ix = vm_memory_init(payer_pk, vm_address, capacity, account_size, name);
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer_pk), &[payer], blockhash);

    send_tx(svm, tx)
}

pub fn tx_resize_memory(
    svm: &mut LiteSVM,
    payer: &Keypair,
    vm_address: Pubkey,
    vm_memory: Pubkey,
    account_size: u32,
) -> TransactionResult {
    let payer_pk = payer.pubkey();
    let blockhash = svm.latest_blockhash();
    let ix = vm_memory_resize(payer_pk, vm_address, vm_memory, account_size);
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer_pk), &[payer], blockhash);

    send_tx(svm, tx)
}

pub fn tx_create_virtual_nonce(
    svm: &mut LiteSVM,
    payer: &Keypair,
    vm_address: Pubkey,
    vm_memory: Pubkey,
    virtual_account_owner: Pubkey,
    account_index: u16,
) -> TransactionResult {
    let payer_pk = payer.pubkey();
    let blockhash = svm.latest_blockhash();
    let ix = system_nonce_init(payer_pk, vm_address, vm_memory, virtual_account_owner, account_index);
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer_pk), &[payer], blockhash);

    send_tx(svm, tx)
}

pub fn tx_create_virtual_timelock(
    svm: &mut LiteSVM,
    payer: &Keypair,
    vm_address: Pubkey,
    vm_memory: Pubkey,
    virtual_account_owner: Pubkey,
    account_index: u16,
    virtual_timelock_bump: u8,
    virtual_vault_bump: u8,
    unlock_pda_bump: u8,
) -> TransactionResult {
    let payer_pk = payer.pubkey();
    let blockhash = svm.latest_blockhash();
    let ix = system_timelock_init(
        payer_pk, 
        vm_address, 
        vm_memory, 
        virtual_account_owner, 
        account_index, 
        virtual_timelock_bump, 
        virtual_vault_bump, 
        unlock_pda_bump
    );
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer_pk), &[payer], blockhash);

    send_tx(svm, tx)
}

pub fn tx_account_compress(
    svm: &mut LiteSVM,
    payer: &Keypair,
    vm_address: Pubkey,
    vm_memory: Pubkey,
    vm_storage: Pubkey,
    account_index: u16,
    signature: Signature,
) -> TransactionResult {
    let payer_pk = payer.pubkey();
    let blockhash = svm.latest_blockhash();
    let ix = system_account_compress(
        payer_pk, 
        vm_address, 
        vm_memory, 
        vm_storage, 
        account_index, 
        signature
    );
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer_pk), &[payer], blockhash);

    send_tx(svm, tx)
}

pub fn tx_account_decompress(
    svm: &mut LiteSVM,
    payer: &Keypair,
    vm_address: Pubkey,
    vm_memory: Pubkey,
    vm_storage: Pubkey,
    unlock_pda: Option<Pubkey>,
    withdraw_receipt: Option<Pubkey>,
    account_index: u16,
    packed_va: Vec<u8>,
    proof: Vec<Hash>,
    signature: Signature,
) -> TransactionResult {
    let payer_pk = payer.pubkey();
    let blockhash = svm.latest_blockhash();
    let ix = system_account_decompress(
        payer_pk, 
        vm_address, 
        vm_memory, 
        vm_storage, 
        unlock_pda, 
        withdraw_receipt, 
        account_index, 
        packed_va, 
        proof, 
        signature
    );
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer_pk), &[payer], blockhash);

    send_tx(svm, tx)
}

pub fn tx_exec_opcode(
    svm: &mut LiteSVM,
    payer: &Keypair,
    vm_address: Pubkey,
    mem_a: Option<Pubkey>,
    mem_b: Option<Pubkey>,
    mem_c: Option<Pubkey>,
    mem_d: Option<Pubkey>,
    vm_omnibus: Option<Pubkey>,
    relay: Option<Pubkey>,
    relay_vault: Option<Pubkey>,
    external_address: Option<Pubkey>,
    token_program: Option<Pubkey>,
    opcode: u8,
    mem_indicies: Vec<u16>,
    mem_banks: Vec<u8>,
    data: Vec<u8>,
) -> TransactionResult {
    let payer_pk = payer.pubkey();
    let blockhash = svm.latest_blockhash();
    let ix = vm_exec(
        payer_pk,
        vm_address,
        mem_a,
        mem_b,
        mem_c,
        mem_d,
        vm_omnibus,
        relay,
        relay_vault,
        external_address,
        token_program,
        opcode,
        mem_indicies,
        mem_banks,
        data,
    );
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer_pk), &[payer], blockhash);

    send_tx(svm, tx)
}

pub fn tx_create_relay(
    svm: &mut LiteSVM,
    payer: &Keypair,
    mint: &Pubkey,
    vm_address: Pubkey,
    name: &str,
) -> TransactionResult {
    let payer_pk = payer.pubkey();
    let blockhash = svm.latest_blockhash();
    let ix = relay_init(payer_pk, vm_address, *mint, name);
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer_pk), &[payer], blockhash);

    send_tx(svm, tx)
}

pub fn tx_save_root(
    svm: &mut LiteSVM,
    payer: &Keypair,
    vm_address: Pubkey,
    relay: Pubkey,
) -> TransactionResult {
    let payer_pk = payer.pubkey();
    let blockhash = svm.latest_blockhash();
    let ix = relay_save_root(payer_pk, vm_address, relay);
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer_pk), &[payer], blockhash);

    send_tx(svm, tx)
}

pub fn tx_unlock_init(
    svm: &mut LiteSVM,
    payer: &Keypair,
    account_owner: &Keypair,
    vm_address: Pubkey,
    unlock_pda: Pubkey,
) -> TransactionResult {
    let owner = account_owner.pubkey();
    let payer_pk = payer.pubkey();
    let blockhash = svm.latest_blockhash();

    let ix = timelock_unlock_init(
        owner,
        payer_pk,
        vm_address,
        unlock_pda,
    );

    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer_pk), &[payer, account_owner], blockhash);

    send_tx(svm, tx)
}

pub fn tx_unlock_finalize(
    svm: &mut LiteSVM,
    payer: &Keypair,
    account_owner: &Keypair,
    vm_address: Pubkey,
    unlock_pda: Pubkey,
) -> TransactionResult {
    let owner = account_owner.pubkey();
    let payer_pk = payer.pubkey();
    let blockhash = svm.latest_blockhash();

    let ix = timelock_unlock_finalize(
        owner,
        payer_pk,
        vm_address,
        unlock_pda,
    );

    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer_pk), &[payer, account_owner], blockhash);

    send_tx(svm, tx)
}

pub fn tx_deposit(
    svm: &mut LiteSVM,
    payer: &Keypair,
    vm_address: Pubkey,
    vm_memory: Pubkey,
    depositor: Pubkey,
    deposit_pda: Pubkey,
    deposit_ata: Pubkey,
    omnibus: Pubkey,
    account_index: u16,
    amount: u64,
    bump: u8,
) -> TransactionResult {
    let payer_pk = payer.pubkey();
    let blockhash = svm.latest_blockhash();

    let ix = timelock_deposit_from_pda(
        payer_pk,
        vm_address,
        vm_memory,
        depositor,
        deposit_pda,
        deposit_ata,
        omnibus,
        account_index,
        amount,
        bump,
    );

    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer_pk), &[payer], blockhash);

    send_tx(svm, tx)
}

pub fn tx_withdraw_from_deposit(
    svm: &mut LiteSVM,
    payer: &Keypair,
    owner: &Keypair,
    vm_address: Pubkey,
    deposit_pda: Pubkey,
    deposit_ata: Pubkey,
    unlock_pda: Pubkey,
    external_address: Pubkey,
    data: WithdrawIxData,
) -> TransactionResult {
    let depositor = owner.pubkey();
    let payer_pk = payer.pubkey();
    let blockhash = svm.latest_blockhash();

    let ix = timelock_withdraw(
        depositor,
        payer_pk,
        vm_address,
        None, // vm_omnibus
        None, // vm_memory
        None, // vm_storage
        Some(deposit_pda),
        Some(deposit_ata),
        unlock_pda,
        None, // withdraw_receipt
        external_address,
        data,
    );

    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer_pk), &[payer, owner], blockhash);

    send_tx(svm, tx)
}

pub fn tx_withdraw_from_memory(
    svm: &mut LiteSVM,
    payer: &Keypair,
    owner: &Keypair,
    vm_address: Pubkey,
    vm_omnibus: Pubkey,
    vm_memory: Pubkey,
    unlock_pda: Pubkey,
    withdraw_receipt: Pubkey,
    external_address: Pubkey,
    data: WithdrawIxData,
) -> TransactionResult {
    let depositor = owner.pubkey();
    let payer_pk = payer.pubkey();
    let blockhash = svm.latest_blockhash();

    let ix = timelock_withdraw(
        depositor,
        payer_pk,
        vm_address,
        Some(vm_omnibus),
        Some(vm_memory),
        None, // vm_storage
        None, // deposit_pda
        None, // deposit_ata
        unlock_pda,
        Some(withdraw_receipt),
        external_address,
        data,
    );

    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer_pk), &[payer, owner], blockhash);

    send_tx(svm, tx)
}

pub fn tx_withdraw_from_storage(
    svm: &mut LiteSVM,
    payer: &Keypair,
    owner: &Keypair,
    vm_address: Pubkey,
    vm_omnibus: Pubkey,
    vm_storage: Pubkey,
    unlock_pda: Pubkey,
    withdraw_receipt: Pubkey,
    external_address: Pubkey,
    data: WithdrawIxData,
) -> TransactionResult {
    let depositor = owner.pubkey();
    let payer_pk = payer.pubkey();
    let blockhash = svm.latest_blockhash();

    let ix = timelock_withdraw(
        depositor,
        payer_pk,
        vm_address,
        Some(vm_omnibus),
        None, // vm_memory
        Some(vm_storage),
        None, // deposit_pda
        None, // deposit_ata
        unlock_pda,
        Some(withdraw_receipt),
        external_address,
        data,
    );

    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer_pk), &[payer, owner], blockhash);

    send_tx(svm, tx)
}