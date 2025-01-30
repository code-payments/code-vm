use code_vm_api::prelude::*;
use steel::*;

use crate::ExecContext;

/*
    This instruction is used to transfer tokens from *one* virtual account to a
    number of virtual accounts. The signature of the source account is required
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
    2. count: [u8]         - The number of destinations.
*/
pub fn process_airdrop(
    ctx: &ExecContext,
    data: &ExecIxData,
) -> ProgramResult {

    let vm = load_vm(ctx.vm_info)?;
    let args = AirdropOp::try_from_bytes(&data.data)?.to_struct()?;

    let mem_indicies = &data.mem_indicies;
    let mem_banks = &data.mem_banks;
    let num_accounts = 2 + (args.count as usize);

    check_condition(
        mem_indicies.len() == num_accounts,
        "invalid number of memory indicies",
    )?;

    check_condition(
        mem_banks.len() == num_accounts,
        "invalid number of memory banks",
    )?;

    let nonce_index = mem_indicies[0];
    let nonce_mem = mem_banks[0];

    let src_index = mem_indicies[1];
    let src_mem = mem_banks[1];

    let vm_mem = ctx.get_banks();

    check_condition(
        vm_mem[nonce_mem as usize].is_some(),
        "the nonce memory account must be provided",
    )?;

    check_condition(
        vm_mem[src_mem as usize].is_some(),
        "the source memory account must be provided",
    )?;

    let nonce_mem_info = vm_mem[nonce_mem as usize].unwrap();
    let src_mem_info = vm_mem[src_mem as usize].unwrap();

    let va = try_read(&nonce_mem_info, nonce_index)?;
    let mut vdn = va.into_inner_nonce().unwrap();

    let va = try_read(&src_mem_info, src_index)?;
    let mut src_vta = va.into_inner_timelock().unwrap();

    let total_amount = args.amount
        .checked_mul(args.count as u64)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    if src_vta.balance < total_amount {
        return Err(ProgramError::InsufficientFunds);
    }

    src_vta.balance = src_vta.balance
        .checked_sub(total_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    let mut dst_pubkeys = Vec::new();
    for i in 0..args.count as usize {
        let dst_index = mem_indicies[2 + i];
        let dst_mem = mem_banks[2 + i];

        check_condition(
            vm_mem[dst_mem as usize].is_some(),
            "a destination memory account must be provided",
        )?;

        let dst_mem_info = vm_mem[dst_mem as usize].unwrap();

        let va = try_read(&dst_mem_info, dst_index)?;
        let mut dst_vta = va.into_inner_timelock().unwrap();

        // Check if this destination is actually the source.
        let is_same_account = (src_mem == dst_mem) && (src_index == dst_index);
        if is_same_account {
            // If the source is also in the destinations list, it receives the airdrop as well.
            src_vta.balance = src_vta.balance
                .checked_add(args.amount)
                .ok_or(ProgramError::ArithmeticOverflow)?;

        } else {
            // Normal destination: add the airdrop to its balance
            dst_vta.balance = dst_vta.balance
                .checked_add(args.amount)
                .ok_or(ProgramError::ArithmeticOverflow)?;

            // Write the updated destination back
            try_write(
                dst_mem_info,
                dst_index,
                &VirtualAccount::Timelock(dst_vta)
            )?;
        }

        dst_pubkeys.push(dst_vta.owner);
    }

    let hash = create_airdrop_message(
        &vm,
        &src_vta,
        &dst_pubkeys,
        args.amount,
        &vdn,
    );

    sig_verify(
        src_vta.owner.as_ref(),
        args.signature.as_ref(),
        hash.as_ref(),
    )?;

    vdn.value = vm.get_current_poh();

    // Finally, write back the updated source (which now includes
    // any airdrop shares if the source was also in the destination list).
    try_write(
        src_mem_info,
        src_index,
        &VirtualAccount::Timelock(src_vta)
    )?;

    try_write(
        nonce_mem_info,
        nonce_index,
        &VirtualAccount::Nonce(vdn)
    )?;

    Ok(())
}