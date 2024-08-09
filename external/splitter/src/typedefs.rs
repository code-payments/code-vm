use borsh::{BorshDeserialize, BorshSerialize};
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MerkleTree {
    pub levels: u8,
    pub next_index: u64,
    pub root: [u8; 32],
    pub filled_subtrees: Vec<[u8; 32]>,
    pub zero_values: Vec<[u8; 32]>,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DataVersion {
    Unknown,
    Version1,
}
