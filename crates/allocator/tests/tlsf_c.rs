//! TLSF_C memory allocation.
//!

use allocator::{AllocResult, BaseAllocator, ByteAllocator};

use std::ffi::c_ulonglong;
#[link(name = "tlsf")]
extern {
    pub fn tlsf_create_with_pool(mem: c_ulonglong, bytes: c_ulonglong) -> c_ulonglong;
    pub fn tlsf_add_pool(tlsf: c_ulonglong, mem: c_ulonglong, bytes: c_ulonglong) -> c_ulonglong;

    pub fn tlsf_malloc(tlsf: c_ulonglong, bytes: c_ulonglong) -> c_ulonglong;//申请一段内存
    pub fn tlsf_memalign(tlsf: c_ulonglong, align: c_ulonglong, bytes: c_ulonglong) -> c_ulonglong;//申请一段内存，要求对齐到align
    pub fn tlsf_free(tlsf: c_ulonglong, ptr: c_ulonglong);//回收
}  

pub struct TlsfCAllocator {
    inner: Option<c_ulonglong>,
}

impl TlsfCAllocator {
    pub const fn new() -> Self {
        Self { inner: None }
    }
    fn inner_mut(&mut self) -> &mut c_ulonglong {
        self.inner.as_mut().unwrap()
    }

    fn inner(&self) -> &c_ulonglong {
        self.inner.as_ref().unwrap()
    }
}

impl BaseAllocator for TlsfCAllocator {
    fn init(&mut self, start: usize, size: usize){
        //log::debug!("init: start = {:#x}, size = {:#?}",start, size);
        unsafe{ 
            self.inner = Some(tlsf_create_with_pool(start as c_ulonglong,size as c_ulonglong) as c_ulonglong);
        }
    }

    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        unsafe {
            tlsf_add_pool(*self.inner() as c_ulonglong,start as c_ulonglong,size as c_ulonglong);
        }
        Ok(())
    }
}

impl ByteAllocator for TlsfCAllocator {
    fn alloc(&mut self, size: usize, align_pow2: usize) -> AllocResult<usize> {
        if align_pow2 <= 8 {
            unsafe {
                let ptr = tlsf_malloc(*self.inner() as c_ulonglong,size as c_ulonglong) as usize;
                if ptr == 0 {
                    panic!("alloc err.");
                }
                Ok(ptr)
            }
        } else {
            unsafe {
                let ptr = tlsf_memalign(*self.inner() as c_ulonglong,align_pow2 as c_ulonglong, size as c_ulonglong) as usize;
                if ptr == 0 {
                    panic!("alloc err.");
                }
                Ok(ptr)
            }
        }
    }

    fn dealloc(&mut self, pos: usize, size: usize, align_pow2: usize) {
        unsafe {
            tlsf_free(*self.inner() as c_ulonglong,pos as c_ulonglong);
        }
    }

    fn total_bytes(&self) -> usize {
        0
    }

    fn used_bytes(&self) -> usize {
        0
    }

    fn available_bytes(&self) -> usize {
        0
    }
}
