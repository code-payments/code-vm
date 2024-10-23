use code_vm_api::prelude::*;
use steel::*;
use solana_program::msg;

/*
    This instruction is used to begin the unlock process for a timelocked
    account. Once the VM lockduration has passed, the owner can finalize the
    unlock process and withdraw their funds non-custodially.

    Accounts expected by this instruction:
    
    | # | R/W | Type        | PDA | Name           | Description                       |
    |---|-----|-------------|-----|----------------|-----------------------------------|
    | 0 | mut | Signer      |     | account_owner  | The virtual account owner.        |
    | 1 | mut | Signer      |     | payer          | The transaction fee payer.        |
    | 2 | mut | Vm          | PDA | vm             | The VM instance state account.    |
    | 3 | mut | UnlockState | PDA | unlock_pda     | Account to create.                |
    | 4 |     | Program     |     | system_program | The system program.               |
    | 5 |     | Sysvar      |     | rent_sysvar    | The rent sysvar.                  |


    Derived account seeds:

    2. vm:          [ "code_vm", <mint>, <vm_authority>, <lock_duration> ]
    3. unlock_pda:  [ "code_vm", "vm_unlock_pda_account", <account_owner>, <timelock_address>, <vm> ]

*/
pub fn process_init_unlock(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {

    let [
        account_owner_info,
        payer_info,
        vm_info,
        unlock_pda_info,
        system_program_info,
        rent_sysvar_info,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);        
    };

    check_signer(account_owner_info)?;
    check_signer(payer_info)?;
    check_program(system_program_info, &system_program::id())?;
    check_sysvar(rent_sysvar_info, &sysvar::rent::id())?;

    let vm = vm_info.to_account_mut::<CodeVmAccount>(&code_vm_api::ID)?;

    check_seeds(
        vm_info, 
        &[
            CODE_VM, 
            vm.mint.as_ref(), 
            vm.authority.as_ref(), 
            vm.lock_duration.to_le_bytes().as_ref()
        ],
        vm.bump,
        &code_vm_api::ID
    )?;

    let (timelock_address, _) = find_virtual_timelock_address(
        &vm.get_mint(), 
        &vm.get_authority(), 
        account_owner_info.key, 
        vm.get_lock_duration(),
    );

    let (unlock_pda, bump) = find_unlock_address(
        account_owner_info.key,
        &timelock_address,
        vm_info.key,
    );

    check_condition(
        unlock_pda.eq(&unlock_pda_info.key),
        "unlock PDA does not match the given owner",
    )?;

    check_uninitialized_pda(
        unlock_pda_info, 
        &[
            CODE_VM,
            VM_UNLOCK_ACCOUNT,
            account_owner_info.key.as_ref(),
            timelock_address.as_ref(),
            vm_info.key.as_ref(),
        ],
        bump, 
        &code_vm_api::id()
    )?;

    create_account::<UnlockStateAccount>(
        unlock_pda_info,
        &code_vm_api::ID,
        &[
            CODE_VM,
            VM_UNLOCK_ACCOUNT,
            account_owner_info.key.as_ref(),
            timelock_address.as_ref(),
            vm_info.key.as_ref(),
            &[bump]
        ],
        system_program_info,
        payer_info,
    )?;

    let now = Clock::get()?.unix_timestamp;
    let second_per_day = 86400; // 60sec * 60min * 24hrs = 86400
    let mut unlock_at = now + (vm.lock_duration as i64 * second_per_day); 
    if unlock_at % second_per_day > 0 {
        unlock_at = unlock_at + (second_per_day - (unlock_at % second_per_day))
    }

    let unlock_pda = 
        unlock_pda_info.to_account_mut::<UnlockStateAccount>(&code_vm_api::id())?;

    unlock_pda.vm = vm_info.key.clone();
    unlock_pda.bump = bump;
    unlock_pda.owner = account_owner_info.key.clone();
    unlock_pda.address = timelock_address;
    unlock_pda.state = TimelockState::WaitingForTimeout as u8;
    unlock_pda.unlock_at = unlock_at;

    msg!("current time: {}", now);
    msg!("the timelock can be released after: {}", unlock_at);

    vm.advance_poh(CodeInstruction::InitUnlockIx, accounts, data);

    Ok(())
}

