//! Basic memory allocation.
//!
//! TODO

use super::{AllocError, AllocResult, BaseAllocator, ByteAllocator};
use core::alloc::Layout;
use basic_allocator::Heap;


pub struct BasicAllocator {
    inner: Option<Heap>,
}

impl BasicAllocator {
    pub const fn new() -> Self {
        Self { inner: None }
    }

    fn inner_mut(&mut self) -> &mut Heap {
        self.inner.as_mut().unwrap()
    }

    fn inner(&self) -> &Heap {
        self.inner.as_ref().unwrap()
    }
}

impl BaseAllocator for BasicAllocator {
    fn init(&mut self, start: usize, size: usize){
        //log::debug!("init: start = {:#x}, size = {:#?}",start, size);
        self.inner = Some(Heap::new());
        unsafe {
            self.inner_mut().init(start, size);
        }
    }

    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        
        unsafe {
            self.inner_mut().add_memory(start, size);
        }
        Ok(())
    }
}

impl ByteAllocator for BasicAllocator {
    fn alloc(&mut self, size: usize, align_pow2: usize) -> AllocResult<usize> {
        //log::debug!("alloc: {:#?}",size);
        self.inner_mut()
        .allocate(Layout::from_size_align(size, align_pow2).unwrap())
        .map_err(|_| AllocError::NoMemory)
    }

    fn dealloc(&mut self, pos: usize, size: usize, align_pow2: usize) {
        unsafe {
            self.inner_mut()
                .deallocate(pos, Layout::from_size_align(size, align_pow2).unwrap())
        }
    }

    fn total_bytes(&self) -> usize {
        self.inner().total_bytes()
    }

    fn used_bytes(&self) -> usize {
        self.inner().used_bytes()
    }

    fn available_bytes(&self) -> usize {
        self.inner().available_bytes()
    }
}
