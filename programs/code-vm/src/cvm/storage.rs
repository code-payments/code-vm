use anchor_lang::prelude::*;

use crate::types::MerkleTree;

#[account]
pub struct CompressedStorageAccount {
    pub vm: Pubkey, // The VM that owns this storage
    pub bump: u8,
    pub name: String,

    pub memory_state: MerkleTree,
}

impl CompressedStorageAccount {
    pub const MAX_NAME_LEN: usize = 32;
    
    pub fn max_size_for(levels: u8) -> usize {
        8 +                                 // anchor (discriminator)
        32 +                                // vm
        1 +                                 // bump
        Self::MAX_NAME_LEN +                // name
        MerkleTree::max_size_for(levels)    // merkle_tree (memory_state)
    }
}