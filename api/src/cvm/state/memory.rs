use steel::*;
use crate::{consts::*, types::SliceAllocator};

#[repr(C, align(8))]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct MemoryAccount {
    pub vm: Pubkey,
    pub name: [u8; MAX_NAME_LEN],
    pub bump: u8,

    _padding1: [u8; 7],

    // The layout can be combined with _data like this (when not using zeroable)
    // https://github.com/code-payments/code-vm/blob/main/idl/src/programs/code-vm/src/state.rs#L33
    
    pub num_accounts: u32,
    pub account_size: u16,

    _padding2: [u8; 2],

    //_data: PhantomData<Vec<u8>>,
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
}
