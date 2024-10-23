use code_vm_api::prelude::*;
use steel::*;

/*
    This instruction saves the current root of the relay into a circular buffer
    in case it needs to be accessed later for a proof.

    Accounts expected by this instruction:
    
    | # | R/W | Type    | PDA | Name           | Description                              |
    |---|-----|---------|-----|----------------|------------------------------------------|
    | 0 | mut | Signer  |     | vm_authority   | The authority of the VM.                 |
    | 1 | mut | Vm      | PDA | vm             | The VM instance state account.           |
    | 2 | mut | Relay   | PDA | relay          | The relay to save a recent root on.      |

    Derived account seeds:

    1. vm:        [ "code_vm", <mint>, <vm_authority>, <lock_duration> ]
    2. relay:     [ "code_vm", "vm_relay_account", <self.name>, <vm> ]

    Instruction data:

    <none>
*/
pub fn process_snapshot(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {

    let [
        vm_authority_info,
        vm_info,
        relay_info,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);        
    };

    check_signer(vm_authority_info)?;
    check_mut(vm_info)?;
    check_mut(relay_info)?;
    check_relay(relay_info, vm_info)?;

    let relay = 
        relay_info.to_account_mut::<RelayAccount>(&code_vm_api::ID)?;

    relay.save_recent_root();

    let vm = load_vm_checked(vm_info, vm_authority_info)?;
    vm.advance_poh(CodeInstruction::SnapshotIx, accounts, data);

    Ok(())
}

