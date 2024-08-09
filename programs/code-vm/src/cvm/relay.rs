use anchor_lang::prelude::*;

use crate::types::{ MerkleTree, CircularBuffer, Hash };
use super::TokenPool;

#[account]
pub struct RelayAccount {
    pub vm: Pubkey, // The VM that owns this relay
    pub bump: u8,

    pub name: String,
    pub num_levels: u8,
    pub num_history: u8,

    pub treasury: TokenPool,
    pub history: MerkleTree,
    pub recent_roots: CircularBuffer<{Hash::LEN}>,
}

impl RelayAccount {
    pub const MAX_NAME_LEN: usize = 32;
    pub const MAX_RECENT_HISTORY: u8 = 32;
    pub const MAX_MERKLE_LEVELS: u8 = 64;

    pub fn max_size_for(levels: u8, history: u8) -> usize {
        8 +                  // anchor (discriminator)
        32 +                 // vm
        1 +                  // bump
        Self::MAX_NAME_LEN + // name
        1 +                  // num_levels
        1 +                  // num_history
        TokenPool::LEN +     // treasury
        MerkleTree::max_size_for(levels) +   // state
        CircularBuffer::<{Hash::LEN}>::max_size_for(history) // recent_roots
    }
}
