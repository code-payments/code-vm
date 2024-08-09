use anchor_lang::prelude::*;

#[error_code]
pub enum CodeVmError {
    #[msg("Lock duration must be greater than 0")]
    InvalidLockDuration,

    #[msg("Invalid memory layout")]
    InvalidMemoryLayout,

    #[msg("Invalid memory name")]
    InvalidMemoryName,

    #[msg("Invalid memory size provided")]
    InvalidMemorySize,

    #[msg("Invalid storage levels provided")]
    InvalidStorageLevels,

    #[msg("Invalid storage name")]
    InvalidStorageName,

    #[msg("Invalid relay levels provided")]
    InvalidRelayLevels,

    #[msg("Invalid relay history size")]
    InvalidRelayHistorySize,

    #[msg("Invalid relay name")]
    InvalidRelayName,

    #[msg("Invalid deposit amount")]
    InvalidDepositAmount,

    #[msg("Invalid memory bank index")]
    InvalidMemoryBank,

    #[msg("Invalid timelock state")]
    InvalidTimelockState,

    #[msg("Invalid virtual account")]
    InvalidVirtualAccount,

    #[msg("Invalid signature")]
    InvalidSignature,

    #[msg("Invalid merkle proof")]
    InvalidMerkleProof,

    #[msg("Invalid merkle tree")]
    MerkleTreeFull,

    #[msg("Virtual account not found")]
    VirtualAccountNotAllocated,

    #[msg("Virtual account already allocated")]
    VirtualAccountAlreadyAllocated,

    #[msg("No empty sector available")]
    MemoryNoEmptySector,

    #[msg("Memory not allocated")]
    MemoryNotAllocated,

    #[msg("Memory already allocated")]
    MemoryAlreadyAllocated,

    #[msg("Memory insufficient size")]
    MemoryInsufficientSize,

    #[msg("Memory insufficient pages")]
    MemoryInsufficientPages,

    #[msg("Memory sector has insufficient pages")]
    MemorySectorInsufficientPages,

    #[msg("Memory sector out of memory")]
    MemorySectorOutOfMemory,
}