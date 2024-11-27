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

    fn read(&self, item_index: usize, size: usize) -> Option<&[u8]> {
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

    fn read(&self, item_index: usize, size: usize) -> Option<&[u8]> {
        if item_index >= self.capacity() || size > self.item_size {
            return None;
        }

        let item_start = item_index * self.item_size;
        Some(&self.data[item_start..item_start + size])
    }

    fn write(&mut self, item_index: usize, data: &[u8]) -> ProgramResult {
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
    use solana_sdk::{signature::Keypair, signer::Signer};

    /// Helper function to create a `SliceAllocator` for testing.
    /// Returns the allocator and the backing buffer to ensure the buffer's lifetime.
    fn create_allocator<'a>(
        buffer: &'a [u8],
        capacity: usize,
        item_size: usize,
    ) -> Result<SliceAllocator<'a>, ProgramError> {
        SliceAllocator::try_from_slice(buffer, capacity, item_size)
    }

    /// Helper function to create a `SliceAllocatorMut` for testing.
    /// Returns the allocator and the mutable backing buffer to ensure the buffer's lifetime.
    fn create_allocator_mut<'a>(
        buffer: &'a mut [u8],
        capacity: usize,
        item_size: usize,
    ) -> Result<SliceAllocatorMut<'a>, ProgramError> {
        SliceAllocatorMut::try_from_slice_mut(buffer, capacity, item_size)
    }

    #[test]
    fn test_allocator_creation() {
        let capacity = 4;
        let item_size = 8;
        let total_size = SliceAllocator::get_size(capacity, item_size);
        let mut buffer = vec![0u8; total_size];

        let allocator = create_allocator(&buffer, capacity, item_size);
        assert!(allocator.is_ok());
        assert_eq!(allocator.unwrap().capacity(), capacity);

        let mut_allocator = create_allocator_mut(&mut buffer, capacity, item_size);
        assert!(mut_allocator.is_ok());
        assert_eq!(mut_allocator.unwrap().capacity(), capacity);
    }

    #[test]
    fn test_initial_state() {
        let capacity = 4;
        let item_size = 8;
        let total_size = SliceAllocator::get_size(capacity, item_size);
        let buffer = vec![0u8; total_size];

        let allocator = create_allocator(&buffer, capacity, item_size).unwrap();
        for i in 0..capacity {
            assert!(allocator.is_empty(i as u16));
            assert!(!allocator.has_item(i as u16));
        }
    }

    #[test]
    fn test_alloc_and_free_item() {
        let capacity = 4;
        let item_size = 8;
        let total_size = SliceAllocator::get_size(capacity, item_size);
        let mut buffer = vec![0u8; total_size];

        let mut allocator = create_allocator_mut(&mut buffer, capacity, item_size).unwrap();

        assert!(allocator.try_alloc_item(1, item_size).is_ok());
        assert!(!allocator.is_empty(1));
        assert!(allocator.has_item(1));

        assert!(allocator.try_free_item(1).is_ok());
        assert!(allocator.is_empty(1));
        assert!(!allocator.has_item(1));
    }

    #[test]
    fn test_write_and_read_item() {
        let capacity = 4;
        let item_size = 8;
        let total_size = SliceAllocator::get_size(capacity, item_size);
        let mut buffer = vec![0u8; total_size];
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8];

        let mut allocator = create_allocator_mut(&mut buffer, capacity, item_size).unwrap();

        assert!(allocator.try_alloc_item(2, data.len()).is_ok());
        assert!(allocator.try_write_item(2, &data).is_ok());

        let read_data = allocator.read_item(2);
        assert!(read_data.is_some());
        assert_eq!(read_data.unwrap(), data);
    }

    #[test]
    fn test_invalid_operations() {
        let capacity = 4;
        let item_size = 8;
        let total_size = SliceAllocator::get_size(capacity, item_size);
        let mut buffer = vec![0u8; total_size];

        let mut allocator = create_allocator_mut(&mut buffer, capacity, item_size).unwrap();

        // Invalid allocation: Out of bounds
        assert!(allocator.try_alloc_item(5, item_size).is_err());

        // Invalid allocation: Already allocated
        assert!(allocator.try_alloc_item(1, item_size).is_ok());
        assert!(allocator.try_alloc_item(1, item_size).is_err());

        // Invalid allocation: Exceeds item size
        assert!(allocator.try_alloc_item(2, item_size + 1).is_err());

        // Invalid write: Item not allocated
        assert!(allocator.try_write_item(0, &[1, 2, 3]).is_err());

        // Invalid write: Data exceeds item size
        let oversized_data = vec![0u8; item_size + 1];
        assert!(allocator.try_write_item(1, &oversized_data).is_err());

        // Invalid free: Not allocated
        assert!(allocator.try_free_item(3).is_err());
    }

    #[test]
    fn test_read_invalid_item() {
        let capacity = 4;
        let item_size = 8;
        let total_size = SliceAllocator::get_size(capacity, item_size);
        let buffer = vec![0u8; total_size];

        let allocator = create_allocator(&buffer, capacity, item_size).unwrap();

        // Reading out-of-bounds index
        assert!(allocator.read_item(5).is_none());

        // Reading unallocated item
        assert!(allocator.read_item(0).is_none());
    }

    #[test]
    fn test_large_capacity_read_write() {
        let capacity = 1000; // Large number of items
        let item_size = 77; // Each item is 77 bytes
        let total_size = SliceAllocator::get_size(capacity, item_size);
        let mut buffer = vec![0u8; total_size];

        let mut allocator = create_allocator_mut(&mut buffer, capacity, item_size).unwrap();

        // Generate some test data
        let test_data: Vec<u8> = (0..item_size as u8).collect();

        // Write data to all items in the allocator
        for i in 0..capacity {
            let index = i as u16;
            assert!(allocator.try_alloc_item(index, item_size).is_ok());
            assert!(allocator.try_write_item(index, &test_data).is_ok());
            assert!(allocator.has_item(index));
        }

        // Verify the data is correctly stored and can be read back
        for i in 0..capacity {
            let index = i as u16;
            let read_data = allocator.read_item(index);
            assert!(read_data.is_some());
            assert_eq!(read_data.unwrap(), test_data);
        }

        // Free all items
        for i in 0..capacity {
            let index = i as u16;
            assert!(allocator.try_free_item(index).is_ok());
            assert!(allocator.is_empty(index));
        }
    }

    #[test]
    fn test_large_capacity_partial_read_write() {

        let capacity = 1000; // Large number of items
        let item_size = 77; // Each item is 77 bytes
        let total_size = SliceAllocator::get_size(capacity, item_size);
        let mut buffer = vec![0u8; total_size];

        let mut allocator = create_allocator_mut(&mut buffer, capacity, item_size).unwrap();

        // Generate some test data
        let test_data: Vec<u8> = (0..item_size as u8).collect();

        // Write data to every 10th item
        for i in (0..capacity).step_by(10) {
            let index = i as u16;
            assert!(allocator.try_alloc_item(index, item_size).is_ok());
            assert!(allocator.try_write_item(index, &test_data).is_ok());
            assert!(allocator.has_item(index));
        }

        // Verify data only exists in the written indices
        for i in 0..capacity {
            let index = i as u16;
            if i % 10 == 0 {
                let read_data = allocator.read_item(index);
                assert!(read_data.is_some());
                assert_eq!(read_data.unwrap(), test_data);
            } else {
                assert!(allocator.read_item(index).is_none());
            }
        }

        // Free every 10th item
        for i in (0..capacity).step_by(10) {
            let index = i as u16;
            assert!(allocator.try_free_item(index).is_ok());
            assert!(allocator.is_empty(index));
        }
    }

    #[test]
    fn test_sign_verify_with_allocator() {
        let capacity = 500; // Number of items to write
        let item_size = 64; // Signature size (64 bytes)
        let total_size = SliceAllocator::get_size(capacity, item_size);
        let mut buffer = vec![0u8; total_size];

        let mut allocator = create_allocator_mut(&mut buffer, capacity, item_size).unwrap();

        // Generate a keypair for signing
        let keypair = Keypair::new();

        // Vector to track messages and their indices
        let mut messages = Vec::new();

        // Sign and write messages into the allocator
        for i in 0..capacity {
            let message = format!("test_{}", i);
            let signature = keypair.sign_message(message.as_bytes());

            // Attempt to allocate and write the signature
            let index = i as u16;
            assert!(allocator.try_alloc_item(index, item_size).is_ok());
            assert!(allocator.try_write_item(index, signature.as_ref()).is_ok());

            // Store the message for later verification
            messages.push((index, message));
        }

        // Read and verify messages
        for (index, message) in messages {
            let read_data = allocator.read_item(index);
            assert!(read_data.is_some());

            let signature_bytes = read_data.unwrap();
            assert_eq!(signature_bytes.len(), item_size);

            // Re-sign the message and compare the signatures
            let expected_signature = keypair.sign_message(message.as_bytes());
            assert_eq!(signature_bytes, expected_signature.as_ref());
        }

        // Free all allocated items and ensure they return to an empty state
        for i in 0..capacity {
            let index = i as u16;
            assert!(allocator.try_free_item(index).is_ok());
            assert!(allocator.is_empty(index));
        }
    }
}
