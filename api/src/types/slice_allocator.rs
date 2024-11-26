use steel::*;
use std::mem;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum ItemState {
    Free = 0,
    Used = 1,
}

pub struct SliceAllocator<'a> {
    pub state: &'a [u8],
    pub data: &'a [u8],
    pub item_size: usize,
}

impl<'a> SliceAllocator<'a> {

    pub fn try_from_slice(
        slice: &'a [u8],
        capacity: usize,
        max_item_size: usize,
    ) -> Result<Self, ProgramError> {
        // Calculate the size required for the state section
        let state_size = Self::get_state_size(capacity);
        let data_size: usize = Self::get_data_size(capacity, max_item_size);

        println!("capacity: {}, max_item_size: {}", capacity, max_item_size);
        println!("state_size: {}, data_size: {}", state_size, data_size);
        println!("slice.len(): {}", slice.len());

        if slice.len() < state_size + data_size {
            return Err(ProgramError::InvalidArgument);
        }

        // Split the slice into `state` and `data` sections
        let (state_bytes, data_bytes) = slice.split_at(state_size);
        let state = bytemuck::cast_slice(state_bytes) ;

        Ok(Self {
            state,
            data: data_bytes,
            item_size: max_item_size,
        })
    }

    pub fn get_state_size(capacity: usize) -> usize {
        capacity * mem::size_of::<ItemState>()
    }

    pub fn get_data_size(capacity: usize, max_item_size: usize) -> usize {
        capacity * max_item_size
    }

    pub fn get_size(capacity: usize, max_item_size: usize) -> usize {
        Self::get_state_size(capacity) + 
        Self::get_data_size(capacity, max_item_size)
    }

    pub fn capacity(&self) -> usize {
        self.state.len()
    }

    pub fn read(&self, item_index: usize, size: usize) -> Option<&[u8]> {
        if item_index >= self.capacity() || size > self.item_size {
            return None;
        }

        let item_start = item_index * self.item_size;
        Some(&self.data[item_start..item_start + size])
    }

    pub fn is_empty(&self, index: u16) -> bool {
        match ItemState::try_from(self.state[index as usize]) {
            Ok(ItemState::Free) => true,
            _ => false,
        }
    }

    pub fn has_item(&self, index: u16) -> bool {
        match ItemState::try_from(self.state[index as usize]) {
            Ok(ItemState::Used) => true,
            _ => false,
        }
    }

    pub fn read_item(&self, item_index: u16) -> Option<Vec<u8>> {
        if item_index as usize >= self.capacity() || self.is_empty(item_index) {
            return None;
        }

        self.read(item_index as usize, self.item_size)
            .map(|data| data.to_vec())
    }
}

pub struct SliceAllocatorMut<'a> {
    pub state: &'a mut [u8],
    pub data: &'a mut [u8],
    pub item_size: usize,
}

impl<'a> SliceAllocatorMut<'a> {

    pub fn try_from_slice_mut(
        slice: &'a mut [u8],
        capacity: usize,
        max_item_size: usize,
    ) -> Result<Self, ProgramError> {
        // Calculate the size required for the state section
        let state_size = SliceAllocator::get_state_size(capacity);
        let data_size: usize = SliceAllocator::get_data_size(capacity, max_item_size);

        if slice.len() < state_size + data_size {
            return Err(ProgramError::InvalidArgument);
        }

        // Split the slice into `state` and `data` sections
        let (state_bytes, data_bytes) = slice.split_at_mut(state_size);
        let state = bytemuck::cast_slice_mut(state_bytes) ;

        Ok(Self {
            state,
            data: data_bytes,
            item_size: max_item_size,
        })
    }

    pub fn capacity(&self) -> usize {
        self.state.len()
    }

    pub fn read(&self, item_index: usize, size: usize) -> Option<&[u8]> {
        if item_index >= self.capacity() || size > self.item_size {
            return None;
        }

        let item_start = item_index * self.item_size;
        Some(&self.data[item_start..item_start + size])
    }

    pub fn write(&mut self, item_index: usize, data: &[u8]) -> ProgramResult {
        if item_index >= self.capacity() || data.len() > self.item_size {
            return Err(ProgramError::InvalidArgument);
        }

        let item_start = item_index * self.item_size;
        self.data[item_start..item_start + data.len()].copy_from_slice(data);
        Ok(())
    }

    pub fn is_empty(&self, index: u16) -> bool {
        match ItemState::try_from(self.state[index as usize]) {
            Ok(ItemState::Free) => true,
            _ => false,
        }
    }

    pub fn has_item(&self, index: u16) -> bool {
        match ItemState::try_from(self.state[index as usize]) {
            Ok(ItemState::Used) => true,
            _ => false,
        }
    }

    pub fn read_item(&self, item_index: u16) -> Option<Vec<u8>> {
        if item_index as usize >= self.capacity() || self.is_empty(item_index) {
            return None;
        }

        self.read(item_index as usize, self.item_size)
            .map(|data| data.to_vec())
    }

    pub fn try_alloc_item(&mut self, item_index: u16, data_size: usize) -> ProgramResult {
        if item_index as usize >= self.capacity() 
            || data_size > self.item_size 
            || self.has_item(item_index) 
        {
            return Err(ProgramError::InvalidArgument);
        }

        self.state[item_index as usize] = ItemState::Used as u8;
        self.write(item_index as usize, &[0; 0])
    }

    pub fn try_free_item(&mut self, item_index: u16) -> ProgramResult {
        if item_index as usize >= self.capacity() || self.is_empty(item_index) {
            return Err(ProgramError::InvalidArgument);
        }

        self.state[item_index as usize] = ItemState::Free as u8;
        self.write(item_index as usize, &[0; 0])
    }

    pub fn try_write_item(&mut self, item_index: u16, data: &[u8]) -> ProgramResult {
        if item_index as usize >= self.capacity() 
            || self.is_empty(item_index) 
            || data.len() > self.item_size 
        {
            return Err(ProgramError::InvalidArgument);
        }

        self.write(item_index as usize, data)
    }
}


#[cfg(test)]
mod tests {
    use std::mem::size_of;
    use super::*;

    const CAPACITY: usize = 10;
    const MAX_ITEM_SIZE: usize = 32;

    fn get_test_buf() -> Vec<u8> {
        let state_size = CAPACITY * size_of::<ItemState>();
        let data_size = CAPACITY * MAX_ITEM_SIZE;
        let total_size = state_size + data_size;
        vec![0; total_size]
    }

    #[test]
    fn test_small_item() {
        let mut buf = get_test_buf();
        let mut mem = SliceAllocatorMut::try_from_slice_mut(buf.as_mut_slice(), CAPACITY, MAX_ITEM_SIZE).unwrap();

        assert!(mem.capacity() == CAPACITY);

        let item_index = 0;
        let item_data = [1, 2, 3, 4, 5];

        assert!(mem.read_item(item_index).is_none());
        assert!(mem.try_alloc_item(item_index as u16, item_data.len()).is_ok());
        assert!(mem.read_item(item_index).is_some());
        assert!(mem.try_write_item(item_index as u16, &item_data).is_ok());

        let read_data = mem.read_item(item_index).unwrap();
        assert_eq!(read_data.as_slice()[..5], item_data);
    }

    #[test]
    fn test_alloc_fail_for_out_of_bounds() {
        let mut buf = get_test_buf();
        let mut mem = SliceAllocatorMut::try_from_slice_mut(buf.as_mut_slice(), CAPACITY, MAX_ITEM_SIZE).unwrap();

        // Attempting to allocate an item beyond the capacity should fail
        assert!(mem.try_alloc_item(CAPACITY as u16, 10).is_err());
    }

    #[test]
    fn test_alloc_fail_for_large_data_size() {
        let mut buf = get_test_buf();
        let mut mem = SliceAllocatorMut::try_from_slice_mut(buf.as_mut_slice(), CAPACITY, MAX_ITEM_SIZE).unwrap();

        // Attempting to allocate an item with data size larger than MAX_ITEM_SIZE should fail
        assert!(mem.try_alloc_item(0, MAX_ITEM_SIZE + 1).is_err());
    }

    #[test]
    fn test_write_fail_to_unallocated_item() {
        let mut buf = get_test_buf();
        let mut mem = SliceAllocatorMut::try_from_slice_mut(buf.as_mut_slice(), CAPACITY, MAX_ITEM_SIZE).unwrap();

        let item_data = [1, 2, 3];

        // Writing to an unallocated item should fail
        assert!(mem.try_write_item(0, &item_data).is_err());
    }

    #[test]
    fn test_free_item() {
        let mut buf = get_test_buf();
        let mut mem = SliceAllocatorMut::try_from_slice_mut(buf.as_mut_slice(), CAPACITY, MAX_ITEM_SIZE).unwrap();

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
        let mut buf = get_test_buf();
        let mut mem = SliceAllocatorMut::try_from_slice_mut(buf.as_mut_slice(), CAPACITY, MAX_ITEM_SIZE).unwrap();

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

}