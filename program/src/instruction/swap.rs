use code_vm_api::prelude::*;
use steel::*;

// todo: comments
pub fn process_transfer_for_swap(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    let args = TransferForSwapIx::try_from_bytes(data)?.to_struct()?;
    let [
        vm_authority_info,
        vm_info,
        swapper_info,
        swap_pda_info,
        swap_ata_info,
        destination_info,
        token_program_info,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    check_signer(vm_authority_info)?;
    check_signer(swapper_info)?;
    check_mut(vm_info)?;
    check_mut(swap_ata_info)?;
    check_mut(destination_info)?;
    check_program(token_program_info, &spl_token::id())?;

    let vm = load_vm_checked(vm_info, vm_authority_info)?;

    transfer_signed(
        swap_pda_info,
        swap_ata_info,
        destination_info,
        token_program_info,
        args.amount,
        &[&[
            CODE_VM,
            VM_SWAP_PDA,
            &swapper_info.key.to_bytes(),
            &vm_info.key.to_bytes(),
            &[args.bump],
        ]],
    )?;

    vm.advance_poh(CodeInstruction::TransferForSwapIx, accounts, data);

    Ok(())
}

// todo: comments
pub fn process_cancel_swap(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    let args = CancelSwapIx::try_from_bytes(data)?.to_struct()?;
    let [
        vm_authority_info,
        vm_info,
        vm_memory_info,
        swapper_info,
        swap_pda_info,
        swap_ata_info,
        omnibus_info,
        token_program_info,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    check_signer(vm_authority_info)?;
    check_mut(vm_info)?;
    check_mut(vm_memory_info)?;
    check_mut(swap_ata_info)?;
    check_mut(omnibus_info)?;
    check_program(token_program_info, &spl_token::id())?;

    let vm = load_vm_checked(vm_info, vm_authority_info)?;

    check_omnibus(omnibus_info, vm_info)?;
    check_memory(vm_memory_info, vm_info)?;

    let va = try_read(&vm_memory_info, args.account_index)?;
    let mut vta = va.into_inner_timelock().unwrap();

    check_condition(
        vta.owner.eq(swapper_info.key),
        "The swapper does not own this account",
    )?;

    transfer_signed(
        swap_pda_info,
        swap_ata_info,
        omnibus_info,
        token_program_info,
        args.amount,
        &[&[
            CODE_VM,
            VM_SWAP_PDA,
            &swapper_info.key.to_bytes(),
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

    vm.advance_poh(CodeInstruction::CancelSwapIx, accounts, data);

    Ok(())
}

// todo: comments
pub fn process_close_swap_account_if_empty(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    let args = CloseSwapAccountIfEmptyIx::try_from_bytes(data)?.to_struct()?;
    let [
        vm_authority_info,
        vm_info,
        swapper_info,
        swap_pda_info,
        swap_ata_info,
        destination_info,
        token_program_info,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    check_signer(vm_authority_info)?;
    check_mut(vm_info)?;
    check_mut(swap_ata_info)?;

    let vm = load_vm_checked(vm_info, vm_authority_info)?;

    let token_account = swap_ata_info.to_token_account()?;

    if token_account.amount > 0 {
        return Ok(())
    }

    solana_program::program::invoke_signed(
            &spl_token::instruction::close_account(
            token_program_info.key,
            swap_ata_info.key,
            destination_info.key,
            swap_pda_info.key,
            &[swap_pda_info.key],
        )?,
        &[
            token_program_info.clone(),
            swap_ata_info.clone(),
            destination_info.clone(),
            swap_pda_info.clone(),
        ],
        &[&[
            CODE_VM,
            VM_SWAP_PDA,
            &swapper_info.key.to_bytes(),
            &vm_info.key.to_bytes(),
            &[args.bump],
        ]],
    )?;


    vm.advance_poh(CodeInstruction::CloseSwapAccountIfEmptyIx, accounts, data);

    Ok(())
}
