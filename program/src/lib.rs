mod opcode;
mod instruction;
use instruction::*;

use code_vm_api::prelude::*;
use steel::*;

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let (ix, data) = parse_instruction(&code_vm_api::ID, program_id, data)?;

    match ix {
        CodeInstruction::Unknown => return Err(ProgramError::InvalidInstructionData),

        CodeInstruction::InitVmIx        => process_init_vm(accounts, data)?,
        CodeInstruction::InitMemoryIx    => process_init_memory(accounts, data)?,
        CodeInstruction::InitStorageIx   => process_init_storage(accounts, data)?,
        CodeInstruction::InitRelayIx     => process_init_relay(accounts, data)?,
        CodeInstruction::InitNonceIx     => process_init_nonce(accounts, data)?,
        CodeInstruction::InitTimelockIx  => process_init_timelock(accounts, data)?,
        CodeInstruction::InitUnlockIx    => process_init_unlock(accounts, data)?,

        CodeInstruction::ExecIx          => process_exec(accounts, data)?,
        CodeInstruction::CompressIx      => process_compress(accounts, data)?,
        CodeInstruction::DecompressIx    => process_decompress(accounts, data)?,
        CodeInstruction::ResizeMemoryIx  => process_resize(accounts, data)?,
        CodeInstruction::SnapshotIx      => process_snapshot(accounts, data)?,

        CodeInstruction::DepositIx       => process_deposit(accounts, data)?,
        CodeInstruction::WithdrawIx      => process_withdraw(accounts, data)?,
        CodeInstruction::UnlockIx        => process_unlock(accounts, data)?,
    }

    Ok(())
}

entrypoint!(process_instruction);
