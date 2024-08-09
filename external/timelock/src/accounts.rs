use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use crate::*;
pub const TIME_LOCK_ACCOUNT_ACCOUNT_DISCM: [u8; 8] = [
    112,
    63,
    106,
    231,
    182,
    101,
    88,
    158,
];
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimeLockAccount {
    pub data_version: DataVersion,
    pub time_authority: Pubkey,
    pub close_authority: Pubkey,
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub vault_bump: u8,
    pub vault_state: TimeLockState,
    pub vault_owner: Pubkey,
    pub unlock_at: Option<i64>,
    pub num_days_locked: u8,
}
#[derive(Clone, Debug, PartialEq)]
pub struct TimeLockAccountAccount(pub TimeLockAccount);
impl TimeLockAccountAccount {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        use std::io::Read;
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != TIME_LOCK_ACCOUNT_ACCOUNT_DISCM {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "discm does not match. Expected: {:?}. Received: {:?}",
                        TIME_LOCK_ACCOUNT_ACCOUNT_DISCM, maybe_discm
                    ),
                ),
            );
        }
        Ok(Self(TimeLockAccount::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&TIME_LOCK_ACCOUNT_ACCOUNT_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
