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
pub struct RelayHistory {
    pub items: [Hash; 32],
    pub offset: u8,
    pub num_items: u8,
    _padding: [u8; 6],
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Debug)]
pub struct RelayTree {
    pub root: Hash,
    pub filled_subtrees: [Hash; 63],
    pub zero_values: [Hash; 63],
    pub next_index: u64,
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Debug)]
pub struct CompressedState {
    pub root: Hash,
    pub filled_subtrees: [Hash; 20],
    pub zero_values: [Hash; 20],
    pub next_index: u64,
}
