
use steel::*;
use std::fmt::Debug;

pub trait MemoryAllocator : Debug {
    fn is_empty(&self, index: u16) -> bool;
    fn has_item(&self, index: u16) -> bool;
    fn read_item(&self, index: u16) -> Option<Vec<u8>>;
    fn try_alloc_item(&mut self, index: u16, size: usize) -> ProgramResult;
    fn try_free_item(&mut self, index: u16) -> ProgramResult;
    fn try_write_item(&mut self, index: u16, data: &[u8]) -> ProgramResult;
}