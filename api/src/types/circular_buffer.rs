use bytemuck::{Pod, Zeroable};

#[repr(C, align(8))]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct CircularBuffer<const N: usize, const M: usize> {
    pub items: [[u8; M]; N],
    pub offset: u8,
    pub num_items: u8,
    _padding: [u8; 6],
}

unsafe impl <const N: usize, const M: usize> 
  Zeroable for CircularBuffer<N, M> {}

unsafe impl <const N: usize, const M: usize>
  Pod for CircularBuffer<N, M> {}

impl<const N: usize, const M: usize> CircularBuffer<N, M> {

    pub fn new() -> Self {
        Self {
            items: [[0; M]; N],
            offset: 0,
            num_items: 0,
            _padding: [0; 6],
        }
    }

    pub const fn capacity(&self) -> usize {
        N
    }

    pub fn is_empty(&self) -> bool {
        self.num_items == 0
    }

    pub fn find_index(&self, item: &[u8]) -> Option<usize> {
        for i in 0..self.num_items as usize {
            let idx = (self.offset as usize + i) % N;
            if self.items[idx] == item {
                return Some(i);
            }
        }
        None
    }

    pub fn contains(&self, item: &[u8]) -> bool {
        self.find_index(item).is_some()
    }

    pub fn push(&mut self, item: &[u8]) {
        // Note: item.len() might be less than M, so we need to copy up to
        // the length of the item and zero out the rest
        let mut buffer = [0; M];
        buffer[..item.len()].copy_from_slice(item);

        if self.num_items < N as u8 {
            self.items[self.num_items as usize] = buffer;
            self.num_items += 1;
        } else {
            self.items[self.offset as usize] = buffer;
            self.offset = (self.offset + 1) % N as u8;
        }
    }

    pub fn unroll(&self) -> Vec<[u8; M]> {
        let mut list = Vec::new();
        for i in 0..self.num_items {
            list.push(self.items[(self.offset as usize + i as usize) % N]);
        }
        list
    }

    pub fn first(&self) -> Option<&[u8; M]> {
        if self.is_empty() {
            return None;
        }

        Some(&self.items[self.offset as usize])
    }

    pub fn last(&self) -> Option<&[u8; M]> {
        if self.is_empty() {
            return None;
        }

        let index = if self.num_items < N as u8 {
            self.num_items - 1
        } else if self.offset == 0 {
            N as u8 - 1
        } else {
            self.offset - 1
        };

        Some(&self.items[index as usize])
    }

    pub fn get(&self, index: usize) -> Option<&[u8; M]> {
        if index < self.num_items as usize {
            let actual_index = (
                self.offset as usize + index
            ) % N as usize;
            Some(&self.items[actual_index])
        } else {
            None
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    type TestBuffer = CircularBuffer::<3, 32>;

    #[test]
    fn test_create_buffer() {
        let buffer = TestBuffer::new();

        assert_eq!(buffer.offset, 0);
        assert_eq!(buffer.num_items, 0);
        assert_eq!(buffer.items.len(), 3);
    }

    #[test]
    fn test_round_trip() {
        let mut buffer = TestBuffer::new();

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
        let mut buffer = TestBuffer::new();

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
        let mut buffer = TestBuffer::new();

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
        let mut buffer = TestBuffer::new();

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
        let mut buffer = TestBuffer::new();

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
        let mut buffer = TestBuffer::new();

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

    #[test]
    fn test_various_len_items() {
        let mut buffer = TestBuffer::new();

        let item1 = [1; 4];
        let item2 = [2; 8];
        let item3 = [3; 16];
        let item4 = [4; 32];

        buffer.push(&item1);
        buffer.push(&item2);
        buffer.push(&item3);
        buffer.push(&item4);

        assert!(buffer.get(0).unwrap()[..8].eq(&item2));
        assert!(buffer.get(1).unwrap()[..16].eq(&item3));
        assert!(buffer.get(2).unwrap()[..32].eq(&item4));
    }

    #[test]
    fn test_ignore_empty_items() {
        let mut buffer = TestBuffer::new();
        let zero_item = [0u8; 32];
        assert!(!buffer.contains(&zero_item));

        let item1 = [1u8; 32];
        buffer.push(&item1);
        assert!(!buffer.contains(&zero_item));

        buffer.push(&zero_item);
        assert!(buffer.contains(&zero_item));

        let item2 = [2u8; 32];
        let item3 = [3u8; 32];
        let item4 = [4u8; 32];

        buffer.push(&item2);
        buffer.push(&item3);
        buffer.push(&item4);

        assert!(!buffer.contains(&zero_item));
    }
}