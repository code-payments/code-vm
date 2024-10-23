use steel::*;
use borsh::{BorshDeserialize, BorshSerialize};

#[repr(C)]
#[derive(BorshDeserialize, BorshSerialize, Clone, Copy, PartialEq, Debug)]
pub struct VirtualRelayAccount {
    pub target: Pubkey,
    pub destination: Pubkey,
}

impl VirtualRelayAccount {
    pub const LEN: usize = // 64 bytes
        32 + // address
        32;  // destination

    pub fn pack<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        BorshSerialize::serialize(self, &mut writer)
    }

    pub fn unpack(buf: &[u8]) -> std::io::Result<Self> {
        let data = &buf[..VirtualRelayAccount::LEN];
        BorshDeserialize::try_from_slice(data)
    }
}
