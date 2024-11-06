use steel::*;
use std::cell::RefMut;
use std::marker::PhantomData;
use crate::consts::*;
use crate::cvm::{
    SimpleAllocator,
    MemoryAllocator,
    VirtualDurableNonce,
    VirtualRelayAccount,
    VirtualTimelockAccount
};

const TIMELOCK_SIZE: usize   = VirtualTimelockAccount::LEN + 1;
const NONCE_SIZE: usize      = VirtualDurableNonce::LEN + 1;
const RELAY_SIZE: usize      = VirtualRelayAccount::LEN + 1;

pub type TimelockMemory = SimpleAllocator<COMPACT_STATE_ITEMS, TIMELOCK_SIZE>;
pub type NonceMemory    = SimpleAllocator<COMPACT_STATE_ITEMS, NONCE_SIZE>;
pub type RelayMemory    = SimpleAllocator<COMPACT_STATE_ITEMS, RELAY_SIZE>;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum MemoryLayout {
    Unknown = 0,
    Timelock,
    Nonce,
    Relay,
}

impl MemoryLayout {
    pub fn get_size(&self) -> usize {
        match self {
            MemoryLayout::Timelock => std::mem::size_of::<TimelockMemory>(),
            MemoryLayout::Nonce => std::mem::size_of::<NonceMemory>(),
            MemoryLayout::Relay => std::mem::size_of::<RelayMemory>(),
            _ => panic!("Invalid layout"),
        }
    }
}

#[repr(C, align(8))]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct MemoryAccount {
    pub vm: Pubkey,
    pub name: [u8; MAX_NAME_LEN],
    pub bump: u8,

    _padding: [u8; 6],

    pub layout: u8,
    _data: PhantomData<dyn MemoryAllocator>,
}

impl MemoryAccount {
    pub const fn get_size() -> usize {
        8 + std::mem::size_of::<Self>()
    }

    pub fn get_size_with_data(layout: MemoryLayout) -> usize {
        Self::get_size() + layout.get_size()
    }

    pub fn unpack(data: &[u8]) -> Self {
        let data = &data[..Self::get_size()];
        Self::try_from_bytes(data).unwrap().clone()
    }

    pub fn unpack_mut(data: &mut [u8]) -> &mut Self {
        let data = &mut data[..Self::get_size()];
        Self::try_from_bytes_mut(data).unwrap()
    }

    pub fn get_layout<'a>(info: &'a AccountInfo) -> MemoryLayout {
        let data = info.try_borrow_data().unwrap();
        let memory = MemoryAccount::unpack(&data);
        MemoryLayout::try_from(memory.layout).unwrap()
    }

    pub fn get_indexed_memory_mut<'a>(info: &'a AccountInfo) 
        -> Result<RefMut<'a, dyn MemoryAllocator>, ProgramError> {
        let data = info.try_borrow_mut_data()?;
        Ok(Self::into_indexed_memory_mut(data))
    }

    pub fn into_indexed_memory_mut<'a>(
        data: RefMut<'a, &mut [u8]>
    ) -> RefMut<'a, dyn MemoryAllocator> {

        let memory = MemoryAccount::unpack(&data);
        let layout = MemoryLayout::try_from(memory.layout).unwrap();
        let offset: usize = MemoryAccount::get_size();
        let until = Self::get_size_with_data(layout);

        match layout {
            MemoryLayout::Timelock => {
                RefMut::map(data, |data: &mut &mut [u8]| -> &mut TimelockMemory {
                    bytemuck::from_bytes_mut(&mut data[offset..until])
                })
            },
            MemoryLayout::Nonce => {
                RefMut::map(data, |data: &mut &mut [u8]| -> &mut NonceMemory {
                    bytemuck::from_bytes_mut(&mut data[offset..until])
                })
            },
            MemoryLayout::Relay => {
                RefMut::map(data, |data: &mut &mut [u8]| -> &mut RelayMemory {
                    bytemuck::from_bytes_mut(&mut data[offset..until])
                })
            },
            _ => panic!("Invalid layout"),
        }
    }

    pub fn into_indexed_memory<'a>(
        data: &'a [u8]
    ) -> &'a dyn MemoryAllocator {

        let memory = MemoryAccount::unpack(&data);
        let layout = MemoryLayout::try_from(memory.layout).unwrap();
        let offset: usize = MemoryAccount::get_size();
        let until = Self::get_size_with_data(layout);

        match layout {
            MemoryLayout::Timelock => {
                bytemuck::from_bytes(&data[offset..until]) as &TimelockMemory
            },
            MemoryLayout::Nonce => {
                bytemuck::from_bytes(&data[offset..until]) as &NonceMemory
            },
            MemoryLayout::Relay => {
                bytemuck::from_bytes(&data[offset..until]) as &RelayMemory
            },
            _ => panic!("Invalid layout"),
        }
    }
}