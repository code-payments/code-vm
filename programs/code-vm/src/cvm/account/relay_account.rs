use anchor_lang::prelude::*;
use borsh::{BorshDeserialize, BorshSerialize};
use crate::types::Hash;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Debug)]
#[repr(C)]
pub struct VirtualRelayAccount {
    pub address: Pubkey,
    pub commitment: Hash,
    pub recent_root: Hash,
    pub destination: Pubkey,
}

impl VirtualRelayAccount {
    pub const LEN: usize = // 128 bytes
        32 + // address
        32 + // commitment
        32 + // recent_root
        32;  // destination

    pub fn pack<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        BorshSerialize::serialize(self, &mut writer)
    }

    pub fn unpack(buf: &[u8]) -> std::io::Result<Self> {
        let data = &buf[..VirtualRelayAccount::LEN];
        BorshDeserialize::try_from_slice(data)
    }
}
