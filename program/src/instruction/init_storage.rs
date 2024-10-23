use code_vm_api::prelude::*;
use solana_program::{
    system_program,
    sysvar,
};
use steel::*;

/*
    This instruction initializes a new cold storage account for a VM instance.
    The VM can compress virtual accounts into one of these. This is useful for
    storing virtual accounts that are not frequently accessed or dormant. It
    reduces the rent cost to practically zero.
    
    Before an account can be compressed, it must be hashed and signed by the VM
    authority. This signature is used to prove that the account was witnessed by
    the VM authority as it currently exists on-chain. 

    Before an account can be used by the VM again, it must be decompressed. The
    decompress process works in reverse to this instruction but also requires a
    merkle proof.

    Accounts expected by this instruction:
    
    | # | R/W | Type    | PDA | Name           | Description                              |
    |---|-----|---------|-----|----------------|------------------------------------------|
    | 0 | mut | Signer  |     | vm_authority   | The authority of the VM.                 |
    | 1 | mut | Vm      | PDA | vm             | The VM instance state account.           |
    | 2 | mut | Storage | PDA | vm_storage     | The storage account to create.           |
    | 3 |     | Program |     | system_program | The system program.                      |
    | 4 |     | Sysvar  |     | rent_sysvar    | The rent sysvar.                         |


    Derived account seeds:

    1. vm:         [ "code_vm", <mint>, <vm_authority>, <lock_duration> ]
    2. vm_storage: [ "code_vm", "vm_storage_account", <data.name>, <vm> ]


    Instruction data:

    0. name: [u8; 32]       - The name of this storage module.
    1. vm_stroage_bump: u8  - The bump seed for the this memory account.
*/
pub fn process_init_storage(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {

    let args = InitStorageIx::try_from_bytes(data)?;
    let [
        vm_authority_info,
        vm_info,
        vm_storage_info,
        system_program_info,
        rent_sysvar_info 
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);        
    };

    check_signer(vm_authority_info)?;
    check_mut(vm_info)?;
    check_mut(vm_storage_info)?;
    check_program(system_program_info, &system_program::id())?;
    check_sysvar(rent_sysvar_info, &sysvar::rent::id())?;

    let vm = load_vm_checked(vm_info, vm_authority_info)?;

    check_uninitialized_pda(
        vm_storage_info, 
        &[
            CODE_VM, 
            VM_STORAGE_ACCOUNT,
            args.name.as_ref(),
            vm_info.key.as_ref()
        ],
        args.vm_storage_bump, 
        &code_vm_api::id()
    )?;

    create_account::<StorageAccount>(
        vm_storage_info,
        &code_vm_api::ID,
        &[
            CODE_VM, 
            VM_STORAGE_ACCOUNT,
            args.name.as_ref(),
            vm_info.key.as_ref(),
            &[args.vm_storage_bump]
        ],
        system_program_info,
        vm_authority_info,
    )?;

    let storage = vm_storage_info.to_account_mut::<StorageAccount>(&code_vm_api::ID)?;

    storage.vm = vm_info.key.clone();
    storage.bump = args.vm_storage_bump;
    storage.name = args.name;
    storage.depth = COMPRESSED_STATE_DEPTH as u8; // not really needed but we have a few free bytes.

    storage.compressed_state.init(&[
        MERKLE_TREE_SEED,
        &args.name.as_ref(),
        vm_info.key.as_ref()
    ]);

    vm.advance_poh(CodeInstruction::InitStorageIx, accounts, data);

    Ok(())
}

