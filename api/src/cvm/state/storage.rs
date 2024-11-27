use steel::*;

use crate::{consts::*, types::MerkleTree};

#[repr(C, align(8))]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct StorageAccount {
    pub vm: Pubkey,
    pub name: [u8; MAX_NAME_LEN],
    pub bump: u8,
    pub depth: u8,

    _padding: [u8; 6],
    pub compressed_state: MerkleTree<{COMPRESSED_STATE_DEPTH}>,
}

impl StorageAccount {
    pub const MERKLE_TREE_DEPTH: usize = COMPRESSED_STATE_DEPTH;
    
    pub const fn get_size() -> usize {
        8 + std::mem::size_of::<Self>()
    }

    pub fn get_compressed_state_mut<'a>(info: &'a AccountInfo) 
        -> Result<&'a mut MerkleTree<{COMPRESSED_STATE_DEPTH}>, ProgramError> {
        let storage = info.to_account_mut::<Self>(&crate::ID)?;
        let compressed_mem = &mut storage.compressed_state;
        Ok(compressed_mem)
    }

    pub fn unpack(data: &[u8]) -> Self {
        let data = &data[..Self::get_size()];
        Self::try_from_bytes(data).unwrap().clone()
    }
}


