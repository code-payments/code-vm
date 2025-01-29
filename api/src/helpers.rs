use steel::*;
use solana_program::msg;

use crate::{
    consts::*, 
    cvm::{
        CodeVmAccount, MemoryAccount, RelayAccount, StorageAccount, VirtualAccount 
    },
    types::{Hash, SliceAllocator, SliceAllocatorMut},
};

pub fn optional_meta(account: Option<Pubkey>, is_signer: bool) -> AccountMeta {
    match account {
        Some(account) => AccountMeta::new(account, is_signer),
        None => AccountMeta::new(crate::ID, is_signer),
    }
}

pub fn optional_readonly_meta(account: Option<Pubkey>, is_signer: bool) -> AccountMeta {
    match account {
        Some(account) => AccountMeta::new_readonly(account, is_signer),
        None => AccountMeta::new_readonly(crate::ID, is_signer),
    }
}

pub fn get_optional<'a, 'b>(account: &'a AccountInfo<'b>) 
    -> Option<&'a AccountInfo<'b>> {
    match account.key {
        &crate::ID => None,
        _ => Some(account),
    }
}

pub fn check_condition(condition: bool, message: &str) -> ProgramResult {
    if !condition {
        msg!("Failed condition: {}", message);
        return Err(ProgramError::InvalidArgument);
    }
    Ok(())
}

pub fn check_signer(account: &AccountInfo) -> ProgramResult {
    account.is_signer()?.is_writable()?;
    Ok(())
}

pub fn check_mut(account: &AccountInfo) -> ProgramResult {
    account.is_writable()?;
    Ok(())
}

pub fn check_unique(accounts: &[&AccountInfo], message: &str) -> ProgramResult {
    let num_accounts = accounts.len();
    for i in 0..num_accounts {
        for j in (i + 1)..num_accounts {
            if accounts[i].key == accounts[j].key {
                msg!("Failed unique constraint: {}", message);
                return Err(ProgramError::InvalidArgument);
            }
        }
    }
    Ok(())
}

pub fn check_readonly(account: &AccountInfo) -> ProgramResult {
    // Technically, this check is nonsense. We could allow writable accounts,
    // but for now we're going to be super opinionated about the assumptions and
    // not encourage unexpected packing of instructions.

    if account.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(())
}

pub fn check_uninitialized_pda(account: &AccountInfo, seeds: &[&[u8]], bump: u8, program_id: &Pubkey) -> ProgramResult {
    if !account.owner.eq(&system_program::ID) {
        return Err(ProgramError::InvalidAccountData);
    }

    account.is_empty()?.is_writable()?.has_seeds(seeds, bump, program_id)?;
    Ok(())
}

pub fn check_seeds(account: &AccountInfo, seeds: &[&[u8]], bump: u8, program_id: &Pubkey) -> ProgramResult {
    account.has_seeds(seeds, bump, program_id)?;
    Ok(())
}

pub fn check_program(account: &AccountInfo, program_id: &Pubkey) -> ProgramResult {
    account.is_program(program_id)?;
    Ok(())
}

pub fn check_sysvar(account: &AccountInfo, sysvar_id: &Pubkey) -> ProgramResult {
    account.is_sysvar(sysvar_id)?;
    Ok(())
}

pub fn load_vm<'a>(
    vm_info: &'a AccountInfo<'_>,
) -> Result<&'a mut CodeVmAccount, ProgramError> {

    let vm = 
        vm_info.to_account_mut::<CodeVmAccount>(&crate::ID)?;

    check_seeds(
        vm_info, 
        &[
            CODE_VM, 
            vm.mint.as_ref(), 
            vm.authority.as_ref(), 
            vm.lock_duration.to_le_bytes().as_ref()
        ],
        vm.bump,
        &crate::ID
    )?;

    Ok(vm)
}

pub fn load_vm_checked<'a>(
    vm_info: &'a AccountInfo<'_>,
    vm_authority_info: &'a AccountInfo<'_>
) -> Result<&'a mut CodeVmAccount, ProgramError> {

    let vm = 
        vm_info.to_account_mut::<CodeVmAccount>(&crate::ID)?;

    check_condition(
        vm.authority.eq(vm_authority_info.key),
        "vm_authority does not match the authority of the VM account",
    )?;

    check_seeds(
        vm_info, 
        &[
            CODE_VM, 
            vm.mint.as_ref(), 
            vm.authority.as_ref(), 
            vm.lock_duration.to_le_bytes().as_ref()
        ],
        vm.bump,
        &crate::ID
    )?;

    Ok(vm)
}

pub fn load_memory<'a>(
    vm_memory_info: &'a AccountInfo<'_>, 
    vm_info: &'a AccountInfo<'_>
) -> Result<&'a mut MemoryAccount, ProgramError> {
    let memory = 
        vm_memory_info.to_account_mut::<MemoryAccount>(&crate::ID)?;

    check_seeds(
        vm_memory_info, 
        &[
            CODE_VM, 
            VM_MEMORY_ACCOUNT,
            memory.name.as_ref(),
            vm_info.key.as_ref()
        ],
        memory.bump, 
        &crate::ID
    )?;

    check_condition(
        memory.vm.eq(vm_info.key),
        "vm does not match the VM account",
    )?;

    Ok(memory)
}

pub fn load_storage<'a>(
    vm_storage_info: &'a AccountInfo<'_>,
    vm_info: &'a AccountInfo<'_>
) -> Result<&'a StorageAccount, ProgramError> {
    let storage = 
        vm_storage_info.to_account_mut::<StorageAccount>(&crate::ID)?;

    check_seeds(
        vm_storage_info, 
        &[
            CODE_VM, 
            VM_STORAGE_ACCOUNT,
            storage.name.as_ref(),
            vm_info.key.as_ref()
        ],
        storage.bump, 
        &crate::ID
    )?;

    check_condition(
        storage.vm.eq(vm_info.key),
        "vm does not match the VM account",
    )?;

    check_condition(
        storage.depth == COMPRESSED_STATE_DEPTH as u8,
        "storage depth is not equal to COMPRESSED_STATE_DEPTH",
    )?;

    Ok(storage)
}

pub fn load_relay<'a>(
    relay_info: &'a AccountInfo<'_>,
    vm_info: &'a AccountInfo<'_>
) -> Result<&'a RelayAccount, ProgramError> {
    let relay = 
        relay_info.to_account_mut::<RelayAccount>(&crate::ID)?;

    check_seeds(
        relay_info, 
        &[
            CODE_VM, 
            VM_RELAY_ACCOUNT,
            relay.name.as_ref(),
            vm_info.key.as_ref()
        ],
        relay.bump, 
        &crate::ID
    )?;

    check_condition(
        relay.vm.eq(vm_info.key),
        "vm does not match the VM account",
    )?;

    check_condition(
        relay.num_levels == RELAY_STATE_DEPTH as u8,
        "relay depth is not equal to RELAY_STATE_DEPTH",
    )?;

    Ok(relay)
}

pub fn check_memory(
    vm_memory_info: &AccountInfo<'_>, 
    vm_info: &AccountInfo<'_>
) -> ProgramResult {
    load_memory(vm_memory_info, vm_info)?;
    Ok(())
}

pub fn check_storage(
    vm_storage_info: &AccountInfo<'_>, 
    vm_info: &AccountInfo<'_>
) -> ProgramResult {
    load_storage(vm_storage_info, vm_info)?;
    Ok(())
}

pub fn check_relay(
    relay_info: &AccountInfo<'_>, 
    vm_info: &AccountInfo<'_>
) -> ProgramResult {
    load_relay(relay_info, vm_info)?;
    Ok(())
}

pub fn check_omnibus(
    omnibus_info: &AccountInfo<'_>, 
    vm_info: &AccountInfo<'_>
) -> ProgramResult {
    let vm = load_vm(vm_info)?;

    check_seeds(
        omnibus_info, 
        &[
            CODE_VM, 
            VM_OMNIBUS,
            vm_info.key.as_ref()
        ],
        vm.get_omnibus_bump(), 
        &crate::ID
    )?;

    Ok(())
}

pub fn check_is_empty<'a>(
    vm_memory: &AccountInfo<'_>,
    account_index: u16,
) -> ProgramResult {

    let (n, m) = MemoryAccount::get_capacity_and_size(vm_memory);
    let data = MemoryAccount::get_data(vm_memory)?;
    let mem = SliceAllocator::try_from_slice(& *data, n, m)?;

    check_condition(
        mem.is_empty(account_index),
        "the virtual account is already allocated",
    )?;

    Ok(())
}

pub fn try_read<'a>(
    vm_memory: &AccountInfo<'_>,
    account_index: u16,
) -> Result<VirtualAccount, ProgramError> {

    let (n, m) = MemoryAccount::get_capacity_and_size(vm_memory);
    let data = MemoryAccount::get_data(vm_memory)?;
    let mem = SliceAllocator::try_from_slice(& *data, n, m)?;

    check_condition(
        mem.has_item(account_index),
        "the virtual account is not allocated",
    )?;

    let account = mem.read_item(account_index);
    check_condition(
        account.is_some(),
        "unable to read the virtual account from the memory",
    )?;

    let va = VirtualAccount::unpack(&account.unwrap())?;
    Ok(va)
}

pub fn try_write<'a>(
    vm_memory: &AccountInfo<'_>,
    account_index: u16,
    account: &VirtualAccount,
) -> ProgramResult {

    let (n, m) = MemoryAccount::get_capacity_and_size(vm_memory);
    let mut data = MemoryAccount::get_data_mut(vm_memory)?;
    let mut mem = SliceAllocatorMut::try_from_slice_mut(&mut *data, n, m)?;
    
    let data = &account.pack();

    if mem.is_empty(account_index) {
        mem.try_alloc_item(account_index, data.len())?;
    }
    mem.try_write_item(account_index, data)?;

    Ok(())
} 

pub fn try_delete<'a>(
    vm_memory: &AccountInfo<'_>,
    account_index: u16,
) -> ProgramResult {

    let (n, m) = MemoryAccount::get_capacity_and_size(vm_memory);
    let mut data = MemoryAccount::get_data_mut(vm_memory)?;
    let mut mem = SliceAllocatorMut::try_from_slice_mut(&mut *data, n, m)?;

    if mem.has_item(account_index) {
        mem.try_free_item(account_index)?;
    }

    Ok(())
} 

pub fn try_compress<'a>(
    vm_storage: &AccountInfo<'_>,
    leaf: Hash,
) -> ProgramResult {
    let storage = 
        StorageAccount::get_compressed_state_mut(vm_storage)?;

    storage.try_insert(leaf)?;

    Ok(())
}

pub fn try_decompress<'a>(
    vm_storage: &AccountInfo<'_>,
    leaf: Hash,
    proof: &[Hash],
) -> ProgramResult {
    let storage = 
        StorageAccount::get_compressed_state_mut(vm_storage)?;

    storage.try_remove(proof, leaf)?;

    Ok(())
}


pub fn create_name(name: &str) -> [u8; MAX_NAME_LEN] {
    let mut name_bytes = [0u8; MAX_NAME_LEN];
    name_bytes[..name.len()].copy_from_slice(name.as_bytes());
    name_bytes
}