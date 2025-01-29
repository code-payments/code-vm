use code_vm_api::prelude::*;
use steel::*;

use crate::ExecContext;

/*
    This instruction is used to transfer tokens from one virtual account to
    another virtual account. The signature of the source account is required
    to authorize the transfer.

    Extra accounts required by this instruction:
    
    | # | R/W | Type         | Req | PDA | Name   | Description  |
    |---|-----|------------- |-----|-----|--------|--------------|
    |...| The same as the vm_exec instruction.                   |
    |---|-----|------------- |-----|-----|--------|--------------|
    | 6 |     | <None>       |     |     |        |              |
    | 7 |     | <None>       |     |     |        |              |
    | 8 |     | <None>       |     |     |        |              |
    | 9 |     | <None>       |     |     |        |              |
    |10 |     | <None>       |     |     |        |              |


    Instruction data:

    0. signature: [u8;64]  - The opcode to execute.
    1. amount: [u64]       - The account_indicies of the virtual accounts to use.
*/
pub fn process_transfer(
    ctx: &ExecContext,
    data: &ExecIxData,
) -> ProgramResult {

    let vm = load_vm(ctx.vm_info)?;
    let args = TransferOp::try_from_bytes(&data.data)?.to_struct()?;

    let mem_indicies = &data.mem_indicies;
    let mem_banks = &data.mem_banks;

    check_condition(
        mem_indicies.len() == 3,
        "the number of memory indicies must be 3",
    )?;

    check_condition(
        mem_banks.len() == 3,
        "the number of memory banks must be 3",
    )?;

    let nonce_index = mem_indicies[0];
    let nonce_mem = mem_banks[0];

    let src_index = mem_indicies[1];
    let src_mem = mem_banks[1];

    let dst_index = mem_indicies[2];
    let dst_mem = mem_banks[2];

    let vm_mem = ctx.get_banks();

    check_condition(
        vm_mem[nonce_mem as usize].is_some(),
        "the nonce memory account must be provided",
    )?;

    check_condition(
        vm_mem[src_mem as usize].is_some(),
        "the source memory account must be provided",
    )?;

    check_condition(
        vm_mem[dst_mem as usize].is_some(),
        "the destination memory account must be provided",
    )?;

    let nonce_mem_info = vm_mem[nonce_mem as usize].unwrap();
    let src_mem_info = vm_mem[src_mem as usize].unwrap();
    let dst_mem_info = vm_mem[dst_mem as usize].unwrap();

    let va = try_read(&nonce_mem_info, nonce_index)?;
    let mut vdn = va.into_inner_nonce().unwrap();

    let va = try_read(&src_mem_info, src_index)?;
    let mut src_vta = va.into_inner_timelock().unwrap();

    let va = try_read(&dst_mem_info, dst_index)?;
    let mut dst_vta = va.into_inner_timelock().unwrap();

    let hash = create_transfer_message(
        &vm,
        &src_vta, 
        &dst_vta, 
        &vdn, 
        args.amount
    );

    sig_verify(
        src_vta.owner.as_ref(), 
        args.signature.as_ref(), 
        hash.as_ref(),
    )?;

    if src_vta.balance < args.amount {
        return Err(ProgramError::InsufficientFunds);
    }

    // If the source and destination accounts are the same, then we don't need
    // to do anything.

    let is_same_account = src_mem == dst_mem && src_index == dst_index;
    if !is_same_account {
        src_vta.balance = src_vta.balance
            .checked_sub(args.amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        dst_vta.balance = dst_vta.balance
            .checked_add(args.amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;
    }

    vdn.value = vm.get_current_poh();

    try_write(
        src_mem_info,
        src_index,
        &VirtualAccount::Timelock(src_vta)
    )?;

    try_write(
        dst_mem_info,
        dst_index,
        &VirtualAccount::Timelock(dst_vta)
    )?;

    try_write(
        nonce_mem_info,
        nonce_index,
        &VirtualAccount::Nonce(vdn)
    )?;

    Ok(())
}