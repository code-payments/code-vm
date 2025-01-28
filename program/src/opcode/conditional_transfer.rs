use code_vm_api::prelude::*;
use steel::*;

use crate::ExecContext;

/*
    This instruction is used to make a conditional transfer. The transfer can
    only happen if another transfer was done prior (either using RelayOp or
    ExternalRelayOp). The signature of the source account is required to
    authorize the transfer.

    The existance of the virtual relay account is proof that the action was done
    for a particular commitment value.

    Extra accounts required by this instruction:

    | # | R/W | Type         | Req | PDA | Name             | Description                                  |
    |---|-----|------------- |-----|-----|------------------|----------------------------------------------|
    |...| The same as the vm_exec instruction.                                                             |
    |---|-----|------------- |-----|-----|------------------|----------------------------------------------|
    | 6 | mut | TokenAccount | Yes | PDA | vm_omnibus       | A derived token account owned by the VM.     |
    | 7 |     | <None>       |     |     |                  |                                              |
    | 8 |     | <None>       |     |     |                  |                                              |
    | 9 | mut | TokenAccount |     |     | external_address | Required when making external transfers.     |
    | 10|     | Program      | Yes |     | token_program    | Required when making token transfers.        |

    Instruction data:

    0. signature: [u8;64]  - The opcode to execute.
    1. amount: [u64]       - The account_indicies of the virtual accounts to use.
*/
pub fn process_conditional_transfer(ctx: &ExecContext, data: &ExecIxData) -> ProgramResult {
    let vm = load_vm(ctx.vm_info)?;
    let args = ConditionalTransferOp::try_from_bytes(&data.data)?.to_struct()?;

    check_condition(
        ctx.omnibus_info.is_some(),
        "the omnibus account must be provided",
    )?;

    check_condition(
        ctx.external_address_info.is_some(),
        "the external address account must be provided",
    )?;

    check_condition(
        ctx.token_program_info.is_some(),
        "the token program account must be provided",
    )?;

    let omnibus_info = ctx.omnibus_info.unwrap();
    let external_address_info = ctx.external_address_info.unwrap();
    let token_program_info = ctx.token_program_info.unwrap();

    check_mut(omnibus_info)?;
    check_mut(external_address_info)?;
    check_program(token_program_info, &spl_token::id())?;
    check_omnibus(omnibus_info, ctx.vm_info)?;

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

    let vra_index = mem_indicies[2];
    let vra_mem = mem_banks[2];

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
        vm_mem[vra_mem as usize].is_some(),
        "the relay memory account must be provided",
    )?;

    let nonce_mem_info = vm_mem[nonce_mem as usize].unwrap();
    let src_mem_info = vm_mem[src_mem as usize].unwrap();
    let vra_mem_info = vm_mem[vra_mem as usize].unwrap();

    let va = try_read(&nonce_mem_info, nonce_index)?;
    let mut vdn = va.into_inner_nonce().unwrap();

    let va = try_read(&src_mem_info, src_index)?;
    let mut src_vta = va.into_inner_timelock().unwrap();

    let va = try_read(&vra_mem_info, vra_index)?;
    let vra = va.into_inner_relay().unwrap();

    check_condition(
        vra.destination.eq(external_address_info.key),
        "the virtual relay destination must match the external address",
    )?;

    let hash = create_transfer_message_to_external(
        &vm,
        &src_vta, 
        &vra.target, 
        &vdn, 
        args.amount
    );

    sig_verify(
        src_vta.owner.as_ref(),
        args.signature.as_ref(),
        hash.as_ref(),
    )?;

    transfer_signed(
        omnibus_info,
        omnibus_info,
        external_address_info,
        token_program_info,
        args.amount,
        &[&[
            CODE_VM,
            VM_OMNIBUS,
            ctx.vm_info.key.as_ref(),
            &[vm.get_omnibus_bump()],
        ]],
    )?;

    src_vta.balance = src_vta
        .balance
        .checked_sub(args.amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    vdn.value = vm.get_current_poh();

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
