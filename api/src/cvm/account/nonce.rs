use steel::*;
use borsh::{BorshDeserialize, BorshSerialize};
use crate::types::Hash;

#[repr(C)]
#[derive(BorshDeserialize, BorshSerialize, Clone, Copy, PartialEq, Debug)]
pub struct VirtualDurableNonce {
    pub address: Pubkey,    // Unlike a real durable nonce, this value is off-curve and owned by the VM authority
    pub value: Hash,        // The current nonce value (auto-advanced when used)
}

impl VirtualDurableNonce {
    pub const LEN: usize = // 64 bytes
        32 + // address
        32;  // nonce

    pub fn pack<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        BorshSerialize::serialize(self, &mut writer)
    }

    pub fn unpack(buf: &[u8]) -> std::io::Result<Self> {
        let data = &buf[..VirtualDurableNonce::LEN];
        BorshDeserialize::try_from_slice(data)
    }
}
