//! Mimalloc(single thread) for `no_std` systems.
//! written by rust code

#![feature(allocator_api)]
#![no_std]

extern crate alloc;

use alloc::alloc::{AllocError, Layout};
use core::cmp::max;
use core::mem::size_of;

mod data;
use data::*;

/// the heap structure of the allocator
pub struct Heap {
    // 指向heap的地址
    pub addr: usize, 
    // 尚未建成段的起始地址
    pub unused_begin: usize,
    // 尚未建成段的终止地址
    pub unused_end: usize,
    // 一个临时的尚未建成段的起始地址，为建立huge segment而暂存
    pub unused_begin_tmp: usize,
    // 一个临时的尚未建成段的终止地址，为建立huge segment而暂存
    pub unused_end_tmp: usize,
}

unsafe impl Send for Heap {}

impl Heap {
    /// Create an empty heap
    pub const fn new() -> Self {
        Heap {
            addr: 0,
            unused_begin: 0,
            unused_end: 0,
            unused_begin_tmp: 0,
            unused_end_tmp: 0,
        }
    }

    /// get reference 
    pub fn get_ref(&self) -> &MiHeap {
        unsafe { &(*(self.addr as *const MiHeap)) }
    }
    /// get mut reference
    pub fn get_mut_ref(&mut self) -> &mut MiHeap {
        unsafe { &mut (*(self.addr as *mut MiHeap)) }
    }

    /// init
    /// 需要保证heap_start_addr是4MB对齐，heap_size是4MB的倍数
    /// # Safety
    /// This function is unsafe because it can cause undefined behavior if the
    /// given address is invalid.
    pub fn init(&mut self, heap_start_addr: usize, heap_size: usize) {
        assert!(
            heap_start_addr % MIN_SEGMENT_SIZE == 0,
            "Start address should be 4MB aligned"
        );
        assert!(
            heap_size % MIN_SEGMENT_SIZE == 0 && heap_size > 0,
            "Add Heap size should be a multiple of 4MB"
        );
        self.addr = heap_start_addr;
        self.unused_begin = heap_start_addr;
        self.unused_end = heap_start_addr + heap_size;
        self.unused_begin_tmp = 0;
        self.unused_end_tmp = 0;
        self.get_mut_ref().init();
    }

    /// 新建一个small类型的segment，并将其中的page塞入heap的free链表中
    /// 从unused_begin中取4MB内存，如果不够则返回false
    pub fn create_small_segment(&mut self) -> bool{
        if self.unused_begin == self.unused_end{
            return false;
        }
        let mut seg_addr = SegmentPointer{
            addr: self.unused_begin,
        };
        let seg = seg_addr.get_mut_ref();
        seg.init(self.unused_begin, MIN_SEGMENT_SIZE, PageKind::SmallPage);
        for i in 0..seg.num_pages{
            let page_addr = PagePointer{
                addr: &seg.pages[i] as *const Page as usize,
            };
            self.get_mut_ref().add_small_page(page_addr);
        }
        self.unused_begin += MIN_SEGMENT_SIZE;
        true
    }

    /// 新建一个medium类型的segment，并将其中的page塞入heap的tmp_page
    /// 从unused_begin中取4MB内存，如果不够则返回false
    pub fn create_medium_segment(&mut self) -> bool{
        if self.unused_begin == self.unused_end{
            return false;
        }
        let mut seg_addr = SegmentPointer{
            addr: self.unused_begin,
        };
        let seg = seg_addr.get_mut_ref();
        seg.init(self.unused_begin, MIN_SEGMENT_SIZE, PageKind::MediumPage);
        let page_addr = PagePointer{
            addr: &seg.pages[0] as *const Page as usize,
        };
        self.get_mut_ref().tmp_page = page_addr;
        self.unused_begin += MIN_SEGMENT_SIZE;
        true
    }

    /// 新建一个huge类型的segment，并将其中的page塞入heap的tmp_page
    /// 优先从unused_begin_tmp中取内存
    /// 如果没有再从unused_begin中取内存
    /// 如果还没有则返回false
    pub fn create_huge_segment(&mut self,size: usize) -> bool{
        assert!(
            size % MIN_SEGMENT_SIZE == 0,
            "Huge segment size should be a multiple of 4MB"
        );
        let begin_addr;
        if self.unused_begin_tmp + size <= self.unused_end_tmp{
            begin_addr = self.unused_begin_tmp;
            self.unused_begin_tmp += size;
        }
        else if self.unused_begin + size <= self.unused_end{
            begin_addr = self.unused_begin;
            self.unused_begin += size;
        }
        else{
            return false;
        }
        let mut seg_addr = SegmentPointer{
            addr: begin_addr,
        };
        let seg = seg_addr.get_mut_ref();
        seg.init(begin_addr, MIN_SEGMENT_SIZE, PageKind::HugePage);
        let page_addr = PagePointer{
            addr: &seg.pages[0] as *const Page as usize,
        };
        self.get_mut_ref().tmp_page = page_addr;
        true
    }

    /// Adds memory to the heap. The start address must be valid
    /// 需要保证heap_start_addr是4MB对齐，heap_size是4MB的倍数
    /// and the memory in the `[mem_start_addr, mem_start_addr + heap_size)` range must not be used for
    /// anything else.
    /// In case of linked list allocator the memory can only be extended.
    /// # Safety
    /// This function is unsafe because it can cause undefined behavior if the
    /// given address is invalid.
    pub fn add_memory(&mut self, start_addr: usize, heap_size: usize) {
        assert!(
            start_addr % MIN_SEGMENT_SIZE == 0,
            "Start address should be 4MB aligned"
        );
        assert!(
            heap_size % MIN_SEGMENT_SIZE == 0 && heap_size > 0,
            "Add Heap size should be a multiple of 4MB"
        );
        if self.unused_begin == self.unused_end{
            self.unused_begin = start_addr;
            self.unused_end = start_addr + heap_size;
        }
        else{
            self.unused_begin_tmp = start_addr;
            self.unused_end_tmp = start_addr + heap_size;
        }
    }

    /// Allocates a chunk of the given size with the given alignment. Returns a pointer to the
    /// beginning of that chunk if it was successful. Else it returns `Err`.
    pub fn allocate(&mut self, layout: Layout) -> Result<usize, AllocError> {
        //单次分配最小8字节
        assert!(
            my_lowbit(layout.align()) == layout.align(),
            "align should be power of 2."
        );
        let size = get_upper_size(alignto(
            max(layout.size(), max(layout.align(), size_of::<usize>())),
            max(layout.align(), size_of::<usize>()),
        ));

        assert!(
            layout.align() <= size_of::<usize>(),
            "align should be not greater than 8."
        );


        let idx = get_queue_id(size);
        // 找一个page
        let mut page = self.get_mut_ref().get_page(idx,size);
        // 如果没找到
        if page.addr == 0{
            let pagetype;
            if size < SMALL_PAGE_SIZE{
                pagetype = PageKind::SmallPage;
            }
            else if size < MEDIUM_PAGE_SIZE{
                pagetype = PageKind::MediumPage;
            }
            else{
                pagetype = PageKind::HugePage;
            }
            // 找一个没分配的
            match pagetype{
                PageKind::SmallPage => {page = self.get_mut_ref().pages[FREE_SMALL_PAGE_QUEUE];}
                _ => {page = self.get_mut_ref().tmp_page;} 
            }
            // 还找不到就寄了
            if page.addr == 0{
                return Err(AllocError);
            }
            
            page.get_mut_ref().block_size = size;
            match pagetype{
                PageKind::SmallPage => {self.get_mut_ref().del_small_page(page);}
                _ => {self.get_mut_ref().tmp_page = PagePointer{
                    addr: 0,
                };} 
            }
            self.get_mut_ref().insert_to_list(idx, page);
        }

        // 获取一个block
        let addr = page.get_mut_ref().get_block();

        // 如果这个块从不满变为满，要塞进full queue里
        if page.get_ref().is_full(){
            self.get_mut_ref().delete_from_list(idx, page);
            self.get_mut_ref().add_full_page(page);
        }

        Ok(addr)
    }

    

    /// Frees the given allocation. `ptr` must be a pointer returned
    /// by a call to the `allocate` function with identical size and alignment. Undefined
    /// behavior may occur for invalid arguments, thus this function is unsafe.
    ///
    /// # Safety
    /// This function is unsafe because it can cause undefined behavior if the
    /// given address is invalid.
    pub fn deallocate(&mut self, ptr: usize, layout: Layout) {
        assert!(
            my_lowbit(layout.align()) == layout.align(),
            "align should be power of 2."
        );
        let size = get_upper_size(alignto(
            max(layout.size(), max(layout.align(), size_of::<usize>())),
            max(layout.align(), size_of::<usize>()),
        ));

        let idx = get_queue_id(size);
        
        // 先找到这个块所在的页
        let mut page = get_page(ptr);
        page.get_mut_ref().push_front(BlockPointer{
            addr: ptr,
        });

        //如果这个块从满变为不满，要塞回原来的queue
        if !page.get_ref().is_full(){
            self.get_mut_ref().del_full_page(page);
            self.get_mut_ref().insert_to_list(idx, page);
        }
    }

    /// get total bytes
    pub fn total_bytes(&self) -> usize {
        0
    }
    /// get used bytes
    pub fn used_bytes(&self) -> usize {
        0
    }
    /// get available bytes
    pub fn available_bytes(&self) -> usize {
        0
    }
}
