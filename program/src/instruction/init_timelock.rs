use code_vm_api::{prelude::*, pdas};
use steel::*;

/*
    This instruction initializes a virtual timelock account. A timelock account
    forms a state channel between the VM authority and the timelock account
    owner. This allows instant transfers of tokens. 
    
    A timelock account can be non-custodially unlocked by the owner using the
    init_unlock and unlock instructions.

    Accounts expected by this instruction:
    
    | # | R/W | Type    | PDA | Name                   | Description                              |
    |---|-----|---------|-----|------------------------|------------------------------------------|
    | 0 | mut | Signer  |     | vm_authority           | The authority of the VM.                 |
    | 1 | mut | Vm      | PDA | vm                     | The VM instance state account.           |
    | 2 | mut | Memory  | PDA | vm_memory              | Where to create the virtual account.     |
    | 3 |     | Address |     | virtual_account_owner  | The virtual account owner.               |


    Derived account seeds:

    1. vm:         [ "code_vm", <mint>, <vm_authority>, <lock_duration> ]
    2. vm_memory:  [ "code_vm", "vm_memory_account", <self.name>, <vm> ]

    Instruction data:

    0. account_index: u16        - The location in the VM's paged memory to create the account.
    1. virtual_timelock_bump: u8 - The bump seed for the virtual timelock account.
    2. virtual_vault_bump: u8    - The bump seed for the virtual token account.
    3. unlock_pda_bump: u8       - The bump seed for the unlock PDA address.

*/
pub fn process_init_timelock(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {

    let args = InitTimelockIx::try_from_bytes(data)?.to_struct()?;
    let [
        vm_authority_info,
        vm_info,
        vm_memory_info,
        virtual_account_owner_info,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);        
    };

    check_signer(vm_authority_info)?;
    check_mut(vm_info)?;
    check_mut(vm_memory_info)?;
    check_readonly(virtual_account_owner_info)?;

    let vm = load_vm_checked(vm_info, vm_authority_info)?;

    check_memory(vm_memory_info, vm_info)?;
    check_is_empty(vm_memory_info, args.account_index)?;

    let owner = virtual_account_owner_info.key.clone();
    let nonce = vm.get_current_poh();

    let (timelock_address, timelock_bump) = pdas::find_virtual_timelock_address(
        &vm.get_mint(), 
        &vm.get_authority(), 
        &owner, 
        vm.get_lock_duration(), 
    );

    if args.virtual_timelock_bump != timelock_bump {
        return Err(ProgramError::InvalidArgument);
    }

    let (unlock_address, unlock_bump) = pdas::find_unlock_address(
        &owner, 
        &timelock_address, 
        vm_info.key);
    
    if args.unlock_pda_bump != unlock_bump {
        return Err(ProgramError::InvalidArgument);
    }

    // We could technically require the user to provide the withdraw_bump,
    // however, that would make using this instruction more cumbersome since the
    // nonce value is determined above.
    let (_, withdraw_bump) = pdas::find_withdraw_receipt_address( // This call *can* be expensive
        &unlock_address, 
        &nonce, 
        vm_info.key);

    let vta = VirtualTimelockAccount {
        owner,
        instance: nonce,
        bump: args.virtual_timelock_bump,
        token_bump: args.virtual_vault_bump,
        unlock_bump: args.unlock_pda_bump,
        withdraw_bump,
        balance: 0,
    };
    let va = VirtualAccount::Timelock(vta);

    try_write(vm_memory_info, args.account_index, &va)?;

    vm.advance_poh(CodeInstruction::InitTimelockIx, accounts, data);

    Ok(())
}

