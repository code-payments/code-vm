use code_vm_api::prelude::*;
use solana_program::{system_program, sysvar};
use steel::*;

/*
    This instruction initializes a new virtual account memory module for a VM
    instance. The VM is able to execute opcodes on virtual accounts that are
    stored in one of these.

    Accounts expected by this instruction:

    | # | R/W | Type    | PDA | Name           | Description                              |
    |---|-----|---------|-----|----------------|------------------------------------------|
    | 0 | mut | Signer  |     | vm_authority   | The authority of the VM.                 |
    | 1 | mut | Vm      | PDA | vm             | The VM instance state account.           |
    | 2 | mut | Memory  | PDA | vm_memory      | The memory account to create.            |
    | 3 |     | Program |     | system_program | The system program.                      |
    | 4 |     | Sysvar  |     | rent_sysvar    | The rent sysvar.                         |


    Derived account seeds:

    1. vm:        [ "code_vm", <mint>, <vm_authority>, <lock_duration> ]
    2. vm_memory: [ "code_vm", "vm_memory_account", <data.name>, <vm> ]


    Instruction data:

    0. name: [u8; 32]       - The name of this memory module.
    1. num_accounts: u32    - The number of accounts that can be stored in this memory module.
    2. account_size: u16    - The size of each account in this memory module.
    3. vm_memory_bump: u8   - The bump seed for the this memory account.
*/
pub fn process_init_memory(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    let args = InitMemoryIx::try_from_bytes(data)?;
    let [
        vm_authority_info,
        vm_info,
        vm_memory_info,
        system_program_info,
        rent_sysvar_info 
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);        
    };

    check_signer(vm_authority_info)?;
    check_mut(vm_info)?;
    check_mut(vm_memory_info)?;
    check_program(system_program_info, &system_program::id())?;
    check_sysvar(rent_sysvar_info, &sysvar::rent::id())?;

    let vm = load_vm_checked(vm_info, vm_authority_info)?;

    check_uninitialized_pda(
        vm_memory_info,
        &[
            CODE_VM,
            VM_MEMORY_ACCOUNT,
            args.name.as_ref(),
            vm_info.key.as_ref(),
        ],
        args.vm_memory_bump,
        &code_vm_api::id(),
    )?;

    create_account::<MemoryAccount>(
        vm_memory_info,
        &code_vm_api::ID,
        &[
            CODE_VM,
            VM_MEMORY_ACCOUNT,
            args.name.as_ref(),
            vm_info.key.as_ref(),
            &[args.vm_memory_bump],
        ],
        system_program_info,
        vm_authority_info,
    )?;

    let memory = vm_memory_info.to_account_mut::<MemoryAccount>(&code_vm_api::ID)?;

    memory.name = args.name;
    memory.vm = vm_info.key.clone();
    memory.bump = args.vm_memory_bump;
    memory.num_accounts = args.num_accounts as u32;
    memory.account_size = args.account_size as u16;
    memory.version = 1;

    vm.advance_poh(CodeInstruction::InitMemoryIx, accounts, data);

    Ok(())
}
