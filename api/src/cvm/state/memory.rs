use steel::*;
use std::marker::PhantomData;
use crate::{
    consts::*, 
    types::SliceAllocator
};

// Using packed instead of align(8) to keep compatibility with older
// versions of the program

#[repr(C, packed)] 
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct MemoryAccount {
    pub vm: Pubkey,
    pub name: [u8; MAX_NAME_LEN],
    pub bump: u8,

    pub version: u8,
    pub num_accounts: u32,
    pub account_size: u16,

    // Data starts at 72 bytes into the account
    _data: PhantomData<[u8]>,
}

impl MemoryAccount {
    pub const fn get_size() -> usize {
        8 + std::mem::size_of::<Self>()
    }

    pub fn get_size_with_data(num_accounts: usize, account_size: usize) -> usize {
        Self::get_size() + SliceAllocator::get_size(num_accounts, account_size)
    }

    pub fn unpack(data: &[u8]) -> Self {
        let data = &data[..Self::get_size()];
        Self::try_from_bytes(data).unwrap().clone()
    }

    pub fn get_capacity_and_size(info: &AccountInfo) -> (usize, usize) {
        let data = info.data.borrow();
        let info = MemoryAccount::unpack(&data);
        (info.num_accounts as usize, info.account_size as usize)
    }

    pub fn get_capacity(&self) -> usize {
        self.num_accounts as usize
    }

    pub fn get_account_size(&self) -> usize {
        self.account_size as usize
    }
}
