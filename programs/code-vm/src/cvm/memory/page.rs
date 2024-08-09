use bytemuck::{Pod, Zeroable};
use crate::types::ZeroCopy;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Page<const PAGE_SIZE: usize> {
    pub is_allocated: u8,      // 0 = free, 1 = allocated (not bool due to Pod/Zeroable)
    pub data: [u8; PAGE_SIZE],
    pub next_page: u8,         // index of the next page in the sector
}

impl<const PAGE_SIZE: usize> Default for Page<PAGE_SIZE> {
    fn default() -> Self {
        Self {
            is_allocated: 0,
            data: [0u8; PAGE_SIZE],
            next_page: 0,
        }
    }
}

impl<const PAGE_SIZE: usize> Page<PAGE_SIZE> {
    pub const CAPACITY: usize = PAGE_SIZE;

    pub fn next(&self) -> Option<u8> {
        match self.next_page {
            0 => None,
            i => Some(i),
        }
    }

    pub fn get_next_ref(&self) -> PageReference {
        PageReference::unpack(self.next_page)
    }

    pub fn set_next_ref(&mut self, next: PageReference) {
        self.next_page = next.pack();
    }

    pub fn has_next(&self) -> bool {
        self.get_next_ref() != PageReference::None
    }

    pub fn is_empty(&self) -> bool {
        self.is_allocated == 0
    }

    pub fn is_allocated(&self) -> bool {
        self.is_allocated == 1
    }

    pub fn allocate(&mut self) {
        self.is_allocated = 1;
    }

    pub fn free(&mut self) {
        self.is_allocated = 0;
        self.data.fill(0);
        self.set_next_ref(PageReference::None);
    }

    pub fn calc_num_pages_needed(data_size: usize) -> usize {
        (data_size + PAGE_SIZE - 1) / PAGE_SIZE
    }


}

impl<const PAGE_SIZE: usize> ZeroCopy 
    for Page<PAGE_SIZE> {
}

unsafe impl<const PAGE_SIZE: usize> Zeroable
    for Page<PAGE_SIZE> {
}

unsafe impl<const PAGE_SIZE: usize> Pod
    for Page<PAGE_SIZE> {
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PageReference {
    None = 0,
    Index(u8),
}

impl PageReference {
    pub fn pack(&self) -> u8 {
        match self {
            PageReference::None => 0,
            PageReference::Index(i) => *i,
        }
    }

    pub fn unpack(value: u8) -> PageReference {
        match value {
            0 => PageReference::None,
            i => PageReference::Index(i),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn test_page() {
        let mut data = [0u8; size_of::<Page<32>>()];
        let page = Page::<32>::load_mut_bytes(&mut data).unwrap();

        assert_eq!(page.is_allocated(), false);
        assert_eq!(page.is_empty(), true);
        assert_eq!(page.has_next(), false);

        page.allocate();
        assert_eq!(page.is_allocated(), true);
        assert_eq!(page.is_empty(), false);
        assert_eq!(page.has_next(), false);

        page.data.copy_from_slice(&[3u8; 32]);

        assert_eq!(data[0], 1);
        assert_eq!(data[1..33], [3u8; 32]);
    }

    #[test]
    fn test_page_ref() {
        let none = PageReference::None;
        let index = PageReference::Index(1);

        assert_eq!(none.pack(), 0);
        assert_eq!(index.pack(), 1);

        assert_eq!(PageReference::unpack(0), none);
        assert_eq!(PageReference::unpack(1), index);

        let mut data = [0u8; size_of::<Page<32>>()];
        let page = Page::<32>::load_mut_bytes(&mut data).unwrap();
        page.set_next_ref(PageReference::Index(1));

        assert_eq!(page.has_next(), true);
        assert_eq!(page.get_next_ref(), PageReference::Index(1));
    }

    #[test]
    fn test_page_calc_num_pages_needed() {
        assert_eq!(Page::<32>::calc_num_pages_needed(0), 0);
        assert_eq!(Page::<32>::calc_num_pages_needed(1), 1);
        assert_eq!(Page::<32>::calc_num_pages_needed(32), 1);
        assert_eq!(Page::<32>::calc_num_pages_needed(33), 2);
    }

    #[test]
    fn test_page_free() {
        let mut data = [0u8; size_of::<Page<32>>()];
        let page = Page::<32>::load_mut_bytes(&mut data).unwrap();

        page.allocate();
        page.data.copy_from_slice(&[3u8; 32]);
        page.set_next_ref(PageReference::Index(1));

        page.free();

        assert_eq!(page.is_allocated(), false);
        assert_eq!(page.is_empty(), true);
        assert_eq!(page.has_next(), false);
        assert_eq!(page.data, [0u8; 32]);
    }
}