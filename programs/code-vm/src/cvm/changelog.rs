use anchor_lang::prelude::*;
use borsh::{BorshDeserialize, BorshSerialize};

use crate::types::{Hash, Signature};
use super::{
    VirtualAccount,
    VirtualTimelockAccount,
    IndexedMemory,
    PagedMemory,
};

const CHANGELOG_PAGE_SIZE: usize = 21; // Chosen as the minimum size for a changelog event (divided by 2)
pub type PagedChangeLog = PagedMemory<255, 2, 180, CHANGELOG_PAGE_SIZE>;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Debug)]
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
    pub fn pack<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        BorshSerialize::serialize(self, &mut writer)
    }

    pub fn unpack(buf: &[u8]) -> std::io::Result<Self> {
        BorshDeserialize::try_from_slice(buf)
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Debug)]
#[repr(C)]
pub enum ChangeLogData {
    Create(VirtualAccount), // 144 bytes
    Update(VirtualAccount), // 144 bytes

    Compress { store: Pubkey, account: VirtualAccount, signature: Signature }, // 240 bytes
    Decompress { store: Pubkey, account: VirtualAccount, signature: Signature }, // 240 bytes

    Transfer { src: Pubkey, dst: Pubkey, amount: u64, }, // 80 bytes
    Unlock { owner: Pubkey, address: Pubkey, unlock_pda: Pubkey }, // 104 bytes

    Deposit { account: VirtualTimelockAccount, amount: u64, }, // 104 bytes

    Withdraw { 
        src: Pubkey,
        dst: Pubkey,
        amount: u64, 
        account: Option<VirtualTimelockAccount>,
        signature: Option<Signature>,
    }, // 248 bytes
}

pub struct ChangeLog {}
impl ChangeLog {
    pub fn push(info: &AccountInfo, event: ChangeLogEvent) -> Result<()> {
        let buf = event.try_to_vec().unwrap();

        //msg!("Changelog: {:?}", event);
        //msg!("Changelog length: {:?}", buf.len());

        let data = info.try_borrow_mut_data()?;
        let mut log = PagedChangeLog::into_paged_mem(data);

        // The changelog is a circular buffer, where the current index is the
        // vm.slot value modulo the log.MAX_ITEMS. However, the events are
        // different sizes, so we may need to pop off multiple events to make
        // room for the new event.

        let index = (event.seq % PagedChangeLog::MAX_ITEMS as u64) as i16;
        let mut oldest = (index - 1) % PagedChangeLog::MAX_ITEMS as i16;

        // Pop off the oldest in reverse order until we have room for the new
        // event
        loop {
            if log.has_room_for(buf.len()) {
                break;
            }

            // Ignoring the result of the free because we don't care if the item
            // was successfully freed or not. The only way this fails is if the
            // item was not allocated to begin with.
            let _ = log.try_free_item(oldest as u16);

            // Keep going...
            oldest = (oldest - 1) % PagedChangeLog::MAX_ITEMS as i16;
        }


        log.try_alloc_item(index as u16, buf.len())?;
        log.try_write_item(index as u16, buf.as_ref())?;

        Ok(())
    }

}
