use crate::cvm::memory::PagedMemory;
use super::{ VirtualTimelockAccount, VirtualDurableNonce, VirtualRelayAccount};

const MAX_ACCOUNTS: usize = 100; // TODO: set to 65536
const NUM_SECTORS: usize = 2;    // TODO: set to 255
const NUM_PAGES: usize = 255;

// The VM can initialize memory accounts with a certain memory layout. For
// example, a relay account takes up 129 bytes, while a timelock account takes
// up 77 bytes. The maximum amount of indexable pages in a memory account is
// 2^16. A memory account that is optimized for timelock account won't be able
// to store the full 65k relay accounts as it will need to split the account
// into multiple pages.

type AccountMemory<const T: usize> = PagedMemory<MAX_ACCOUNTS, NUM_SECTORS, NUM_PAGES, T>;

const TIMELOCK_PAGE_SIZE: usize = VirtualTimelockAccount::LEN + 1;
const NONCE_PAGE_SIZE: usize = VirtualDurableNonce::LEN + 1;
const RELAY_PAGE_SIZE: usize = VirtualRelayAccount::LEN + 1;
const MIXED_PAGE_SIZE: usize = 32; // bytes

pub type PagedTimelockMemory = AccountMemory<TIMELOCK_PAGE_SIZE>;
pub type PagedNonceMemory = AccountMemory<NONCE_PAGE_SIZE>;
pub type PagedRelayMemory = AccountMemory<RELAY_PAGE_SIZE>;
pub type PagedAccounts = AccountMemory<MIXED_PAGE_SIZE>;

#[repr(C)]
pub enum MemoryLayout {
    Mixed = 0,
    Timelock,
    Nonce,
    Relay,
}

impl Default for MemoryLayout {
    fn default() -> Self {
        MemoryLayout::Mixed
    }
}

impl From<MemoryLayout> for u8 {
    fn from(kind: MemoryLayout) -> u8 {
        match kind {
            MemoryLayout::Mixed => 0,
            MemoryLayout::Timelock => 1,
            MemoryLayout::Nonce => 2,
            MemoryLayout::Relay => 3,
        }
    }
}

impl MemoryLayout {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(MemoryLayout::Mixed),
            1 => Some(MemoryLayout::Timelock),
            2 => Some(MemoryLayout::Nonce),
            3 => Some(MemoryLayout::Relay),
            _ => None,
        }
    }
}