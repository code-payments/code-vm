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
        let state_size = Self::get_state_size(capacity);
        let data_size: usize = Self::get_data_size(capacity, max_item_size);

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
    use super::*;

    #[test]
    fn test_slice_allocator_creation() {
        let capacity = 4;
        let item_size = 8;
        let total_size = SliceAllocator::get_size(capacity, item_size);
        let buffer = vec![0u8; total_size];

        let allocator = SliceAllocator::try_from_slice(&buffer, capacity, item_size);
        assert!(allocator.is_ok());
        let allocator = allocator.unwrap();

        assert_eq!(allocator.capacity(), capacity);
    }

    #[test]
    fn test_slice_allocator_mut_creation() {
        let capacity = 4;
        let item_size = 8;
        let total_size = SliceAllocator::get_size(capacity, item_size);
        let mut buffer = vec![0u8; total_size];

        let allocator = SliceAllocatorMut::try_from_slice_mut(&mut buffer, capacity, item_size);
        assert!(allocator.is_ok());
        let allocator = allocator.unwrap();

        assert_eq!(allocator.capacity(), capacity);
    }

    #[test]
    fn test_read_empty_item() {
        let capacity = 4;
        let item_size = 8;
        let total_size = SliceAllocator::get_size(capacity, item_size);
        let buffer = vec![0u8; total_size];

        let allocator = SliceAllocator::try_from_slice(&buffer, capacity, item_size).unwrap();
        assert!(allocator.is_empty(0));
        assert!(!allocator.has_item(0));
    }

    #[test]
    fn test_alloc_and_free_item() {
        let capacity = 4;
        let item_size = 8;
        let total_size = SliceAllocator::get_size(capacity, item_size);
        let mut buffer = vec![0u8; total_size];

        let mut allocator = SliceAllocatorMut::try_from_slice_mut(&mut buffer, capacity, item_size).unwrap();

        assert!(allocator.is_empty(1));
        assert!(allocator.try_alloc_item(1, item_size).is_ok());
        assert!(!allocator.is_empty(1));
        assert!(allocator.has_item(1));

        assert!(allocator.try_free_item(1).is_ok());
        assert!(allocator.is_empty(1));
    }

    #[test]
    fn test_write_and_read_item() {
        let capacity = 4;
        let item_size = 8;
        let total_size = SliceAllocator::get_size(capacity, item_size);
        let mut buffer = vec![0u8; total_size];
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8];

        let mut allocator = SliceAllocatorMut::try_from_slice_mut(&mut buffer, capacity, item_size).unwrap();
        allocator.try_alloc_item(2, data.len()).unwrap();
        allocator.try_write_item(2, &data).unwrap();

        let read_data = allocator.read_item(2).unwrap();
        assert_eq!(read_data, data);
    }

    #[test]
    fn test_invalid_allocation() {
        let capacity = 4;
        let item_size = 8;
        let total_size = SliceAllocator::get_size(capacity, item_size);
        let mut buffer = vec![0u8; total_size];

        let mut allocator = SliceAllocatorMut::try_from_slice_mut(&mut buffer, capacity, item_size).unwrap();

        // Exceeding capacity
        assert!(allocator.try_alloc_item(5, item_size).is_err());

        // Allocating an already used item
        allocator.try_alloc_item(1, item_size).unwrap();
        assert!(allocator.try_alloc_item(1, item_size).is_err());

        // Allocating with oversized data
        assert!(allocator.try_alloc_item(2, item_size + 1).is_err());
    }

    #[test]
    fn test_invalid_write() {
        let capacity = 4;
        let item_size = 8;
        let total_size = SliceAllocator::get_size(capacity, item_size);
        let mut buffer = vec![0u8; total_size];
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];

        let mut allocator = SliceAllocatorMut::try_from_slice_mut(&mut buffer, capacity, item_size).unwrap();

        // Writing to an empty item
        assert!(allocator.try_write_item(1, &data).is_err());

        // Writing oversized data
        allocator.try_alloc_item(1, item_size).unwrap();
        assert!(allocator.try_write_item(1, &data).is_err());
    }

    #[test]
    fn test_read_invalid_item() {
        let capacity = 4;
        let item_size = 8;
        let total_size = SliceAllocator::get_size(capacity, item_size);
        let buffer = vec![0u8; total_size];

        let allocator = SliceAllocator::try_from_slice(&buffer, capacity, item_size).unwrap();

        // Reading an out-of-bounds index
        assert!(allocator.read_item(5).is_none());
    }
}
