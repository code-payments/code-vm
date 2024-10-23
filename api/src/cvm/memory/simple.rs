use steel::*;
use bytemuck::{Pod, Zeroable};

use crate::types::ZeroCopy;
use super::MemoryAllocator;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemState {
    Empty = 0,
    Allocated = 1,
}

#[repr(C, align(8))]
#[derive(Clone, Copy, Debug)]
pub struct SimpleAllocator<const N: usize, const M: usize> {
    pub state: [ItemState; N],
    pub data: [[u8; M]; N],
}

impl <const N: usize, const M: usize> SimpleAllocator<N, M> {
    pub const CAPACITY: usize = N;
    pub const MAX_ITEM_SIZE: usize = M;

    fn read(&self, item_index: usize, offset: usize, size: usize) -> Option<Vec<u8>> {
        if offset + size > M {
            return None;
        }
        Some(self.data[item_index][offset..offset + size].to_vec())
    }

    fn write(&mut self, item_index: usize, offset: usize, data: &[u8]) -> ProgramResult {
        if offset + data.len() > M {
            return Err(ProgramError::InvalidArgument);
        }
        self.data[item_index][offset..offset + data.len()].copy_from_slice(data);
        Ok(())
    }
}

impl <const N: usize, const M: usize> MemoryAllocator for SimpleAllocator<N, M> {
    fn is_empty(&self, index: u16) -> bool {
        self.state[index as usize] == ItemState::Empty
    }

    fn has_item(&self, index: u16) -> bool {
        self.state[index as usize] == ItemState::Allocated
    }

    fn read_item(&self, item_index: u16) -> Option<Vec<u8>> {
        if item_index >= N as u16 || self.is_empty(item_index) {
            return None;
        }

        self.read(item_index as usize, 0, M)
    }

    fn try_alloc_item(&mut self, item_index: u16, data_size: usize) -> ProgramResult {
        if item_index >= N as u16 || data_size > M || self.has_item(item_index) {
            return Err(ProgramError::InvalidArgument);
        }

        self.state[item_index as usize] = ItemState::Allocated;
        self.data[item_index as usize] = [0; M];
        Ok(())
    }

    fn try_free_item(&mut self, item_index: u16) -> ProgramResult {
        if item_index >= N as u16 || self.is_empty(item_index) {
            return Err(ProgramError::InvalidArgument);
        }

        self.state[item_index as usize] = ItemState::Empty;
        self.data[item_index as usize] = [0; M];
        Ok(())
    }

    fn try_write_item(&mut self, item_index: u16, data: &[u8]) -> ProgramResult {
        if item_index >= N as u16 || self.is_empty(item_index) || data.len() > M {
            return Err(ProgramError::InvalidArgument);
        }

        self.write(item_index as usize, 0, data)
    }
}


impl <const N: usize, const M: usize> 
  ZeroCopy for SimpleAllocator<N, M> {}

unsafe impl <const N: usize, const M: usize> 
  Zeroable for SimpleAllocator<N, M> {}

unsafe impl <const N: usize, const M: usize>
  Pod for SimpleAllocator<N, M> {}


#[cfg(test)]
mod tests {
    use std::mem::size_of;
    use super::*;

    const CAPACITY: usize = 10;
    const MAX_ITEM_SIZE: usize = 32;

    type TestMemory = SimpleAllocator<CAPACITY, MAX_ITEM_SIZE>;

    #[test]
    fn test_small_item() {
        let mut buf = vec![0; size_of::<TestMemory>()];
        let mem = TestMemory::load_mut_bytes(&mut buf).unwrap();

        assert!(mem.data.len() == TestMemory::CAPACITY);

        let item_index = 0;
        let item_data = [1, 2, 3, 4, 5];

        assert!(mem.try_alloc_item(item_index, item_data.len()).is_ok());
        assert!(mem.try_write_item(item_index, &item_data).is_ok());

        let read_data = mem.read_item(item_index).unwrap();
        assert_eq!(read_data.as_slice()[..5], item_data);
    }

    #[test]
    fn test_alloc_fail_for_out_of_bounds() {
        let mut buf = vec![0; size_of::<TestMemory>()];
        let mem = TestMemory::load_mut_bytes(&mut buf).unwrap();

        // Attempting to allocate an item beyond the capacity should fail
        assert!(mem.try_alloc_item(CAPACITY as u16, 10).is_err());
    }

    #[test]
    fn test_alloc_fail_for_large_data_size() {
        let mut buf = vec![0; size_of::<TestMemory>()];
        let mem = TestMemory::load_mut_bytes(&mut buf).unwrap();

        // Attempting to allocate an item with data size larger than MAX_ITEM_SIZE should fail
        assert!(mem.try_alloc_item(0, MAX_ITEM_SIZE + 1).is_err());
    }

    #[test]
    fn test_write_fail_to_unallocated_item() {
        let mut buf = vec![0; size_of::<TestMemory>()];
        let mem = TestMemory::load_mut_bytes(&mut buf).unwrap();

        let item_data = [1, 2, 3];

        // Writing to an unallocated item should fail
        assert!(mem.try_write_item(0, &item_data).is_err());
    }

    #[test]
    fn test_free_item() {
        let mut buf = vec![0; size_of::<TestMemory>()];
        let mem = TestMemory::load_mut_bytes(&mut buf).unwrap();

        let item_index = 1;
        let item_data = [10, 20, 30, 40, 50];

        // Allocate and write to the item
        assert!(mem.try_alloc_item(item_index, item_data.len()).is_ok());
        assert!(mem.try_write_item(item_index, &item_data).is_ok());

        // Free the item
        assert!(mem.try_free_item(item_index).is_ok());

        // Reading from a freed item should return None
        assert!(mem.read_item(item_index).is_none());

        // The state should be set to empty
        assert!(mem.is_empty(item_index));
    }

    #[test]
    fn test_realloc_item_after_free() {
        let mut buf = vec![0; size_of::<TestMemory>()];
        let mem = TestMemory::load_mut_bytes(&mut buf).unwrap();

        let item_index = 2;
        let item_data1 = [10, 20, 30];
        let item_data2 = [5, 15, 25, 35];

        // Allocate and write to the item
        assert!(mem.try_alloc_item(item_index, item_data1.len()).is_ok());
        assert!(mem.try_write_item(item_index, &item_data1).is_ok());

        // Free the item
        assert!(mem.try_free_item(item_index).is_ok());

        // Reallocate and write new data
        assert!(mem.try_alloc_item(item_index, item_data2.len()).is_ok());
        assert!(mem.try_write_item(item_index, &item_data2).is_ok());

        // Reading from the item should return the new data
        let read_data = mem.read_item(item_index).unwrap();
        assert_eq!(read_data.as_slice()[..item_data2.len()], item_data2);
    }

    #[test]
    fn test_free_unallocated_item() {
        let mut buf = vec![0; size_of::<TestMemory>()];
        let mem = TestMemory::load_mut_bytes(&mut buf).unwrap();

        let item_index = 0;

        // Attempting to free an unallocated item should fail
        assert!(mem.try_free_item(item_index).is_err());
    }

    #[test]
    fn test_alloc_multiple_items() {
        let mut buf = vec![0; size_of::<TestMemory>()];
        let mem = TestMemory::load_mut_bytes(&mut buf).unwrap();

        let item_data1 = [1, 2, 3];
        let item_data2 = [4, 5, 6, 7];
        let item_data3 = [8, 9, 10, 11, 12];

        // Allocate and write to multiple items
        assert!(mem.try_alloc_item(0, item_data1.len()).is_ok());
        assert!(mem.try_write_item(0, &item_data1).is_ok());

        assert!(mem.try_alloc_item(1, item_data2.len()).is_ok());
        assert!(mem.try_write_item(1, &item_data2).is_ok());

        assert!(mem.try_alloc_item(2, item_data3.len()).is_ok());
        assert!(mem.try_write_item(2, &item_data3).is_ok());

        // Verify data from all items
        let read_data1 = mem.read_item(0).unwrap();
        assert_eq!(read_data1.as_slice()[..item_data1.len()], item_data1);

        let read_data2 = mem.read_item(1).unwrap();
        assert_eq!(read_data2.as_slice()[..item_data2.len()], item_data2);

        let read_data3 = mem.read_item(2).unwrap();
        assert_eq!(read_data3.as_slice()[..item_data3.len()], item_data3);
    }

    #[test]
    fn test_full_capacity_allocations() {
        let mut buf = vec![0; size_of::<TestMemory>()];
        let mem = TestMemory::load_mut_bytes(&mut buf).unwrap();

        // Allocate all items up to capacity
        for i in 0..CAPACITY {
            assert!(mem.try_alloc_item(i as u16, 1).is_ok());
            assert!(mem.try_write_item(i as u16, &[1]).is_ok());
        }

        // Any further allocation should fail
        assert!(mem.try_alloc_item(CAPACITY as u16, 1).is_err());
    }
}