use anchor_lang::prelude::*;

use crate::{
    cvm::{
        self, 
        OpCode, 
        SplitterInstructions, 
        TimelockInstructions,
    },
    instructions::CodeVmExec,
};

pub fn exec_opcode(
    ctx: Context<CodeVmExec>, 
    opcode: u8,
    mem_indicies: Vec<u16>, 
    mem_banks: Vec<u8>, 
    data: Vec<u8>,
) -> Result<()> {

    let opcode = OpCode::from(opcode);
    msg!("Executing OpCode: {}", opcode);

    match opcode {

        OpCode::Timelock(TimelockInstructions::TransferToExternal) => {
            cvm::opcode::timelock::transfer_to_external(
                ctx, 
                mem_indicies, 
                mem_banks, 
                data,
            )
        },
        OpCode::Timelock(TimelockInstructions::TransferToInternal) => {
            cvm::opcode::timelock::transfer_to_internal(
                ctx, 
                mem_indicies, 
                mem_banks, 
                data,
            )
        },
        OpCode::Timelock(TimelockInstructions::TransferToRelay) => {
            cvm::opcode::timelock::transfer_to_relay(
                ctx, 
                mem_indicies, 
                mem_banks, 
                data,
            )
        },
        OpCode::Timelock(TimelockInstructions::WithdrawToExternal) => {
            cvm::opcode::timelock::withdraw_to_external(
                ctx, 
                mem_indicies, 
                mem_banks, 
                data,
            )
        },
        OpCode::Timelock(TimelockInstructions::WithdrawToInternal) => {
            cvm::opcode::timelock::withdraw_to_internal(
                ctx, 
                mem_indicies, 
                mem_banks, 
                data,
            )
        },

        OpCode::Splitter(SplitterInstructions::TransferToExternal) => {
            cvm::opcode::relay::transfer_to_external(
                ctx, 
                mem_indicies, 
                mem_banks, 
                data,
            )
        },
        OpCode::Splitter(SplitterInstructions::TransferToInternal) => {
            cvm::opcode::relay::transfer_to_internal(
                ctx, 
                mem_indicies, 
                mem_banks, 
                data,
            )
        },

        _ => {
            panic!("Unknown OpCode: {}", opcode.to_u8())
        }
    }
}
