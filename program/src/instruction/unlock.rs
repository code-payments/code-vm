use code_vm_api::prelude::*;
use steel::*;
use solana_program::msg;

/*
    This instruction is used to finalize a timelock unlock. This allows the 
    owner to issue a non-custodial withdraw instruction to claim the balance of
    any linked virtual account or deposit.

    Accounts expected by this instruction:
    
    | # | R/W | Type        | PDA | Name           | Description                       |
    |---|-----|-------------|-----|----------------|-----------------------------------|
    | 0 | mut | Signer      |     | account_owner  | The virtual account owner.        |
    | 1 | mut | Signer      |     | payer          | The transaction fee payer.        |
    | 2 | mut | Vm          | PDA | vm             | The VM instance state account.    |
    | 3 | mut | UnlockState | PDA | unlock_pda     | Account to create.                |


    Derived account seeds:

    2. vm:          [ "code_vm", <mint>, <vm_authority>, <lock_duration> ]
    3. unlock_pda:  [ "code_vm", "vm_unlock_pda_account", <account_owner>, <timelock_address>, <vm> ]

*/
pub fn process_unlock(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {

    let [
        account_owner_info,
        payer_info,
        vm_info,
        unlock_pda_info,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);        
    };

    check_signer(account_owner_info)?;
    check_signer(payer_info)?;

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

    let unlock_pda = unlock_pda_info.to_account_mut::<UnlockStateAccount>(&code_vm_api::ID)?;

    check_seeds(
        unlock_pda_info, 
        &[
            CODE_VM,
            VM_UNLOCK_ACCOUNT,
            account_owner_info.key.as_ref(),
            unlock_pda.address.as_ref(),
            vm_info.key.as_ref(),
        ],
        unlock_pda.bump, 
        &code_vm_api::id()
    )?;

    check_condition(
        unlock_pda.state == TimelockState::WaitingForTimeout as u8,
        "invalid unlock state"
    )?;

    let now = Clock::get()?.unix_timestamp;

    msg!("current time: {}", now);
    msg!("unlock time: {}", unlock_pda.unlock_at);
    
    check_condition(
        unlock_pda.unlock_at < now,
        "unlock time has not passed yet"
    )?;

    unlock_pda.state = TimelockState::Unlocked as u8;

    vm.advance_poh(CodeInstruction::UnlockIx, accounts, data);

    Ok(())
}

