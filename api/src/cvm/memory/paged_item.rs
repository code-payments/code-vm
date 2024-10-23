use bytemuck::{Pod, Zeroable};
use crate::types::ZeroCopy;

use super::Page;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PagedItem<const PAGE_SIZE: usize> {
    pub size: u16,
    pub page: u8,
    pub sector: u8,
}

impl<const PAGE_SIZE: usize> PagedItem<PAGE_SIZE> {
    pub const LEN: usize = 
        2 + // size
        1 + // page
        1;  // sector

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn is_allocated(&self) -> bool {
        !self.is_empty()
    }

    pub fn num_pages(&self) -> usize {
        Page::<PAGE_SIZE>::calc_num_pages_needed(self.size as usize)
    }

    pub fn clear(&mut self) {
        self.size = 0;
        self.sector = 0;
        self.page = 0;
    }
}

impl<const PAGE_SIZE: usize> ZeroCopy 
    for PagedItem<PAGE_SIZE> {
}

unsafe impl<const PAGE_SIZE: usize> Zeroable
    for PagedItem<PAGE_SIZE> {
}

unsafe impl<const PAGE_SIZE: usize> Pod
    for PagedItem<PAGE_SIZE> {
}
