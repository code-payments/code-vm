use steel::*;
use std::{cell::{Ref, RefMut}, marker::PhantomData};
use crate::{
    consts::*,
    cvm::{
        VirtualDurableNonce,
        VirtualRelayAccount,
        VirtualTimelockAccount
    },
    types::SliceAllocator
};

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum MemoryVersion {
    Legacy = 0,
    V1 = 1,
}

#[repr(C, packed)] 
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct MemoryAccount {
    pub vm: Pubkey,
    pub name: [u8; MAX_NAME_LEN],
    pub bump: u8,

    pub version: u8,
    pub packed_info: [u8; 6],

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
        (info.get_capacity(), info.get_account_size())
    }

    pub fn get_data<'a>(info: &'a AccountInfo) 
        -> Result<Ref<'a, [u8]>, ProgramError> {

        let data = info.data.borrow();
        let offset = MemoryAccount::get_size();

        // Map the `Ref` to a subslice, preserving the borrow
        let data = Ref::map(data, |d| {
            let (_, data) = d.split_at(offset);
            data
        });

        Ok(data)
    }

    pub fn get_data_mut<'a>(info: &'a AccountInfo) 
        -> Result<RefMut<'a, [u8]>, ProgramError> {

        let data = info.data.borrow_mut();
        let offset = MemoryAccount::get_size();

        // Map the `RefMut` to a subslice, preserving the mutable borrow
        let data = RefMut::map(data, |d| {
            let (_, data) = d.split_at_mut(offset);
            data
        });

        Ok(data)
    }

    pub fn get_version(&self) -> MemoryVersion {
        match self.version {
            0 => MemoryVersion::Legacy,
            1 => MemoryVersion::V1,
            _ => panic!("Invalid memory version"),
        }
    }

    pub fn get_capacity(&self) -> usize {
        match self.get_version() {
            MemoryVersion::Legacy => NUM_ACCOUNTS,
            MemoryVersion::V1 => {
                let packed: &PackedInfoV1 = bytemuck::from_bytes(&self.packed_info);
                packed.num_accounts as usize
            }
        }
    }

    pub fn get_account_size(&self) -> usize {
        match self.get_version() {
            MemoryVersion::Legacy => {
                let packed: &PackedInfoLegacy = bytemuck::from_bytes(&self.packed_info);

                // Values pulled from:
                // https://github.com/code-payments/code-vm/blob/acf276fce3e6858aa70e40dc99c6905f9bd655b9/api/src/cvm/state/memory.rs#L30

                match packed.layout {
                    1 => VirtualTimelockAccount::LEN + 1,
                    2 => VirtualDurableNonce::LEN + 1,
                    3 => VirtualRelayAccount::LEN + 1,
                    _ => panic!("Invalid layout"),
                }
            }
            MemoryVersion::V1 => {
                let packed: &PackedInfoV1 = bytemuck::from_bytes(&self.packed_info);
                packed.account_size as usize
            }
        }
    }

    pub fn set_num_accounts(&mut self, num_accounts: u32) {
        if self.get_version() != MemoryVersion::V1 {
            panic!("Setting num_accounts is only valid for V1 memory version");
        }
        let packed: &mut PackedInfoV1 = bytemuck::from_bytes_mut(&mut self.packed_info);
        packed.num_accounts = num_accounts;
    }

    pub fn set_account_size(&mut self, account_size: u16) {
        if self.get_version() != MemoryVersion::V1 {
            panic!("Setting account_size is only valid for V1 memory version");
        }
        let packed: &mut PackedInfoV1 = bytemuck::from_bytes_mut(&mut self.packed_info);
        packed.account_size = account_size;
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
struct PackedInfoLegacy {
    _padding: [u8; 5],
    pub layout: u8,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
struct PackedInfoV1 {
    pub account_size: u16,
    pub num_accounts: u32,
}