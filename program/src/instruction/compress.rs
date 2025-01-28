use code_vm_api::prelude::*;
use steel::*;

/*
    This instruction is used to compress an account in the VM's working memory
    (compact_mem) into the VM's cold storage (compressed_mem). This action takes
    the virtual account state off chain and stores it in a more efficient form
    on chain.

    Before an account is compressed, the data of that account is hashed and
    signed by the VM authority. This signature is used to prove that the account
    was witnessed by the VM authority as it currently exists in the VM's working
    memory before it is compressed.

    Accounts expected by this instruction:

    | # | R/W | Type    | PDA | Name         | Description                              |
    |---|-----|---------|-----|--------------|------------------------------------------|
    | 0 | mut | Signer  |     | vm_authority | The authority of the VM.                 |
    | 1 | mut | Vm      | PDA | vm           | The VM instance state account.           |
    | 2 | mut | Memory  | PDA | vm_memory    | The memory account to pull from.         |
    | 3 | mut | Storage | PDA | vm_storage   | The storage account to push to.          |


    Derived account seeds:

    1. vm:         [ "code_vm", <mint>, <vm_authority>, <lock_duration> ]
    2. vm_memory:  [ "code_vm", "vm_memory_account", <self.name>, <vm> ]
    3. vm_storage: [ "code_vm", "vm_storage_account", <self.name>, <vm> ]

    Instruction data:

    0. account_index: u16   - The index of the account in the VM's paged memory.
    1. signature: [u8; 64]  - A signature of the current account state signed by the VM authority.
*/
pub fn process_compress(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    let args = CompressIx::try_from_bytes(data)?.to_struct()?;
    let [vm_authority_info, vm_info, vm_memory_info, vm_storage_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    check_signer(vm_authority_info)?;
    check_mut(vm_info)?;
    check_mut(vm_memory_info)?;
    check_mut(vm_storage_info)?;

    let vm = load_vm_checked(vm_info, vm_authority_info)?;

    check_memory(vm_memory_info, vm_info)?;
    check_storage(vm_storage_info, vm_info)?;

    let va = try_read(vm_memory_info, args.account_index)?;
    let va_hash = va.get_hash();

    sig_verify(
        vm_authority_info.key.as_ref(),
        args.signature.as_ref(),
        va_hash.as_ref(),
    )?;

    let sig_hash = hashv(&[args.signature.as_ref(), va_hash.as_ref()]);

    try_compress(vm_storage_info, sig_hash)?;
    try_delete(vm_memory_info, args.account_index)?;

    vm.advance_poh(CodeInstruction::CompressIx, accounts, data);

    Ok(())
}
