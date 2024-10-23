use steel::*;
use borsh::{BorshDeserialize, BorshSerialize};
use crate::types::{CircularBuffer, Hash, Signature};
use super::VirtualAccount;

const CAPACITY : usize = 32;
const MAX_ITEM_SIZE : usize = std::mem::size_of::<ChangeLogEvent>();

pub type ChangeLog = CircularBuffer<CAPACITY, MAX_ITEM_SIZE>;

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, PartialEq, Debug)]
pub struct ChangeLogEvent {
    /// Proof of history hash for this event
    pub id: Hash,

    /// Index corresponding to the number of successful operations in this vm.
    /// Used by the off-chain indexer to figure out when there are gaps to be backfilled.
    pub seq: u64,

    /// The data for this event (if any); not meant to be comprehensive. Only account
    /// changes are recorded here, the rest is captured as part of the PoH hash.
    pub data: Option<ChangeLogData>,
}

impl ChangeLogEvent {
    pub fn to_bytes(&self) -> Vec<u8> {
        BorshSerialize::try_to_vec(self).unwrap()
    }

    pub fn pack<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        BorshSerialize::serialize(self, &mut writer)
    }

    pub fn unpack(buf: &[u8]) -> std::io::Result<Self> {
        BorshDeserialize::try_from_slice(buf)
    }
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, PartialEq, Debug)]
pub enum ChangeLogData {
    Create(VirtualAccount),

    Compress { 
        storage: Pubkey, 
        account: VirtualAccount, 
        signature: Signature 
    },

    Decompress { 
        storage: Pubkey, 
        account: VirtualAccount, 
        signature: Signature
    },

    Transfer { src: VirtualAccount, dst: Pubkey, amount: u64, },
    Withdraw { src: VirtualAccount, dst: Pubkey },

    TimelockUnlock { address: Pubkey, owner: Pubkey, },
    TimelockDeposit { account: VirtualAccount, amount: u64, },
    TimelockWithdraw { owner: Pubkey, dst: Pubkey, amount: u64, },
}
