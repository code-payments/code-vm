use steel::*;
use bytemuck::{Pod, Zeroable};

use crate::helpers::check_condition;
use crate::types::ZeroCopy;
use super::{Page, PageReference};

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Sector<const NUM_PAGES: usize, const PAGE_SIZE: usize> {
    pub num_allocated: u8,
    pub pages: [Page<PAGE_SIZE>; NUM_PAGES],
}

impl <const NUM_PAGES: usize, const PAGE_SIZE: usize> Default 
    for Sector<NUM_PAGES, PAGE_SIZE> {
    fn default() -> Self {
        Self {
            num_allocated: 0,
            pages: [Page::default(); NUM_PAGES],
        }
    }
}

impl <const NUM_PAGES: usize, const PAGE_SIZE: usize> 
    Sector<NUM_PAGES, PAGE_SIZE> {

    pub fn get_num_empty(&self) -> u8 {
        (NUM_PAGES - self.num_allocated as usize) as u8
    }

    fn try_find_empty(&self, num_required: usize) -> Result<Vec<u8>, ProgramError> {
        let mut empty = Vec::with_capacity(num_required);

        for (i, page) in self.pages.iter().enumerate() {
            if empty.len() == num_required {
                break;
            }

            if page.is_empty() {
                empty.push(i as u8);
            }
        }

        check_condition(
            empty.len() >= num_required,
            "memory sector has insufficient empty pages",
        )?;

        Ok(empty)
    }

    pub fn get_linked_pages(&self, start_index: u8) -> Vec<u8> {
        let mut linked = Vec::new();
        let mut current = start_index;

        loop {
            linked.push(current);

            let page = self.pages[current as usize];
            match page.next() {
                Some(next) => current = next,
                _ => break,
            }
        }

        linked
    }

    pub fn try_alloc_pages(&mut self, data_size: usize) -> Result<u8, ProgramError> {
        let num_required = Page::<PAGE_SIZE>::calc_num_pages_needed(data_size);

        check_condition(
            num_required <= self.get_num_empty() as usize,
            "memory sector has insufficient empty pages"
        )?;

        // Find empty pages
        let empty = self.try_find_empty(num_required)?;

        // Allocate the pages
        for i in 0..num_required {
            let next = if i == num_required - 1 {
                PageReference::None
            } else {
                PageReference::Index(empty[i + 1])
            };

            let page = &mut self.pages[empty[i] as usize];
            page.allocate();
            page.set_next_ref(next);
        }

        self.update_state();

        Ok(empty[0])
    }

    pub fn free_pages(&mut self, starting_index: u8) {
        let pages = self.get_linked_pages(starting_index);
        for page_index in pages {
            self.pages[page_index as usize].free();
        }

        self.update_state();
    }

    fn update_state(&mut self) {
        self.num_allocated = self.pages.iter()
            .filter(|page| page.is_allocated())
            .count() as u8;
    }
}


impl<const NUM_PAGES: usize, const PAGE_SIZE: usize> ZeroCopy 
    for Sector<NUM_PAGES, PAGE_SIZE> {
}

unsafe impl<const NUM_PAGES: usize, const PAGE_SIZE: usize> Zeroable
    for Sector<NUM_PAGES, PAGE_SIZE> {
}

unsafe impl<const NUM_PAGES: usize, const PAGE_SIZE: usize> Pod
    for Sector<NUM_PAGES, PAGE_SIZE> {
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sector() {
        let mut sector: Sector<4, 32> = Default::default();
        assert_eq!(sector.num_allocated, 0);
        assert_eq!(sector.get_num_empty(), 4);

        let index = sector.try_alloc_pages(32).unwrap();
        assert_eq!(sector.num_allocated, 1);
        assert_eq!(sector.get_num_empty(), 3);

        let linked = sector.get_linked_pages(index);
        assert_eq!(linked.len(), 1);
        assert_eq!(linked[0], index);

        let index2 = sector.try_alloc_pages(32).unwrap();
        assert_eq!(sector.num_allocated, 2);
        assert_eq!(sector.get_num_empty(), 2);

        let linked = sector.get_linked_pages(index2);
        assert_eq!(linked.len(), 1);
        assert_eq!(linked[0], index2);

        sector.free_pages(index);
        assert_eq!(sector.num_allocated, 1);
        assert_eq!(sector.get_num_empty(), 3);

        sector.free_pages(index2);
        assert_eq!(sector.num_allocated, 0);
        assert_eq!(sector.get_num_empty(), 4);
    }

    #[test]
    fn test_multiple_page_allocation() {
        let mut sector: Sector<4, 32> = Default::default();
        assert_eq!(sector.num_allocated, 0);
        assert_eq!(sector.get_num_empty(), 4);

        let index = sector.try_alloc_pages(32 * 3).unwrap();
        assert_eq!(sector.num_allocated, 3);
        assert_eq!(sector.get_num_empty(), 1);

        let linked = sector.get_linked_pages(index);
        assert_eq!(linked.len(), 3);
        assert_eq!(linked[0], index);

        sector.free_pages(index);

        assert_eq!(sector.num_allocated, 0);
        assert_eq!(sector.get_num_empty(), 4);
    }
}