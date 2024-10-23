use steel::*;
use std::fmt;
use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{ Pod, Zeroable };

pub const HASH_BYTES: usize = 32;

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, PartialEq, Default, Pod, Zeroable)]
pub struct Hash {
   pub(crate) value: [u8; 32] // Using an explicit "value" here to avoid IDL generation issues
}

impl From<Hash> for Pubkey {
    fn from(from: Hash) -> Self {
        Pubkey::from(from.value)
    }
}

impl From<Hash> for [u8; HASH_BYTES] {
    fn from(from: Hash) -> Self {
        from.value
    }
}

impl From<[u8; HASH_BYTES]> for Hash {
    fn from(from: [u8; 32]) -> Self {
        Self { value: from }
    }
}

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        &self.value
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", bs58::encode(self.value).into_string())
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", bs58::encode(self.value).into_string())
    }
}

impl Hash {
    pub const LEN: usize = HASH_BYTES;

    pub fn new(hash_slice: &[u8]) -> Self {
        Hash { value: <[u8; HASH_BYTES]>::try_from(hash_slice).unwrap() }
    }

    pub const fn new_from_array(hash_array: [u8; HASH_BYTES]) -> Self {
        Self { value: hash_array }
    }

    pub fn to_bytes(self) -> [u8; HASH_BYTES] {
        self.value
    }
}
