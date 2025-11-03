use code_vm_api::prelude::*;
use steel::*;

/*
    This instruction pulls in a token deposit that was made by a user. In this
    instruction, tokens are moved from the deposit_ata to the vm omnibus and a
    virtual account owned by the depositor is updated.

    Users that wish to timelock/depsit tokens must first find their derived
    deposit PDA. This is exposed by the mobile app. From there, the user can
    send tokens to the associated token address of that deposit PDA using any
    SPL token wallet.

    Once they have done this, we can call this instruction to pull in the
    deposit and update the user's virtual account.

    Accounts expected by this instruction:

    | # | R/W | Type         | PDA | Name          | Description                                   |
    |---|-----|--------------|-----|---------------|-----------------------------------------------|
    | 0 | mut | Signer       |     | vm_authority  | The authority of the VM.                      |
    | 1 | mut | Vm           | PDA | vm            | The VM instance state account.                |
    | 2 | mut | Memory       | PDA | vm_memory     | The memory account to pull from.              |
    | 3 |     | Address      |     | depositor     | The owner of this deposit.                    |
    | 4 |     | Address      | PDA | deposit_pda   | A derived account within the VM address space.|
    | 5 | mut | TokenAccount | PDA | deposit_ata   | A derived token account owned by deposit_pda. |
    | 6 | mut | TokenAccount | PDA | omnibus       | A derived token account owned by vm.          |
    | 7 |     | Program      |     | token_program | The SPL token program.                        |


    Derived account seeds:

    1. vm:          [ "code_vm", <mint>, <vm_authority>, <lock_duration> ]
    2. vm_memory:   [ "code_vm", "vm_memory_account", <self.name>, <vm> ]
    3. deposit_pda: [ "code_vm", "vm_deposit_pda", <depositor>, <vm> ]
    4. deposit_ata: <standard ATA seed>
    5. omnibus:     [ "code_vm", "vm_omnibus", <vm> ]

    Instruction data:

    0. account_index: u16 - The index of the account in the VM's paged memory.
    1. amount: u64        - Amount to deposit
    2. bump: u8           - Deposit PDA bump
*/
pub fn process_deposit_from_pda(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    let args = DepositFromPdaIx::try_from_bytes(data)?.to_struct()?;
    let [
        vm_authority_info,
        vm_info,
        vm_memory_info,
        depositor_info,
        deposit_pda_info,
        deposit_ata_info,
        omnibus_info,
        token_program_info,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    check_signer(vm_authority_info)?;
    check_mut(vm_info)?;
    check_mut(vm_memory_info)?;
    check_mut(deposit_ata_info)?;
    check_mut(omnibus_info)?;
    check_program(token_program_info, &spl_token::id())?;

    let vm = load_vm_checked(vm_info, vm_authority_info)?;

    check_omnibus(omnibus_info, vm_info)?;
    check_memory(vm_memory_info, vm_info)?;

    let va = try_read(&vm_memory_info, args.account_index)?;
    let mut vta = va.into_inner_timelock().unwrap();

    check_condition(
        vta.owner.eq(depositor_info.key),
        "The depositor does not own this account",
    )?;

    transfer_signed(
        deposit_pda_info,
        deposit_ata_info,
        omnibus_info,
        token_program_info,
        args.amount,
        &[&[
            CODE_VM,
            VM_DEPOSIT_PDA,
            &depositor_info.key.to_bytes(),
            &vm_info.key.to_bytes(),
            &[args.bump],
        ]],
    )?;

    vta.balance = vta
        .balance
        .checked_add(args.amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    try_write(
        vm_memory_info,
        args.account_index,
        &VirtualAccount::Timelock(vta),
    )?;

    vm.advance_poh(CodeInstruction::DepositFromPdaIx, accounts, data);

    Ok(())
}

/*
    This instruction deposits tokens from a source user account with coordnation between
    the user and VM authority. In this instruction, tokens are moved from the source_ata
    to the vm omnibus and a virtual account owned by the desired destination is updated.

    Accounts expected by this instruction:

    | # | R/W | Type         | PDA | Name             | Description                                   |
    |---|-----|--------------|-----|------------------|-----------------------------------------------|
    | 0 | mut | Signer       |     | vm_authority     | The authority of the VM.                      |
    | 1 | mut | Vm           | PDA | vm               | The VM instance state account.                |
    | 2 | mut | Memory       | PDA | vm_memory        | The memory account to pull from.              |
    | 3 | mut | Signer       |     | source_authority | The owner of the source of funds to deposit.  |
    | 4 | mut | TokenAccount | PDA | source_ata       | A derived account within the VM address space.|
    | 5 |     | Address      |     | destination      | Destination VTA owner                         |
    | 6 | mut | TokenAccount | PDA | omnibus          | A derived token account owned by vm.          |
    | 7 |     | Program      |     | token_program    | The SPL token program.                        |


    Derived account seeds:

    1. vm:          [ "code_vm", <mint>, <vm_authority>, <lock_duration> ]
    2. vm_memory:   [ "code_vm", "vm_memory_account", <self.name>, <vm> ]
    3. source_ata:  <standard ATA seed>
    4. omnibus:     [ "code_vm", "vm_omnibus", <vm> ]

    Instruction data:

    0. account_index: u16 - The index of the account in the VM's paged memory.
    1. amount: u64        - Amount to deposit
*/
pub fn process_deposit_with_authority(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    let args = DepositWithAuthorityIx::try_from_bytes(data)?.to_struct()?;
    let [
        vm_authority_info,
        vm_info,
        vm_memory_info,
        source_authority_info,
        source_ata_info,
        destination_info,
        omnibus_info,
        token_program_info,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);        
    };

    check_signer(vm_authority_info)?;
    check_signer(source_authority_info)?;
    check_mut(vm_info)?;
    check_mut(vm_memory_info)?;
    check_mut(source_ata_info)?;
    check_mut(omnibus_info)?;
    check_program(token_program_info, &spl_token::id())?;

    let vm = load_vm_checked(vm_info, vm_authority_info)?;

    check_omnibus(omnibus_info, vm_info)?;
    check_memory(vm_memory_info, vm_info)?;

    let va = try_read(&vm_memory_info, args.account_index)?;
    let mut vta = va.into_inner_timelock().unwrap();

    check_condition(
        vta.owner.eq(destination_info.key),
        "The destination does not match the vta owner",
    )?;

    transfer(
        source_authority_info,
        source_ata_info,
        omnibus_info,
        token_program_info,
        args.amount,
    )?;

    vta.balance = vta
        .balance
        .checked_add(args.amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    try_write(
        vm_memory_info,
        args.account_index,
        &VirtualAccount::Timelock(vta),
    )?;

    vm.advance_poh(CodeInstruction::DepositWithAuthorityIx, accounts, data);

    Ok(())
}
