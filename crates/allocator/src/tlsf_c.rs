//! TLSF memory allocation.
//!
//! 

use super::{AllocError, AllocResult, BaseAllocator, ByteAllocator};
use core::alloc::Layout;
use tlsf_c_allocator::Heap;


pub struct TLSFCAllocator {
    inner: Option<Heap>,
}

impl TLSFCAllocator {
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

impl BaseAllocator for TLSFCAllocator {
    fn init(&mut self, start: usize, size: usize){
        //log::debug!("init: start = {:#x}, size = {:#?}",start, size);
        self.inner = Some(Heap::new());
        self.inner_mut().init(start, size);
    }

    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        //log::debug!("add memory: start = {:#x}, size = {:#?}",start, size);
        self.inner_mut().add_memory(start, size);
        Ok(())
    }
}

impl ByteAllocator for TLSFCAllocator {
    fn alloc(&mut self, size: usize, align_pow2: usize) -> AllocResult<usize> {
        //log::debug!("alloc: {:#?}",size);
        self.inner_mut()
        //.allocate(Layout::from_size_align(size, align_pow2).unwrap())
        .allocate(size, align_pow2)
        .map_err(|_| AllocError::NoMemory)
    }

    fn dealloc(&mut self, pos: usize, size: usize, align_pow2: usize) {
        //log::debug!("dealloc: {:#x} {:#?}",pos,size);
        self.inner_mut()
        //.deallocate(pos, Layout::from_size_align(size, align_pow2).unwrap())
        .deallocate(pos, size, align_pow2)
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
