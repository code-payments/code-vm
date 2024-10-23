use steel::*;

use crate::consts::*;
use crate::cvm::TokenPool;
use crate::types::{ 
    MerkleTree, 
    CircularBuffer, 
    Hash
};

#[repr(C, align(8))]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct RelayAccount {
    pub vm: Pubkey,
    pub name: [u8; MAX_NAME_LEN],

    pub treasury: TokenPool,
    pub bump: u8,
    pub num_levels: u8,
    pub num_history: u8,

    _padding: [u8; 4],

    pub recent_roots: CircularBuffer<{RELAY_HISTORY_ITEMS}, {Hash::LEN}>,
    pub history: MerkleTree<{RELAY_STATE_DEPTH}>,
}

impl RelayAccount {
    pub const fn get_size() -> usize {
        8 + std::mem::size_of::<Self>()
    }

    pub fn get_recent_root(&self) -> Hash {
        self.recent_roots.first().unwrap().clone().into()
    }

    pub fn save_recent_root(&mut self) {
        let current = self.history.get_root();
        let last = self.recent_roots.last();

        match last {
            None => {
                // There is no last root, proceed to save the current root
            }
            Some(last) => {
                // We have a last root, check if it is the same as the current root
                if current.as_ref().eq(last) {
                    // The root is already saved
                    return;
                }
            },
        };

        self.recent_roots.push(current.as_ref());   
    }

    pub fn add_commitment(&mut self, commitment: &Pubkey) 
        -> ProgramResult {
        self.history.try_insert(commitment.to_bytes().into())
    }

    pub fn unpack(data: &[u8]) -> Self {
        let data = &data[..Self::get_size()];
        Self::try_from_bytes(data).unwrap().clone()
    }

    pub fn unpack_mut(data: &mut [u8]) -> &mut Self {
        let data = &mut data[..Self::get_size()];
        Self::try_from_bytes_mut(data).unwrap()
    }
}