use code_vm_api::prelude::*;
use steel::*;

/*
    This instruction is used to decompress a virtual account from the VM's cold
    storage (compressed_mem). The full account state along with a signature of
    the state is required in order to decompress it. The virtual account is
    decompressed into the VM's working memory (compact_mem) at the specified
    account_index.

    Accounts expected by this instruction:

    | # | R/W | Type            | Req | PDA | Name             | Description                              |
    |---|-----|-----------------|-----|-----|------------------|------------------------------------------|
    | 0 | mut | Signer          | Yes |     | vm_authority     | The authority of the VM.                 |
    | 1 | mut | Vm              | Yes | PDA | vm               | The VM instance state account.           |
    | 2 | mut | Memory          | Yes | PDA | vm_memory        | The memory account to pull from.         |
    | 3 | mut | Storage         | Yes | PDA | vm_storage       | The storage account to push to.          |
    | 4 |     | UnlockState     |     | PDA | unlock_pda       | State for unlocked timelock accounts.    |
    | 5 |     | WithdrawReceipt |     | PDA | withdraw_receipt | State for withdrawn tokens.              |


    Derived account seeds:

    1. vm:         [ "code_vm", <mint>, <vm_authority>, <lock_duration> ]
    2. vm_memory:  [ "code_vm", "vm_memory_account", <self.name>, <vm> ]
    3. vm_storage: [ "code_vm", "vm_storage_account", <self.name>, <vm> ]
    4. unlock_pda:  [ "code_vm", "vm_unlock_pda_account", <account_owner>, <timelock_address>, <vm> ]

    Instruction data:

    0. account_index: u16   - The index of the account in the VM's paged memory.
    1. signature: [u8; 64]  - A signature of the current account state signed by the VM authority.

    Notes:

    * unlock_pda
        We expect this to be uninitialized in the happy path. If this account
        exists and has a non-locked state, then the decompress instruction will
        fail.

    * withdraw_receipt
        This account is used to prove that a user has not non-custodially
        withdrawn tokens from a virtual account. If this account exists, then
        the decompress instruction will fail.
*/
pub fn process_decompress(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    let args = DecompressIx::try_from_slice(data)?;

    let (
        vm_authority_info,
        vm_info,
        vm_memory_info,
        vm_storage_info,
        unlock_pda_info,       // optional
        withdraw_receipt_info, // optional
    ) = match accounts {
        [a1, a2, a3, a4, a5, a6] => (a1, a2, a3, a4, get_optional(a5), get_optional(a6)),
        _ => return Err(ProgramError::NotEnoughAccountKeys),
    };

    check_signer(vm_authority_info)?;
    check_mut(vm_info)?;
    check_mut(vm_memory_info)?;
    check_mut(vm_storage_info)?;

    let vm = load_vm_checked(vm_info, vm_authority_info)?;

    check_memory(vm_memory_info, vm_info)?;
    check_storage(vm_storage_info, vm_info)?;
    check_is_empty(vm_memory_info, args.account_index)?;

    let unchecked_va = VirtualAccount::unpack(&args.packed_va)?;
    match unchecked_va {
        VirtualAccount::Timelock(vta) => {
            check_condition(
                unlock_pda_info.is_some(),
                "unlock_pda address is required for timelocked virtual accounts",
            )?;

            check_condition(
                withdraw_receipt_info.is_some(),
                "withdraw_receipt address is required for timelocked virtual accounts",
            )?;

            check_timelock_state(
                &vta,
                vm,
                vm_info,
                unlock_pda_info.unwrap(),
                withdraw_receipt_info.unwrap(),
            )?;
        }
        VirtualAccount::Nonce(_) => {
            // Nonce accounts are not timelocked
        }
        VirtualAccount::Relay(_) => {
            // Relay accounts are not timelocked
        }
    }

    let va = unchecked_va;
    let va_hash = va.get_hash();

    sig_verify(
        vm_authority_info.key.as_ref(),
        args.signature.as_ref(),
        va_hash.as_ref(),
    )?;

    let sig_hash = hashv(&[args.signature.as_ref(), va_hash.as_ref()]);
    try_decompress(vm_storage_info, sig_hash, &args.proof)?;
    try_write(vm_memory_info, args.account_index, &va)?;

    vm.advance_poh(CodeInstruction::DecompressIx, accounts, data);

    Ok(())
}

fn check_timelock_state(
    vta: &VirtualTimelockAccount,
    vm: &CodeVmAccount,
    vm_info: &AccountInfo<'_>,
    unlock_pda_info: &AccountInfo<'_>,
    withdraw_receipt_info: &AccountInfo<'_>,
) -> ProgramResult {
    let timelock_address =
        vta.get_timelock_address(&vm.get_mint(), &vm.get_authority(), vm.get_lock_duration());

    let unlock_address = vta.get_unlock_address(&timelock_address, &vm_info.key);

    check_condition(
        unlock_pda_info.key.eq(&unlock_address),
        "unlock_pda does not match the expected unlock address",
    )?;

    let receipt_address = vta.get_withdraw_receipt_address(&unlock_address, &vm_info.key);

    check_condition(
        withdraw_receipt_info.key.eq(&receipt_address),
        "withdraw_receipt does not match the expected receipt address",
    )?;

    // Check that the receipt account is empty (no data; len == 0)
    check_condition(
        withdraw_receipt_info.data_is_empty(),
        "withdraw_receipt is not empty",
    )?;

    // If we have made it this far, then we can assume that the account has not
    // been non-custodially withdrawn from (yet).

    // The account might be unlocked, but we don't really care as long as the
    // withdraw_receipt is empty still.

    Ok(())
}
