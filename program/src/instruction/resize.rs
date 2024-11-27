use code_vm_api::prelude::*;
use solana_program::{
    system_program,
    sysvar,
};
use steel::*;

/*
    This instruction resizes a memory account to the specified size. Only increasing
    the size of the account is allowed.

    This instruction works around the limitations the Solana runtime imposes on
    the maximum size of an account is allowed to be when it is created. Calling
    this instruction repeatedly will resize the memory account to the required
    size.

    Accounts expected by this instruction:
    
    | # | R/W | Type    | PDA | Name           | Description                              |
    |---|-----|---------|-----|----------------|------------------------------------------|
    | 0 | mut | Signer  |     | vm_authority   | The authority of the VM.                 |
    | 1 | mut | Vm      | PDA | vm             | The VM instance state account.           |
    | 2 | mut | Memory  | PDA | vm_memory      | The memory account to realloc.           |
    | 3 |     | Program |     | system_program | The system program.                      |
    | 4 |     | Sysvar  |     | rent_sysvar    | The rent sysvar.                         |


    Derived account seeds:

    1. vm:        [ "code_vm", <mint>, <vm_authority>, <lock_duration> ]
    2. vm_memory: [ "code_vm", "vm_memory_account", <self.name>, <vm> ]


    Instruction data:

    0. len: u32             - The new size of the vm_memory account.
*/
pub fn process_resize(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {

    let args = ResizeMemoryIx::try_from_bytes(data)?;
    let [
        vm_authority_info,
        vm_info,
        vm_memory_info,
        system_program_info,
        rent_sysvar_info 
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);        
    };

    check_condition(
        args.account_size as usize > MemoryAccount::get_size(),
        "account_size must be greater than the base size of a memory account",
    )?;

    check_signer(vm_authority_info)?;
    check_mut(vm_info)?;
    check_mut(vm_memory_info)?;
    check_program(system_program_info, &system_program::id())?;
    check_sysvar(rent_sysvar_info, &sysvar::rent::id())?;

    let vm = load_vm_checked(vm_info, vm_authority_info)?;
    let memory = load_memory(vm_memory_info, vm_info)?;

    let capacity = memory.num_accounts as usize;
    let account_size = memory.account_size as usize;

    let max_size = MemoryAccount::get_size_with_data(capacity, account_size);
    check_condition(
        args.account_size as usize <= max_size,
        "account_size must be less than or equal to the maximum size for this type of memory account",
    )?; 

    check_condition(
        args.account_size as usize >= vm_memory_info.data_len(),
        "account_size must be greater than or equal to the current size of the memory account",
    )?;

    resize_account(
        vm_memory_info,
        vm_authority_info,
        args.account_size as usize,
        system_program_info,
    )?;

    vm.advance_poh(CodeInstruction::ResizeMemoryIx, accounts, data);

    Ok(())
}

