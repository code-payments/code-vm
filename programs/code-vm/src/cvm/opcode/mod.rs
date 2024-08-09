pub mod timelock;
pub mod relay;

use std::fmt::Display;

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum SystemInstructions {
    Unknown = 0,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum TimelockInstructions {
    TransferToExternal = 10,
    TransferToInternal = 11,
    TransferToRelay = 12,
    WithdrawToExternal = 13,
    WithdrawToInternal = 14,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum SplitterInstructions {
    TransferToExternal = 20,
    TransferToInternal = 21,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum OpCode {
    System(SystemInstructions),
    Timelock(TimelockInstructions),
    Splitter(SplitterInstructions),
}

impl From<SystemInstructions> for OpCode {
    fn from(instr: SystemInstructions) -> Self {
        OpCode::System(instr)
    }
}

impl From<TimelockInstructions> for OpCode {
    fn from(instr: TimelockInstructions) -> Self {
        OpCode::Timelock(instr)
    }
}

impl From<SplitterInstructions> for OpCode {
    fn from(instr: SplitterInstructions) -> Self {
        OpCode::Splitter(instr)
    }
}

impl From<u8> for OpCode {
    fn from(value: u8) -> Self {
        OpCode::from(value)
    }
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            OpCode::System(instr) => write!(f, "System({:?})", instr),
            OpCode::Timelock(instr) => write!(f, "Timelock({:?})", instr),
            OpCode::Splitter(instr) => write!(f, "Splitter({:?})", instr),
        }
    }
}

impl OpCode {
    pub fn to_u8(&self) -> u8 {
        match *self {
            OpCode::System(instr) => instr as u8,
            OpCode::Timelock(instr) => instr as u8,
            OpCode::Splitter(instr) => instr as u8,
        }
    }

    pub fn from(val: u8) -> OpCode {
        match val {
            0 => OpCode::System(SystemInstructions::Unknown),

            10 => OpCode::Timelock(TimelockInstructions::TransferToExternal),
            11 => OpCode::Timelock(TimelockInstructions::TransferToInternal),
            12 => OpCode::Timelock(TimelockInstructions::TransferToRelay),
            13 => OpCode::Timelock(TimelockInstructions::WithdrawToExternal),
            14 => OpCode::Timelock(TimelockInstructions::WithdrawToInternal),

            20 => OpCode::Splitter(SplitterInstructions::TransferToExternal),
            21 => OpCode::Splitter(SplitterInstructions::TransferToInternal),

            _ => panic!("Unknown OpCode: {}", val),
        }
    }
}