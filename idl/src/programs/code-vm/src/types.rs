use anchor_lang::prelude::*;

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Debug)]
pub struct Hash {
   pub(crate) value: [u8; 32]
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Debug)]
pub struct Signature {
   pub(crate) value: [u8; 64]
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Debug)]
pub struct TokenPool {
    pub vault: Pubkey,
    pub vault_bump: u8,
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Debug)]
pub struct CircularBuffer<const N: usize, const M: usize> {
    pub items: [[u8; M]; N],
    pub offset: u8,
    pub num_items: u8,
    _padding: [u8; 6],
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Debug)]
pub struct MerkleTree<const N: usize> {
    pub root: Hash,
    pub filled_subtrees: [Hash; N],
    pub zero_values: [Hash; N],
    pub next_index: u64,
}
