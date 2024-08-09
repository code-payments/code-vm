use std::mem::size_of;
use std::cell::RefMut;
use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};
use borsh::{BorshDeserialize, BorshSerialize};

use crate::{
    utils,
    types::{ Hash, ZeroCopy }
};

use super::{
    IndexedMemory, 
    MemoryLayout,
    PagedNonceMemory,
    PagedAccounts,
    PagedChangeLog,
    PagedRelayMemory,
    PagedTimelockMemory,
    TokenPool,
};

#[repr(C)]
#[account]
#[derive(Debug, Copy)]
pub struct CodeVmAccount {
    pub authority: Pubkey,
    pub mint: Pubkey,
    pub omnibus: TokenPool,
    pub lock_duration: u8,  // in days
    pub bump: u8,
    pub slot: u64,
    pub poh: Hash,
    _padding: [u8; 5], // bytemuck::cast_slice(...) padding

    // Not including the changelog here because it's too big for the automatic
    // anchor deserialize. Use CodeVmAccountWithChangeLog to access it.
}

impl ZeroCopy for CodeVmAccount { }
unsafe impl Zeroable for CodeVmAccount {}
unsafe impl Pod for CodeVmAccount {}

#[repr(C)]
#[account]
#[derive(Debug, Copy, Pod, Zeroable)]
pub struct MemoryAccount {
    pub vm: Pubkey, // The VM that owns this memory
    pub bump: u8,
    pub name: [u8; MemoryAccount::MAX_NAME_LEN],
    pub layout: u8,

    // Not including the data here because it's too big for the automatic
    // anchor deserialize. Use MemoryAccountWithData to access it.
}

impl MemoryAccount {
    pub const MAX_NAME_LEN: usize = 32;
    pub const LEN: usize =            // 74 bytes
        8 +                           // anchor (discriminator)
        32 +                          // vm
        1 +                           // bump
        1 +                           // padding
        MemoryAccount::MAX_NAME_LEN;  // name
    
    pub fn get_address(&self) -> Pubkey {
        utils::create_memory_address(
            self.vm,
            &self.name,
            self.bump
        )
    }

    pub fn pack<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        BorshSerialize::serialize(self, &mut writer)
    }

    pub fn unpack(buf: &[u8]) -> std::io::Result<Self> {
        // first 8 bytes are the discriminator
        let data = &buf[8..MemoryAccount::LEN];
        BorshDeserialize::try_from_slice(data)
    }
}

#[repr(C)]
#[account(zero_copy)]
#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct MemoryAccountWithData {
    _discriminator: [u8; 8],
    pub info: MemoryAccount,
    pub data: [u8; size_of::<PagedAccounts>()],
}

impl MemoryAccountWithData {
    pub fn into_indexed_memory<'a>(
        data: RefMut<'a, &mut [u8]>
    ) -> RefMut<'a, dyn IndexedMemory>  {
        const OFFSET: usize = 8 + size_of::<MemoryAccount>();
        let account = MemoryAccount::unpack(data.as_ref()).unwrap();
        match MemoryLayout::from_u8(account.layout) {
            Some(MemoryLayout::Timelock) => {
                const UNTIL: usize = OFFSET + size_of::<PagedTimelockMemory>();
                RefMut::map(data, |data: &mut &mut [u8]| -> &mut PagedTimelockMemory {
                    bytemuck::from_bytes_mut(&mut data[OFFSET..UNTIL])
                })
            },
            Some(MemoryLayout::Nonce) => {
                const UNTIL: usize = OFFSET + size_of::<PagedNonceMemory>();
                RefMut::map(data, |data: &mut &mut [u8]| -> &mut PagedNonceMemory {
                    bytemuck::from_bytes_mut(&mut data[OFFSET..UNTIL])
                })
            },
            Some(MemoryLayout::Relay) => {
                const UNTIL: usize = OFFSET + size_of::<PagedRelayMemory>();
                RefMut::map(data, |data: &mut &mut [u8]| -> &mut PagedRelayMemory {
                    bytemuck::from_bytes_mut(&mut data[OFFSET..UNTIL])
                })
            },
            _ => {
                const UNTIL: usize = OFFSET + size_of::<PagedAccounts>();
                RefMut::map(data, |data: &mut &mut [u8]| -> &mut PagedAccounts {
                    bytemuck::from_bytes_mut(&mut data[OFFSET..UNTIL])
                })
            }
        }
    }
}


#[repr(C)]
#[account(zero_copy)]
#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct CodeVmAccountWithChangeLog {
    _discriminator: [u8; 8],
    pub info: CodeVmAccount,
    pub changelog: [u8; size_of::<PagedChangeLog>()],
    _padding: [u8; 2],
}

impl PagedChangeLog {
    pub fn into_paged_mem<'a>(
        data: RefMut<'a, &mut [u8]>
    ) -> RefMut<'a, PagedChangeLog> {
        const OFFSET: usize = size_of::<CodeVmAccount>(); // no discriminator offset
        const UNTIL: usize = OFFSET + size_of::<PagedChangeLog>();
        RefMut::map(data, |data: &mut &mut [u8]| -> &mut PagedChangeLog {
            bytemuck::from_bytes_mut(&mut data[OFFSET..UNTIL])
        })
    }
}
