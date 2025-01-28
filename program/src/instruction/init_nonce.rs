use code_vm_api::prelude::*;
use steel::*;

/*
    This instruction initializes a new virtual durable nonce. The nonce is
    functionally similar to a real durable nonce within Solana, but is stored in
    the VM's memory.

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

    0. account_index: u16  - The location in the VM's paged memory to create the account.
    0. nonce_bump: u8      - The bump seed for the nonce account address.
*/
pub fn process_init_nonce(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    let args = InitNonceIx::try_from_bytes(data)?.to_struct()?;
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

    let seed = virtual_account_owner_info.key;
    let (nonce_address, _) = find_virtual_nonce_pda(
        &vm_info.key, seed, &vm.get_current_poh()
    );

    let vdn = VirtualDurableNonce {
        address: nonce_address,
        value: vm.get_current_poh(),
    };

    let va = VirtualAccount::Nonce(vdn);

    try_write(vm_memory_info, args.account_index, &va)?;

    vm.advance_poh(CodeInstruction::InitNonceIx, accounts, data);

    Ok(())
}
