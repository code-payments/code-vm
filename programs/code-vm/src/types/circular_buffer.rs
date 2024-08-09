use anchor_lang::prelude::*;

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Default, Debug)]
pub struct CircularBuffer<const ITEM_SIZE: usize> {
    capacity: u8,
    offset: u8,
    items: Vec<[u8; ITEM_SIZE]>,
}

impl<const ITEM_SIZE: usize> CircularBuffer<ITEM_SIZE> {
    pub const MIN_SIZE: usize = 
        1 + // capacity
        1 + // index
        4;  // items (vec length)

    pub fn max_size_for(capacity: u8) -> usize {
        Self::MIN_SIZE + ITEM_SIZE * capacity as usize // items
    }

    pub fn new(capacity: u8) -> Self {
        CircularBuffer {
            capacity,
            offset: 0,
            items: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn find_index(&self, item: &[u8]) -> Option<usize> {
        self.items.iter().position(|x| x.eq(item))
    }

    pub fn contains(&self, item: &[u8]) -> bool {
        self.find_index(item).is_some()
    }

    pub fn unroll(&self) -> Vec<[u8; ITEM_SIZE]> {
        let mut list = Vec::new();
        for i in 0..self.capacity as usize {
            list.push(self.items[
                (self.offset as usize + i) % self.capacity as usize]
            );
        }
        list
    }

    pub fn push(&mut self, item: &[u8]) {
        if self.items.len() < self.capacity as usize {
            self.items.push(item[..ITEM_SIZE].try_into().unwrap());
        } else {
            self.items[self.offset as usize].copy_from_slice(item[..ITEM_SIZE].as_ref());
            self.offset = (self.offset + 1) % self.capacity as u8;
        }
    }

    pub fn first(&self) -> Option<&[u8; ITEM_SIZE]> {
        if self.is_empty() {
            return None;
        }

        Some(&self.items[self.offset as usize])
    }

    pub fn last(&self) -> Option<&[u8; ITEM_SIZE]> {
        if self.is_empty() {
            return None;
        }

        let index = if self.items.len() < self.capacity as usize {
            self.items.len() as u8 - 1
        } else {
            if self.offset == 0 {
                self.capacity - 1
            } else {
                self.offset - 1
            }
        };

        Some(&self.items[index as usize])
    }

    pub fn get(&self, index: usize) -> Option<&[u8; ITEM_SIZE]> {
        if index < self.capacity as usize {
            let actual_index = (
                self.offset as usize + index
            ) % self.capacity as usize;
            Some(&self.items[actual_index])
        } else {
            None
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_buffer() {
        let capacity = 3;
        let buffer = CircularBuffer::<32>::new(capacity);

        assert_eq!(buffer.capacity, capacity);
        assert_eq!(buffer.offset, 0);
        assert_eq!(buffer.items.len(), 0);
    }

    #[test]
    fn test_round_trip() {
        let capacity = 3;
        let mut buffer = CircularBuffer::<32>::new(capacity);

        let item1 = [1; 32];
        let item2 = [2; 32];
        let item3 = [3; 32];

        buffer.push(&item1);
        buffer.push(&item2);
        buffer.push(&item3);

        assert_eq!(buffer.get(0).unwrap(), &item1);
        assert_eq!(buffer.get(1).unwrap(), &item2);
        assert_eq!(buffer.get(2).unwrap(), &item3);
    }

    #[test]
    fn test_find_index() {
        let capacity = 3;
        let mut buffer = CircularBuffer::<32>::new(capacity);

        let item1 = [1; 32];
        let item2 = [2; 32];
        let item3 = [3; 32];

        buffer.push(&item1);
        buffer.push(&item2);
        buffer.push(&item3);

        assert_eq!(buffer.find_index(&item1).unwrap(), 0);
        assert_eq!(buffer.find_index(&item2).unwrap(), 1);
        assert_eq!(buffer.find_index(&item3).unwrap(), 2);
    }

    #[test]
    fn test_overflow() {
        let capacity = 3;
        let mut buffer = CircularBuffer::<32>::new(capacity);

        let item1 = [1; 32];
        let item2 = [2; 32];
        let item3 = [3; 32];
        let item4 = [4; 32];

        buffer.push(&item1);
        buffer.push(&item2);
        buffer.push(&item3);
        buffer.push(&item4);

        assert_eq!(buffer.get(0).unwrap(), &item2);
        assert_eq!(buffer.get(1).unwrap(), &item3);
        assert_eq!(buffer.get(2).unwrap(), &item4);
    }

    #[test]
    fn test_contains() {
        let capacity = 3;
        let mut buffer = CircularBuffer::<32>::new(capacity);

        let item1 = [1; 32];
        let item2 = [2; 32];
        let item3 = [3; 32];

        buffer.push(&item1);
        buffer.push(&item2);
        buffer.push(&item3);

        assert!(buffer.contains(&item1));
        assert!(buffer.contains(&item2));
        assert!(buffer.contains(&item3));
    }

    #[test]
    fn test_unroll() {
        let capacity = 3;
        let mut buffer = CircularBuffer::<32>::new(capacity);

        let item1 = [1; 32];
        let item2 = [2; 32];
        let item3 = [3; 32];
        let item4 = [4; 32];

        buffer.push(&item1);
        buffer.push(&item2);
        buffer.push(&item3);
        buffer.push(&item4);

        let list = buffer.unroll();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0], item2);
        assert_eq!(list[1], item3);
        assert_eq!(list[2], item4);
    }

    #[test]
    fn test_first_and_last() {
        let capacity = 3;
        let mut buffer = CircularBuffer::<32>::new(capacity);

        assert!(buffer.first().is_none());
        assert!(buffer.last().is_none());

        let item1 = [1; 32];
        let item2 = [2; 32];
        let item3 = [3; 32];
        let item4 = [4; 32];

        buffer.push(&item1);
        assert!(buffer.first().unwrap().eq(&item1));
        assert!(buffer.last().unwrap().eq(&item1));

        buffer.push(&item2);
        assert!(buffer.first().unwrap().eq(&item1));
        assert!(buffer.last().unwrap().eq(&item2));

        buffer.push(&item3);
        assert!(buffer.first().unwrap().eq(&item1));
        assert!(buffer.last().unwrap().eq(&item3));

        buffer.push(&item4);
        assert!(buffer.first().unwrap().eq(&item2));
        assert!(buffer.last().unwrap().eq(&item4));
    }
}