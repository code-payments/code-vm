use borsh::{BorshSerialize, BorshDeserialize};
use std::marker::PhantomData;

use steel::*;
use crate::{
    consts::*, types::{Hash, Signature}
};

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum CodeInstruction {
    Unknown = 0,

    InitVmIx,
    InitMemoryIx,
    InitStorageIx,
    InitRelayIx,
    InitNonceIx,
    InitTimelockIx,
    InitUnlockIx,

    ExecIx,
    CompressIx,
    DecompressIx,

    ResizeMemoryIx,
    SnapshotIx,

    DepositIx,
    WithdrawIx,
    UnlockIx,
}

instruction!(CodeInstruction, InitVmIx);
instruction!(CodeInstruction, InitMemoryIx);
instruction!(CodeInstruction, InitStorageIx);
instruction!(CodeInstruction, InitRelayIx);
instruction!(CodeInstruction, InitNonceIx);
instruction!(CodeInstruction, InitTimelockIx);
instruction!(CodeInstruction, InitUnlockIx);

instruction!(CodeInstruction, ExecIx);
instruction!(CodeInstruction, CompressIx);
instruction!(CodeInstruction, DecompressIx);

instruction!(CodeInstruction, ResizeMemoryIx);
instruction!(CodeInstruction, SnapshotIx);

instruction!(CodeInstruction, DepositIx);
instruction!(CodeInstruction, WithdrawIx);
instruction!(CodeInstruction, UnlockIx);

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct InitVmIx {
    pub lock_duration: u8,
    pub vm_bump: u8,
    pub vm_omnibus_bump: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct InitMemoryIx {
    pub name: [u8; MAX_NAME_LEN],
    pub num_accounts: [u8; 4],       // Pack u32 as [u8; 4]
    pub account_size: [u8; 2],       // Pack u16 as [u8; 2]
    pub vm_memory_bump: u8,
}

impl InitMemoryIx {
    /// Converts the byte arrays to their respective data types.
    pub fn to_struct(&self) -> Result<ParsedInitMemoryIx, std::io::Error> {
        Ok(ParsedInitMemoryIx {
            name: self.name,
            num_accounts: u32::from_le_bytes(self.num_accounts),
            account_size: u16::from_le_bytes(self.account_size),
            vm_memory_bump: self.vm_memory_bump,
        })
    }

    /// Creates `InitMemoryIx` from the parsed struct by converting data types back to byte arrays.
    pub fn from_struct(parsed: ParsedInitMemoryIx) -> Self {
        InitMemoryIx {
            name: parsed.name,
            num_accounts: parsed.num_accounts.to_le_bytes(),
            account_size: parsed.account_size.to_le_bytes(),
            vm_memory_bump: parsed.vm_memory_bump,
        }
    }
}

pub struct ParsedInitMemoryIx {
    pub name: [u8; MAX_NAME_LEN],
    pub num_accounts: u32,
    pub account_size: u16,
    pub vm_memory_bump: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ResizeMemoryIx {
    pub account_size: [u8; 4], // Pack u32 as [u8; 4]
}

impl ResizeMemoryIx {
    /// Converts the byte array to u32.
    pub fn to_struct(&self) -> Result<ParsedResizeMemoryIx, std::io::Error> {
        Ok(ParsedResizeMemoryIx {
            account_size: u32::from_le_bytes(self.account_size),
        })
    }

    /// Creates `ResizeMemoryIx` from the parsed struct by converting u32 to byte array.
    pub fn from_struct(parsed: ParsedResizeMemoryIx) -> Self {
        ResizeMemoryIx {
            account_size: parsed.account_size.to_le_bytes(),
        }
    }
}

pub struct ParsedResizeMemoryIx {
    pub account_size: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct InitStorageIx {
    pub name: [u8; MAX_NAME_LEN],
    pub vm_storage_bump: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ExecIx {
    // Dynamically sized data, not supported by Pod (or steel)
    _data: PhantomData<ExecIxData>,
}

impl ExecIx {
    pub fn try_from_slice(data: &[u8]) -> Result<ExecIxData, std::io::Error> {
        ExecIxData::try_from_slice(data)
    }

    pub fn try_to_bytes(args: ExecIxData) -> Result<Vec<u8>, std::io::Error> {
        let discriminator = CodeInstruction::ExecIx as u8;
        let data = args.try_to_vec()?;
        let mut result = Vec::with_capacity(1 + data.len());
        result.push(discriminator);
        result.extend_from_slice(&data);
        Ok(result)
    }
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub struct ExecIxData {
    pub opcode: u8,
    pub mem_indicies: Vec<u16>,
    pub mem_banks: Vec<u8>,
    pub data: Vec<u8>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct InitNonceIx {
    pub account_index: [u8; 2], // Pack u16 as [u8; 2]
}

impl InitNonceIx {
    pub fn to_struct(&self) -> Result<ParsedInitNonceIx, std::io::Error> {
        Ok(ParsedInitNonceIx {
            account_index: u16::from_le_bytes(self.account_index),
        })
    }

    pub fn from_struct(parsed: ParsedInitNonceIx) -> Self {
        InitNonceIx {
            account_index: parsed.account_index.to_le_bytes(),
        }
    }
}

pub struct ParsedInitNonceIx {
    pub account_index: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct InitTimelockIx {
    pub account_index: [u8; 2], // Pack u16 as [u8; 2]
    pub virtual_timelock_bump: u8,
    pub virtual_vault_bump: u8,
    pub unlock_pda_bump: u8,
}

impl InitTimelockIx {
    pub fn to_struct(&self) -> Result<ParsedInitTimelockIx, std::io::Error> {
        Ok(ParsedInitTimelockIx {
            account_index: u16::from_le_bytes(self.account_index),
            virtual_timelock_bump: self.virtual_timelock_bump,
            virtual_vault_bump: self.virtual_vault_bump,
            unlock_pda_bump: self.unlock_pda_bump,
        })
    }

    pub fn from_struct(parsed: ParsedInitTimelockIx) -> Self {
        InitTimelockIx {
            account_index: parsed.account_index.to_le_bytes(),
            virtual_timelock_bump: parsed.virtual_timelock_bump,
            virtual_vault_bump: parsed.virtual_vault_bump,
            unlock_pda_bump: parsed.unlock_pda_bump,
        }
    }
}

pub struct ParsedInitTimelockIx {
    pub account_index: u16,
    pub virtual_timelock_bump: u8,
    pub virtual_vault_bump: u8,
    pub unlock_pda_bump: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct CompressIx {
    pub account_index: [u8; 2], // Pack u16 as [u8; 2]
    pub signature: Signature,
}

impl CompressIx {
    pub fn to_struct(&self) -> Result<ParsedCompressIx, std::io::Error> {
        Ok(ParsedCompressIx {
            account_index: u16::from_le_bytes(self.account_index),
            signature: self.signature,
        })
    }

    pub fn from_struct(parsed: ParsedCompressIx) -> Self {
        CompressIx {
            account_index: parsed.account_index.to_le_bytes(),
            signature: parsed.signature,
        }
    }
}

pub struct ParsedCompressIx {
    pub account_index: u16,
    pub signature: Signature,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct DecompressIx {
    // Dynamically sized data, not supported by Pod (or steel)
    _data: PhantomData<DecompressIxData>,
}

impl DecompressIx {
    pub fn try_from_slice(data: &[u8]) -> Result<DecompressIxData, std::io::Error> {
        DecompressIxData::try_from_slice(data)
    }

    pub fn try_to_bytes(args: DecompressIxData) -> Result<Vec<u8>, std::io::Error> {
        let discriminator = CodeInstruction::DecompressIx as u8;
        let data = args.try_to_vec()?;
        let mut result = Vec::with_capacity(1 + data.len());
        result.push(discriminator);
        result.extend_from_slice(&data);
        Ok(result)
    }
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub struct DecompressIxData {
    pub account_index: u16,
    pub packed_va: Vec<u8>,
    pub proof: Vec<Hash>,
    pub signature: Signature,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct InitRelayIx {
    pub name: [u8; MAX_NAME_LEN],
    pub relay_bump: u8,
    pub relay_vault_bump: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct SnapshotIx { // SaveRecentRoot
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct DepositIx {
    pub account_index: [u8; 2], // Pack u16 as [u8; 2]
    pub amount: [u8; 8],        // Pack u64 as [u8; 8]
    pub bump: u8,
}

impl DepositIx {
    pub fn to_struct(&self) -> Result<ParsedDepositIx, std::io::Error> {
        Ok(ParsedDepositIx {
            account_index: u16::from_le_bytes(self.account_index),
            amount: u64::from_le_bytes(self.amount),
            bump: self.bump,
        })
    }

    pub fn from_struct(parsed: ParsedDepositIx) -> Self {
        DepositIx {
            account_index: parsed.account_index.to_le_bytes(),
            amount: parsed.amount.to_le_bytes(),
            bump: parsed.bump,
        }
    }
}

pub struct ParsedDepositIx {
    pub account_index: u16,
    pub amount: u64,
    pub bump: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct InitUnlockIx {
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct UnlockIx {
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct WithdrawIx {
    _data: PhantomData<WithdrawIxData>,
}

impl WithdrawIx {
    pub fn try_from_slice(data: &[u8]) -> Result<WithdrawIxData, std::io::Error> {
        WithdrawIxData::try_from_slice(data)
    }

    pub fn try_to_bytes(args: WithdrawIxData) -> Result<Vec<u8>, std::io::Error> {
        let discriminator = CodeInstruction::WithdrawIx as u8;
        let data = args.try_to_vec()?;
        let mut result = Vec::with_capacity(1 + data.len());
        result.push(discriminator);
        result.extend_from_slice(&data);
        Ok(result)
    }
}

#[repr(u8)]
#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub enum WithdrawIxData {
    FromMemory {
        account_index: u16,
    } = 0,
    FromStorage {
        packed_va: Vec<u8>,
        proof: Vec<Hash>,
        signature: Signature,
    } = 1,
    FromDeposit {
        bump: u8,
    } = 2,
}