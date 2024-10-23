use steel::*;
use bytemuck::{Pod, Zeroable};

use crate::helpers::check_condition;
use crate::types::ZeroCopy;
use super::{ Page, Sector, PagedItem, MemoryAllocator };

// A bit verbose on some of the constants, unfortunately const generic
// expressions are not stable yet.

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PagedAllocator
    <const CAPACITY: usize, const SECTORS: usize, const PAGES: usize, const PAGE_SIZE: usize>
{
    pub items: [PagedItem<PAGE_SIZE>; CAPACITY],
    pub sectors: [Sector<PAGES, PAGE_SIZE>; SECTORS],
}

impl <const CAPACITY: usize, const SECTORS: usize, const PAGES: usize, const PAGE_SIZE: usize>
    PagedAllocator<CAPACITY, SECTORS, PAGES, PAGE_SIZE> {

    pub const MAX_ITEMS: usize = CAPACITY;   // Allocated memory lookup table (can't be more than 2^16)
    pub const NUM_SECTORS: usize = SECTORS;  // Number of sectors in this memory (can't be more than 255)
    pub const NUM_PAGES: usize = PAGES;      // Number of pages in each sector (can't be more than 255)
    pub const PAGE_SIZE: usize = PAGE_SIZE;  // Size of a page (in bytes)

    pub fn calc_num_pages_needed(item_size: usize) -> usize {
        Page::<PAGE_SIZE>::calc_num_pages_needed(item_size)
    }

    pub fn has_room_for(&self, item_size: usize) -> bool {
        let count_pages = Self::calc_num_pages_needed(item_size);
        self.find_sector_for(count_pages).is_some()
    }

    pub fn find_sector_for(&self, count_pages: usize) -> Option<usize> {
        for (i, sector) in self.sectors.iter().enumerate() {
            if count_pages < sector.get_num_empty() as usize {
                return Some(i);
            }
        }
        None
    }

    pub fn get_item_info(&self, item_index: u16) -> Option<PagedItem<PAGE_SIZE>> {
        let item = self.items[item_index as usize];
        match item.is_allocated() {
            true => Some(item),
            false => None,
        }
    }
}

impl <const CAPACITY: usize, const SECTORS: usize, const PAGES: usize, const PAGE_SIZE: usize> 
    MemoryAllocator for PagedAllocator<CAPACITY, SECTORS, PAGES, PAGE_SIZE> {

    fn is_empty(&self, index: u16) -> bool {
        match self.get_item_info(index) {
            Some(_) => false,
            None => true,
        }
    }

    fn has_item(&self, index: u16) -> bool {
        match self.get_item_info(index) {
            Some(_) => true,
            None => false,
        }
    }

    fn read_item(&self, item_index: u16) -> Option<Vec<u8>> {
        let item = self.items[item_index as usize];
        if item.is_empty() {
            return None;
        }

        let sector = &self.sectors[item.sector as usize];
        let pages = sector.get_linked_pages(item.page);

        let mut data = Vec::new();
        for page_index in pages {
            let page = &sector.pages[page_index as usize];
            data.extend_from_slice(&page.data);
        }

        Some(data)
    }

    fn try_alloc_item(&mut self, item_index: u16, data_size: usize) -> ProgramResult {

        let item = self.items[item_index as usize];

        check_condition(
            item.is_empty(),
            "allocate failed, memory index is already allocated"
        )?;

        let num_required = Page::<PAGE_SIZE>::calc_num_pages_needed(data_size);
        let sector_index = self.find_sector_for(num_required);

        check_condition(
            sector_index.is_some(),
            "allocate failed, no sector has enough space"
        )?;

        let sector_index = sector_index.unwrap();
        let sector = &mut self.sectors[sector_index];
        let page_index = sector.try_alloc_pages(data_size)?;


        let item = PagedItem {
            size: data_size as u16,
            sector: sector_index as u8,
            page: page_index,
        };

        self.items[item_index as usize] = item;

        Ok(())
    }

    fn try_free_item(&mut self, item_index: u16) -> ProgramResult {
        let item = &mut self.items[item_index as usize];
        if item.is_allocated() {
            let sector = &mut self.sectors[item.sector as usize];
            sector.free_pages(item.page);
            item.clear();
        }

        Ok(())
    }

    fn try_write_item(&mut self, item_index: u16, data: &[u8]) -> ProgramResult {
        let item = &mut self.items[item_index as usize];

        check_condition(
            item.is_allocated(),
            "write failed, memory index is not allocated"
        )?;

        let size = data.len() as u16;

        check_condition(
            item.size >= size,
            "write failed, data size is larger than allocated memory"
        )?;

        let sector = &mut self.sectors[item.sector as usize];
        let pages = sector.get_linked_pages(item.page);

        assert!(item.num_pages() == pages.len());

        let chunks = data.chunks(PAGE_SIZE);

        check_condition(
            pages.len() >= chunks.len(),
            "write failed, chunk count is larger than allocated memory"
        )?;

        for (i, chunk) in chunks.enumerate() {
            let page_index = pages[i];
            let page = &mut sector.pages[page_index as usize];
            page.data[..chunk.len()].copy_from_slice(chunk);
        }

        Ok(())
    }
}


impl <const CAPACITY: usize, const SECTORS: usize, const PAGES: usize, const PAGE_SIZE: usize>
    ZeroCopy for PagedAllocator<CAPACITY, SECTORS, PAGES, PAGE_SIZE> {
}

unsafe impl <const CAPACITY: usize, const SECTORS: usize, const PAGES: usize, const PAGE_SIZE: usize>
    Zeroable for PagedAllocator<CAPACITY, SECTORS, PAGES, PAGE_SIZE> {
}

unsafe impl <const CAPACITY: usize, const SECTORS: usize, const PAGES: usize, const PAGE_SIZE: usize>
    Pod for PagedAllocator<CAPACITY, SECTORS, PAGES, PAGE_SIZE> {
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;
    use super::*;

    const NUM_SECTORS: usize = 10;
    const NUM_PAGES: usize = 255;
    const MAX_ITEMS: usize = NUM_SECTORS * NUM_PAGES;
    const PAGE_SIZE: usize = 32;

    type TestMemory = PagedAllocator<MAX_ITEMS, NUM_SECTORS, NUM_PAGES, PAGE_SIZE>;

    #[test]
    fn test_small_item() {
        let mut buf = vec![0; size_of::<TestMemory>()];
        let mem = TestMemory::load_mut_bytes(&mut buf).unwrap();

        assert!(mem.items.len() == TestMemory::MAX_ITEMS);
        assert!(mem.sectors.len() == TestMemory::NUM_SECTORS);

        let item_index = 0;
        let item_data = [1, 2, 3, 4, 5];

        assert!(mem.try_alloc_item(item_index, item_data.len()).is_ok());
        assert!(mem.try_write_item(item_index, &item_data).is_ok());

        let read_data = mem.read_item(item_index).unwrap();
        assert_eq!(read_data.as_slice()[..5], item_data);
    }

    #[test]
    fn test_large_item() {
        let mut buf = vec![0; size_of::<TestMemory>()];
        let mem = TestMemory::load_mut_bytes(&mut buf).unwrap();

        let item_index = 0;
        let item_data: [u8; 256] = (0..=255).collect::<Vec<u8>>().try_into().unwrap();

        assert!(mem.try_alloc_item(item_index, item_data.len()).is_ok());
        assert!(mem.try_write_item(item_index, &item_data).is_ok());

        let read_data = mem.read_item(item_index).unwrap();
        assert_eq!(read_data.as_slice()[..item_data.len()], item_data);
    }

    #[test]
    fn test_multiple_items() {
        let mut buf = vec![0; size_of::<TestMemory>()];
        let mem = TestMemory::load_mut_bytes(&mut buf).unwrap();

        let a: [u8; 42] = [42; 42];
        let b: [u8; 69] = [69; 69];
        let c: [u8; 137] = [137; 137];

        assert!(mem.try_alloc_item(0, a.len()).is_ok());
        assert!(mem.try_alloc_item(1, b.len()).is_ok());
        assert!(mem.try_alloc_item(2, c.len()).is_ok());

        assert!(mem.try_write_item(0, &a).is_ok());
        assert!(mem.try_write_item(1, &b).is_ok());
        assert!(mem.try_write_item(2, &c).is_ok());

        let read_a = mem.read_item(0).unwrap();
        let read_b = mem.read_item(1).unwrap();
        let read_c = mem.read_item(2).unwrap();

        assert_eq!(read_a.as_slice()[..42], a);
        assert_eq!(read_b.as_slice()[..69], b);
        assert_eq!(read_c.as_slice()[..137], c);
    }

    #[test]
    fn test_random_allocate() {
        let mut buf = vec![0; size_of::<TestMemory>()];
        let mem = TestMemory::load_mut_bytes(&mut buf).unwrap();

        let a: [u8; 42] = [42; 42];
        let b: [u8; 69] = [69; 69];
        let c: [u8; 137] = [137; 137];

        assert!(mem.try_alloc_item(192, a.len()).is_ok());
        assert!(mem.try_alloc_item(180, b.len()).is_ok());
        assert!(mem.try_alloc_item(28, c.len()).is_ok());

        assert!(mem.try_write_item(192, &a).is_ok());
        assert!(mem.try_write_item(180, &b).is_ok());
        assert!(mem.try_write_item(28, &c).is_ok());

        let read_a = mem.read_item(192).unwrap();
        let read_b = mem.read_item(180).unwrap();
        let read_c = mem.read_item(28).unwrap();

        assert_eq!(read_a.as_slice()[..a.len()], a);
        assert_eq!(read_b.as_slice()[..b.len()], b);
        assert_eq!(read_c.as_slice()[..c.len()], c);
    }

    #[test]
    fn test_free() {
        let mut buf = vec![0; size_of::<TestMemory>()];
        let mem = TestMemory::load_mut_bytes(&mut buf).unwrap();

        let a: [u8; 42] = [42; 42];
        let b: [u8; 69] = [69; 69];
        let c: [u8; 137] = [137; 137];
        let d: [u8; 255] = [255; 255];

        assert!(mem.try_alloc_item(0, a.len()).is_ok());
        assert!(mem.try_alloc_item(1, b.len()).is_ok());
        assert!(mem.try_alloc_item(2, c.len()).is_ok());

        assert!(mem.try_write_item(0, &a).is_ok());
        assert!(mem.try_write_item(1, &b).is_ok());
        assert!(mem.try_write_item(2, &c).is_ok());

        assert!(mem.try_free_item(1).is_ok());
        assert!(mem.try_alloc_item(1, d.len()).is_ok());
        assert!(mem.try_write_item(1, &d).is_ok());

        let read_a = mem.read_item(0).unwrap();
        let read_b = mem.read_item(1).unwrap();
        let read_c = mem.read_item(2).unwrap();

        assert_eq!(read_a.as_slice()[..a.len()], a);
        assert_eq!(read_b.as_slice()[..d.len()], d);
        assert_eq!(read_c.as_slice()[..c.len()], c);
    }
}