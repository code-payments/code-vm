use code_vm_api::prelude::*;
use solana_program::{
    system_program,
    sysvar,
};
use steel::*;

/*
    This instruction initializes a new VM instance owned by the given authority.

    Accounts expected by this instruction:
    
    | # | R/W | Type         | PDA | Name           | Description                              |
    |---|-----|------------- |-----|----------------|------------------------------------------|
    | 0 | mut | Signer       |     | vm_authority   | The authority of the VM.                 |
    | 1 | mut | Vm           | PDA | vm             | The VM instance state account.           |
    | 2 | mut | TokenAccount | PDA | omnibus        | A derived token account owned by the VM. |
    | 3 |     | TokenMint    |     | mint           | The mint to use for this VM instance.    |
    | 4 |     | Program      |     | token_program  | The SPL token program.                   |
    | 5 |     | Program      |     | system_program | The system program.                      |
    | 6 |     | Sysvar       |     | rent_sysvar    | The rent sysvar.                         |


    Derived account seeds:

    1. vm:      [ "code_vm", <mint>, <vm_authority>, <lock_duration> ]
    2. omnibus: [ "code_vm", "vm_omnibus", <vm> ]


    Instruction data:

    0. lock_duration: u8    - The duration in days for timelocked accounts created by this VM.
    1. vm_bump: u8          - The bump seed for the VM instance account.
    2. vm_omnibus_bump: u8  - The bump seed for the VM's derived token account.
*/
pub fn process_init_vm(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {

    let args = InitVmIx::try_from_bytes(data)?;
    let [
        vm_authority_info,
        vm_info,
        omnibus_info,
        mint_info,
        token_program_info,
        system_program_info,
        rent_sysvar_info 
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);        
    };

    check_condition(
        args.lock_duration > 0, 
        "lock_duration must be greater than 0",
    )?;

    check_signer(vm_authority_info)?;
    check_mut(vm_info)?;
    check_mut(omnibus_info)?;
    check_readonly(mint_info)?;
    check_program(token_program_info, &spl_token::id())?;
    check_program(system_program_info, &system_program::id())?;
    check_sysvar(rent_sysvar_info, &sysvar::rent::id())?;

    check_uninitialized_pda(
        vm_info, 
        &[
            CODE_VM, 
            mint_info.key.as_ref(), 
            vm_authority_info.key.as_ref(), 
            args.lock_duration.to_le_bytes().as_ref()
        ],
        args.vm_bump,
        &code_vm_api::id()
    )?;
    check_uninitialized_pda(
        omnibus_info, 
        &[
            CODE_VM, 
            VM_OMNIBUS,
            vm_info.key.as_ref()
        ],
        args.vm_omnibus_bump, 
        &code_vm_api::id()
    )?;

    // Create the VM instance account.
    create_account::<CodeVmAccount>(
        vm_info,
        &code_vm_api::ID,
        &[
            CODE_VM, 
            mint_info.key.as_ref(), 
            vm_authority_info.key.as_ref(), 
            args.lock_duration.to_le_bytes().as_ref(),
            &[args.vm_bump]
        ],
        system_program_info,
        vm_authority_info,
    )?;

    // Create the VM's derived token account.
    create_token_account(
        mint_info,
        omnibus_info,
        &[
            CODE_VM, 
            VM_OMNIBUS, 
            vm_info.key.as_ref(),
            &[args.vm_omnibus_bump]
        ],
        vm_authority_info,
        system_program_info,
        rent_sysvar_info,
    )?;

    let vm = vm_info.to_account_mut::<CodeVmAccount>(&code_vm_api::ID)?;

    vm.authority = vm_authority_info.key.clone();
    vm.mint = mint_info.key.clone();
    vm.lock_duration = args.lock_duration;                                                                                                                                                      
    vm.bump = args.vm_bump;
    vm.omnibus.vault = omnibus_info.key.clone();
    vm.omnibus.vault_bump = args.vm_omnibus_bump;

    vm.advance_poh(CodeInstruction::InitVmIx, accounts, data);

    Ok(())
}

