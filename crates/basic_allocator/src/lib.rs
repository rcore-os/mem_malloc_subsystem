#![feature(allocator_api)]
#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use alloc::alloc::{AllocError, Layout};
use core::mem::size_of;
pub mod linked_list;
pub use linked_list::{LinkedList, MemBlockFoot, MemBlockHead};
use core::cmp::{max};

pub enum BasicAllocatorStrategy {
    FirstFitStrategy,
    BestFitStrategy,
    WorstFitStrategy,
}

pub struct Heap {
    free_list: LinkedList,
    user: usize, //分配给用户的内存大小
    allocated: usize, //实际分配出去的内存大小
    total: usize, //总内存大小
    strategy: BasicAllocatorStrategy, //使用的内存分配策略
    begin_addr: usize, //堆区起始地址
    end_addr: usize, //堆区结束地址

    //处理kernel page table，对此的申请是不经过这里的，这会形如在堆空间中挖了一个洞
    kernel_begin: Vec<usize>,
    kernel_end: Vec<usize>,
}

/// 获取一个地址加上一个usize(分配出去的块的头)大小后对齐到align的结果
fn get_aligned(addr: usize, align: usize) -> usize {
    (addr + size_of::<usize>() + align - 1) / align * align
}

/// 获取一个size对齐到align的结果
fn alignto(size: usize, align: usize) -> usize {
    (size + align - 1) / align * align
}

impl Heap {
    /// Create an empty heap
    pub const fn new() -> Self {
        Heap {
            free_list: LinkedList::new(),
            user: 0,
            allocated: 0,
            total: 0,

            //strategy: BasicAllocatorStrategy::FirstFitStrategy, //默认为first fit
            strategy: BasicAllocatorStrategy::BestFitStrategy, //默认为best fit
            //strategy: BasicAllocatorStrategy::WorstFitStrategy, //默认为worst fit

            begin_addr: 0,
            end_addr: 0,

            kernel_begin: Vec::new(),
            kernel_end: Vec::new(),
        }
    }

    ///init
    pub unsafe fn init(&mut self, heap_start_addr: usize, heap_size: usize) {
        assert!(
            heap_start_addr % 4096 == 0,
            "Start address should be page aligned"
        );
        assert!(
            heap_size % 4096 == 0 && heap_size > 0,
            "Add Heap size should be a multiple of page size"
        );
        self.begin_addr = heap_start_addr;
        self.end_addr = heap_start_addr + heap_size;
        self.push_mem_block(heap_start_addr, heap_size);
        self.total = heap_size;
        //self.debug_memblock();
    }

    /// Adds memory to the heap. The start address must be valid
    /// and the memory in the `[mem_start_addr, mem_start_addr + heap_size)` range must not be used for
    /// anything else.
    /// In case of linked list allocator the memory can only be extended.
    /// # Safety
    /// This function is unsafe because it can cause undefined behavior if the
    /// given address is invalid.
    pub unsafe fn add_memory(&mut self, start_addr: usize, heap_size: usize) {
        //log::debug!("begin addr: {:#x}, end addr: {:#x}",self.begin_addr,self.end_addr);
        if start_addr != self.end_addr{
            //assert!(self.kernel_begin == 0 && self.kernel_end == 0,"Kernel page table error");
            self.kernel_begin.push(self.end_addr);
            self.kernel_end.push(start_addr);
        }
        assert!(
            start_addr % 4096 == 0,
            "Start address should be page aligned"
        );
        assert!(
            heap_size % 4096 == 0 && heap_size > 0,
            "Add Heap size should be a multiple of page size"
        );
        self.end_addr = start_addr + heap_size;
        self.push_mem_block(start_addr, heap_size);
        self.total += heap_size;
        //self.debug_memblock();
    }

    /// fitst fit策略
    pub unsafe fn first_fit(&mut self, size: usize, align: usize) -> Option<*mut MemBlockHead> {
        let mut block = self.free_list.head;
        while !block.is_null(){
            let addr = block as usize;
            let bsize = (*block).size();
            if addr + bsize >= get_aligned(addr, align) + size + size_of::<usize>(){
                return Some(block);
            }
            block = (*block).nxt;
        }
        None
    }

    /// best fit策略
    pub unsafe fn best_fit(&mut self, size: usize, align: usize) -> Option<*mut MemBlockHead> {
        let mut res: Option<*mut MemBlockHead> = None;
        let mut now_size: usize = 0;
        let mut block = self.free_list.head;
        while !block.is_null(){
            let addr = block as usize;
            let bsize = (*block).size();
            let addr_left = addr + bsize - get_aligned(addr, align) - size - size_of::<usize>();
            if addr + bsize >= get_aligned(addr, align) + size + size_of::<usize>(){
                if res.is_none() || addr_left < now_size {
                    now_size = addr_left;
                    res = Some(block);
                }
            }
            block = (*block).nxt;
        }
        res
    }

    /// worst fit策略
    pub unsafe fn worst_fit(&mut self, size: usize, align: usize) -> Option<*mut MemBlockHead> {
        let mut res: Option<*mut MemBlockHead> = None;
        let mut now_size: usize = 0;
        let mut block = self.free_list.head;
        while !block.is_null(){
            let addr = block as usize;
            let bsize = (*block).size();
            let addr_left = addr + bsize - get_aligned(addr, align) - size - size_of::<usize>(); 
            if addr + bsize >= get_aligned(addr, align) + size + size_of::<usize>(){
                if res.is_none() || addr_left > now_size {
                    now_size = addr_left;
                    res = Some(block);
                }
            }
            block = (*block).nxt;
        }
        res
    }

    /// Allocates a chunk of the given size with the given alignment. Returns a pointer to the
    /// beginning of that chunk if it was successful. Else it returns `Err`.
    /// This function finds the slab of lowest size which can still accomodate the given chunk.
    /// The runtime is in `O(1)` for chunks of size <= 4096, and `O(n)` when chunk size is > 4096,
    pub fn allocate(&mut self, layout: Layout) -> Result<usize, AllocError> {
        //log::debug!("qaq: {:#x}",self.free_list.head as usize);
        //let tmp: usize = 0xffffffc080276b90;log::debug!("{:#?}",(*(tmp as *mut MemBlockHead)).size());
        //单次分配最小16字节
        let size = alignto(max(
            layout.size(),
            max(layout.align(), 2 * size_of::<usize>()),
        ),size_of::<usize>());
        
        unsafe{
            let block = match self.strategy {
                BasicAllocatorStrategy::FirstFitStrategy => self.first_fit(size, layout.align()),
                BasicAllocatorStrategy::BestFitStrategy => self.best_fit(size, layout.align()),
                BasicAllocatorStrategy::WorstFitStrategy => self.worst_fit(size, layout.align()),
            };
            match block {
                Some(inner) => {
                    let res = inner as usize;
                    let block_size = (*inner).size();
                    //地址对齐
                    let addr = get_aligned(res,layout.align());
                    let addr_left = res + block_size - addr - size - size_of::<usize>();
                    //log::debug!("block_size: {:#?}, addr_left: {:#?}",block_size,addr_left);
                    if addr_left > 4 * size_of::<usize>() {//还能切出去更小的块
                        (*inner).set_size(block_size - addr_left);
                        (*inner).set_used(true);
                        //log::debug!("{:#x}***{:#?}",inner as usize,(*inner).size());
                        self.free_list.del(inner);
                        self.user += layout.size();
                        self.allocated += block_size - addr_left;
                        //一定不会merge
                        self.free_list.push((addr + size + size_of::<usize>()) as *mut MemBlockHead, addr_left);
                    } else {
                        (*inner).set_used(true);
                        self.user += layout.size();
                        self.allocated += block_size;
                        self.free_list.del(inner);
                    }

                    //self.debug_memblock();
                    //let tmp: usize = 0xffffffc080276b90;log::debug!("{:#?}",(*(tmp as *mut MemBlockHead)).size());
                    Ok(addr)
                },
                None => Err(AllocError),
            }
        }
    }

    /// push a memblock to linked list
    /// before push,need to check merge
    pub unsafe fn push_mem_block(&mut self, addr: usize, size: usize) {
        //log::debug!("qaq: {:#x}",self.free_list.head as usize);
        let mut now_addr = addr;
        let mut now_size = size;

        //log::debug!("1:::now_addr: {:#x}, now_size: {:#?}",now_addr,now_size);

        //先找:是否有一个块紧贴在它后面
        if now_addr + now_size != self.end_addr && !self.kernel_begin.contains(&(now_addr + now_size)) {
            let nxt_block = (now_addr + now_size) as *mut MemBlockHead;
            //log::debug!("end_addr: {:#x}, nxt_block: {:#x}",self.end_addr,nxt_block as usize);
            if !(*nxt_block).used() {
                now_size += (*nxt_block).size();
                //log::debug!("***{:#?}",(*nxt_block).size());
                self.free_list.del(nxt_block);
            }
        }
        //let tmp: usize = 0xffffffc080276b90;log::debug!("{:#?}",(*(tmp as *mut MemBlockHead)).size());

        //log::debug!("2:::now_addr: {:#x}, now_size: {:#?}",now_addr,now_size);

        //再找:是否有一个块紧贴在它前面
        if now_addr != self.begin_addr && !self.kernel_end.contains(&(now_addr)) {
            let pre_block = (*((now_addr - size_of::<usize>()) as *mut MemBlockFoot)).get_head();
            //log::debug!("end_addr: {:#x}, pre_block: {:#x}",self.end_addr,pre_block as usize);
            if !(*pre_block).used() {
                now_addr = pre_block as usize;
                now_size += (*pre_block).size();
                self.free_list.del(pre_block);
            }
        }

        //log::debug!("3:::now_addr: {:#x}, now_size: {:#?}, head: {:#x}",now_addr,now_size,self.free_list.head as usize);
        //let tmp: usize = 0xffffffc080276b90;log::debug!("{:#?}",(*(tmp as *mut MemBlockHead)).size());
        self.free_list.push(now_addr as *mut MemBlockHead, now_size);
        //log::debug!("qaq: {:#x}",self.free_list.head as usize);
        //let tmp: usize = 0xffffffc080276b90;log::debug!("{:#?}",(*(tmp as *mut MemBlockHead)).size());
    }  

    /// Frees the given allocation. `ptr` must be a pointer returned
    /// by a call to the `allocate` function with identical size and alignment. Undefined
    /// behavior may occur for invalid arguments, thus this function is unsafe.
    ///
    /// This function finds the slab which contains address of `ptr` and adds the blocks beginning
    /// with `ptr` address to the list of free blocks.
    /// This operation is in `O(1)` for blocks <= 4096 bytes and `O(n)` for blocks > 4096 bytes.
    ///
    /// # Safety
    /// This function is unsafe because it can cause undefined behavior if the
    /// given address is invalid.
    pub unsafe fn deallocate(&mut self, ptr: usize, layout: Layout) {
        //let tmp: usize = 0xffffffc080276b90;log::debug!("{:#?}",(*(tmp as *mut MemBlockHead)).size());
        //log::debug!("deallocate: ptr = {:#x}, size = {:#?}",ptr,layout.size());
        //log::debug!("qaq: {:#x}",self.free_list.head as usize);
        let size = alignto(max(
            layout.size(),
            max(layout.align(), 2 * size_of::<usize>()),
        ),size_of::<usize>());
        let block = (ptr - size_of::<usize>()) as *mut MemBlockHead;
        let block_size = (*block).size();
        //log::debug!("ptr: {:#x}, block: {:#x}, size: {:#?}, block_size: {:#?}",ptr as usize,block as usize,size,block_size);
        assert!(block_size >= size, "Dealloc error");
        self.user -= layout.size();
        self.allocated -= block_size;
        //log::debug!("***");
        //let tmp: usize = 0xffffffc080276b90;log::debug!("{:#?}",(*(tmp as *mut MemBlockHead)).size());
        self.push_mem_block(block as usize, block_size);
        //let tmp: usize = 0xffffffc080276b90;log::debug!("{:#?}",(*(tmp as *mut MemBlockHead)).size());
        //self.debug_memblock();
    }

    pub fn total_bytes(&self) -> usize {
        self.total
    }

    pub fn used_bytes(&self) -> usize {
        self.user
    }

    pub fn available_bytes(&self) -> usize {
        self.total - self.allocated
    }

    pub unsafe fn debug_memblock(&mut self) {
        //let tmp: usize = 0xffffffc080276b90;log::debug!("{:#?}",(*(tmp as *mut MemBlockHead)).size());
        log::debug!("mem debug begin: {:#x}",self.free_list.head as usize);
        log::debug!("begin addr: {:#x}, end addr: {:#x}",self.begin_addr,self.end_addr);
        //log::debug!("kernel begin: {:#x}, kernel end: {:#x}",self.kernel_begin,self.kernel_end);
        let mut cnt = 0;
        let mut block = self.free_list.head;
        while !block.is_null(){
            let addr = (*block).addr() as usize;
            let size = (*block).size();
            let used = (*block).used();
            cnt = cnt + 1;
            log::debug!("mem block: addr = {:#x}, size = {:#?}, used = {:#?}, nxt = {:#x}",addr,size,used,(*block).nxt as usize);
            block = (*block).nxt;
        }
        log::debug!("mem debug end: cnt = {:#?}, total = {:#?}, used = {:#?}, available = {:#?}"
            ,cnt,self.total_bytes(),self.used_bytes(),self.available_bytes());
    }

}
