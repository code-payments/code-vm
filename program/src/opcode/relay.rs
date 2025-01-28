use code_vm_api::prelude::*;
use steel::*;

use crate::ExecContext;

/*
    This instruction makes a private payment from a relay to a virtual account.
    Actual tokens move from relay to omnibus.

    Extra accounts required by this instruction:
    
    | # | R/W | Type         | Req | PDA | Name             | Description                                  |
    |---|-----|--------------|-----|-----|------------------|----------------------------------------------|
    |...| The same as the vm_exec instruction.              |                                              |
    |---|-----|--------------|-----|-----|------------------|----------------------------------------------|
    | 6 | mut | TokenAccount |     | PDA | vm_omnibus       | A derived token account owned by the VM.     |
    | 7 | mut | Relay        |     | PDA | relay            | A relay account to use for private transfers.|
    | 8 | mut | TokenAccount |     | PDA | relay_vault      | A derived token account owned by the relay.  |
    | 9 |     | <None>       |     |     |                  |                                              |
    | 10|     | Program      |     |     | token_program    | Required when making token transfers.        |


    Instruction data:

    0. amount: [u64]           - The amount to transfer.
    1. transcript: [u8;32]     - The transcript to verify.
    2. recent_root: [u8;32]    - The recent root to use.
    3. commitment: [u8;32]     - The commitment to use.

*/
pub fn process_relay(
    ctx: &ExecContext,
    data: &ExecIxData,
) -> ProgramResult {

    let vm = load_vm(ctx.vm_info)?;
    let args = RelayOp::try_from_bytes(&data.data)?.to_struct()?;

    check_condition(
        ctx.omnibus_info.is_some(),
        "the omnibus account must be provided",
    )?;

    check_condition(
        ctx.relay_info.is_some(),
        "the relay account must be provided",
    )?;

    check_condition(
        ctx.relay_vault_info.is_some(),
        "the relay_vault account must be provided",
    )?;

    check_condition(
        ctx.token_program_info.is_some(),
        "the token program account must be provided",
    )?;

    let omnibus_info = ctx.omnibus_info.unwrap();
    let relay_info = ctx.relay_info.unwrap();
    let relay_vault_info = ctx.relay_vault_info.unwrap();
    let token_program_info = ctx.token_program_info.unwrap();

    check_mut(omnibus_info)?;
    check_mut(relay_info)?;
    check_mut(relay_vault_info)?;
    check_program(token_program_info, &spl_token::id())?;
    check_omnibus(omnibus_info, ctx.vm_info)?;
    check_relay(relay_info, ctx.vm_info)?;

    let mem_indicies = &data.mem_indicies;
    let mem_banks = &data.mem_banks;

    check_condition(
        mem_indicies.len() == 2,
        "the number of memory indicies must be 2",
    )?;

    check_condition(
        mem_banks.len() == 2,
        "the number of memory banks must be 2",
    )?;

    let dst_index = mem_indicies[0];
    let dst_mem = mem_banks[0];

    let vra_index = mem_indicies[1];
    let vra_mem = mem_banks[1];

    let vm_mem = ctx.get_banks();

    check_condition(
        vm_mem[dst_mem as usize].is_some(),
        "the destination memory account must be provided",
    )?;

    check_condition(
        vm_mem[vra_mem as usize].is_some(),
        "the relay memory account must be provided",
    )?;

    let dst_mem_info = vm_mem[dst_mem as usize].unwrap();
    let vra_mem_info = vm_mem[vra_mem as usize].unwrap();

    // First, lets send the private payment from the relay_vault to the user
    // (thier virtual account)

    let relay = 
        relay_info.to_account_mut::<RelayAccount>(&code_vm_api::ID)?;

    transfer_signed(
        relay_vault_info,
        relay_vault_info,
        omnibus_info,
        token_program_info,
        args.amount,
        &[&[
            CODE_VM, 
            VM_RELAY_VAULT,
            relay_info.key.as_ref(),
            &[relay.treasury.vault_bump],
        ]]
    )?;

    let va = try_read(&dst_mem_info, dst_index)?;
    let mut vta = va.into_inner_timelock().unwrap();

    vta.balance = vta.balance
        .checked_add(args.amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;


    check_is_empty(vra_mem_info, vra_index)?;
    check_condition(
        relay.recent_roots.contains(&args.recent_root.as_ref()),
        "the provided recent_root was not found in the relay recent_root list",
    )?;

    let timelock_address = vta.get_timelock_address(
        &vm.get_mint(), 
        &vm.get_authority(), 
        vm.get_lock_duration(),
    );
    let token_address = vta.get_token_address(&timelock_address);

    let destination_address = token_address;
    let (commitment, _) = find_relay_commitment_address( // <- expensive
        &relay_info.key,
        &args.recent_root,
        &args.transcript, // Contains the "source" but is hashed :)
        &destination_address,
        args.amount,
    );

    check_condition(
        commitment.eq(&args.commitment),
        "the provided commitment does not match the calculated commitment",
    )?;

    // Add the commitment address to the merkle tree
    relay.add_commitment(&commitment)?;

    // Find the virtual relay address
    let (proof_address, _) = find_relay_proof_address( // <- expensive
        &relay_info.key,
        &args.recent_root,
        &args.commitment,
    );
    
    let (vault_address, _) = find_relay_destination( // <- expensive
        &proof_address,
    );

    let vra = VirtualRelayAccount {
        target: vault_address,
        destination: relay.treasury.vault,
    };

    try_write(
        dst_mem_info,
        dst_index,
        &VirtualAccount::Timelock(vta)
    )?;

    try_write(
        vra_mem_info,
        vra_index,
        &VirtualAccount::Relay(vra)
    )?;

    Ok(())
}
