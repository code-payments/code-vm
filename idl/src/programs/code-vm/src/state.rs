use anchor_lang::prelude::*;
use crate::{types::*, MAX_NAME_LEN};

pub type RelayHistory = CircularBuffer<32, 32>;
pub type RelayTree = MerkleTree<64>;
pub type CompressedState = MerkleTree<24>;

#[account]
#[repr(C, align(8))]
#[derive(Copy, Debug, PartialEq)]
pub struct CodeVmAccount {
    pub authority: Pubkey,
    pub mint: Pubkey,
    pub slot: u64,
    pub poh: Hash,
    pub omnibus: TokenPool,
    pub lock_duration: u8,  // in days
    pub bump: u8,

    _padding: [u8; 5],
}


#[account]
#[repr(C, align(8))]
#[derive(Copy, Debug, PartialEq)]
pub struct MemoryAccount {
    pub vm: Pubkey,
    pub name: [u8; MAX_NAME_LEN],
    pub bump: u8,

    pub packed_info: [u8; 8],
    //pub _data: PhantomData<[u8]>,
}

#[account]
#[repr(C, align(8))]
#[derive(Copy, Debug, PartialEq)]
pub struct RelayAccount {
    pub vm: Pubkey,
    pub name: [u8; MAX_NAME_LEN],

    pub treasury: TokenPool,
    pub bump: u8,
    pub num_levels: u8,
    pub num_history: u8,

    _padding: [u8; 4],

    pub recent_roots: RelayHistory,
    pub history: RelayTree,
}

#[account]
#[repr(C, align(8))]
#[derive(Copy, Debug, PartialEq)]
pub struct StorageAccount {
    pub vm: Pubkey,
    pub name: [u8; MAX_NAME_LEN],
    pub bump: u8,
    pub depth: u8,

    _padding: [u8; 6],

    pub compressed_state: CompressedState,
}

#[account]
#[repr(C, align(8))]
#[derive(Copy, Debug, PartialEq)]
pub struct UnlockStateAccount {
    pub vm: Pubkey,
    pub owner: Pubkey,
    pub address: Pubkey,
    pub unlock_at: i64,
    pub bump: u8,
    pub state: u8,

    _padding: [u8; 6],
}

#[account]
#[repr(C, align(8))]
#[derive(Copy, Debug, PartialEq)]
pub struct WithdrawReceiptAccount {
    pub unlock_pda: Pubkey,
    pub nonce: Hash,
    pub amount: u64,
    pub bump: u8,

    _padding: [u8; 7],
}