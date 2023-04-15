#![feature(allocator_api)]
#![no_std]

extern crate alloc;

use alloc::alloc::{AllocError, Layout};
use core::mem::size_of;
pub mod linked_list;
pub use linked_list::{LinkedList, MemBlockNode, ListNode};
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
}

/// 获取一个地址加上一个MemBlockNode大小后对齐到align的结果
/// 用一个MemBlockNode的空间记录块头,再用一个usize的空间记录块头大小(因为要考虑到offset)
fn get_aligned(addr: usize, align: usize) -> usize {
    (addr + size_of::<MemBlockNode>() + size_of::<usize>() + align - 1) / align * align
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
        }
    }


    /// Adds memory to the heap. The start address must be valid
    /// and the memory in the `[mem_start_addr, mem_start_addr + heap_size)` range must not be used for
    /// anything else.
    /// In case of linked list allocator the memory can only be extended.
    /// # Safety
    /// This function is unsafe because it can cause undefined behavior if the
    /// given address is invalid.
    pub unsafe fn add_memory(&mut self, heap_start_addr: usize, heap_size: usize) {
        assert!(
            heap_start_addr % 4096 == 0,
            "Start address should be page aligned"
        );
        assert!(
            heap_size % 4096 == 0 && heap_size > 0,
            "Add Heap size should be a multiple of page size"
        );

        self.push_mem_block(heap_start_addr, heap_size);
        self.total += heap_size;
        //self.debug_memblock();
    }

    /// fitst fit策略
    pub unsafe fn first_fit(&mut self, size: usize, align: usize) -> Option<ListNode> {
        for block in self.free_list.iter_mut() {
            let addr = block.value() as usize;
            let bsize = block.size();
            if addr + bsize >= get_aligned(addr, align) + size {
                return Some(block);
            }
        }
        None
    }

    /// best fit策略
    pub unsafe fn best_fit(&mut self, size: usize, align: usize) -> Option<ListNode> {
        let mut res: Option<ListNode> = None;
        let mut now_size: usize = 0;
        for block in self.free_list.iter_mut() {
            let addr = block.value() as usize;
            let bsize = block.size();
            let addr_left = addr + bsize - get_aligned(addr, align) - size;
            if addr + bsize >= get_aligned(addr, align) + size {
                if res.is_none() || addr_left < now_size {
                    now_size = addr_left;
                    res = Some(block);
                }
            }
        }
        res
    }

    /// worst fit策略
    pub unsafe fn worst_fit(&mut self, size: usize, align: usize) -> Option<ListNode> {
        let mut res: Option<ListNode> = None;
        let mut now_size: usize = 0;
        for block in self.free_list.iter_mut() {
            let addr = block.value() as usize;
            let bsize = block.size();
            let addr_left = addr + bsize - get_aligned(addr, align) - size;
            if addr + bsize >= get_aligned(addr, align) + size {
                if res.is_none() || addr_left > now_size {
                    now_size = addr_left;
                    res = Some(block);
                }
            }
        }
        res
    }

    /// Allocates a chunk of the given size with the given alignment. Returns a pointer to the
    /// beginning of that chunk if it was successful. Else it returns `Err`.
    /// This function finds the slab of lowest size which can still accomodate the given chunk.
    /// The runtime is in `O(1)` for chunks of size <= 4096, and `O(n)` when chunk size is > 4096,
    pub fn allocate(&mut self, layout: Layout) -> Result<usize, AllocError> {
        let size = max(
            layout.size(),
            max(layout.align(), size_of::<usize>()),
        );
        unsafe{
            let block = match self.strategy {
                BasicAllocatorStrategy::FirstFitStrategy => self.first_fit(size, layout.align()),
                BasicAllocatorStrategy::BestFitStrategy => self.best_fit(size, layout.align()),
                BasicAllocatorStrategy::WorstFitStrategy => self.worst_fit(size, layout.align()),
            };
            match block {
                Some(mut inner) => {
                    let res = inner.value() as usize;
                    let block_size = inner.size();
                    //地址对齐
                    let addr = get_aligned(res,layout.align());
                    let addr_left = res + block_size - addr - size;
                    if addr_left > size_of::<MemBlockNode>() + size_of::<usize>() {//还能切出去更小的块
                        inner.change_size(block_size - addr_left);
                        inner.pop(&mut self.free_list);
                        self.user += layout.size();
                        self.allocated += block_size - addr_left;
                        //一定不会merge
                        self.free_list.push((addr + size) as *mut MemBlockNode, addr_left);
                    } else {
                        self.user += layout.size();
                        self.allocated += block_size;
                        inner.pop(&mut self.free_list);
                    }

                    //在前一个位置写入头部大小
                    *((addr - size_of::<usize>()) as *mut usize) = addr - res;
                    //self.debug_memblock();
                    Ok(addr)
                },
                None => Err(AllocError),
            }
        }
    }

    /// push a memblock to linked list
    /// before push,need to check merge
    pub unsafe fn push_mem_block(&mut self, addr: usize, size: usize) {
        let mut now_addr = addr;
        let mut now_size = size;
        match self.strategy{
            //first fit策略不合并相邻块
            BasicAllocatorStrategy::FirstFitStrategy => {self.free_list.push(now_addr as *mut MemBlockNode, now_size);}
            _ => {
                //let mut cnt = 0;
                //先找:是否有一个块紧贴在它后面
                for block in self.free_list.iter_mut() {
                    let baddr = block.value() as usize;
                    let bsize = block.size();
                    //cnt += 1;
                    if now_addr + now_size == baddr {
                        now_size += bsize;
                        block.pop(&mut self.free_list);
                        break;
                    }
                }
                
                //再找:是否有一个块紧贴在它前面
                for block in self.free_list.iter_mut() {
                    let baddr = block.value() as usize;
                    let bsize = block.size();
                    //cnt += 1;
                    if baddr + bsize == now_addr{
                        now_addr = baddr;
                        now_size += bsize;
                        block.pop(&mut self.free_list);
                        break;
                    }
                }
                //log::debug!("{:#?}",cnt);
                self.free_list.push(now_addr as *mut MemBlockNode, now_size);
            }
        }
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
        //log::debug!("deallocate: ptr = {:#x}, size = {:#?}",ptr,layout.size());
        let size = max(
            layout.size(),
            max(layout.align(), size_of::<usize>()),
        );
        let head_size = *((ptr - size_of::<usize>()) as *mut usize);
        let block_begin = ptr - head_size;
        let mem_block = block_begin as *mut MemBlockNode;
        let block_size = (*mem_block).size;
        assert!(block_size >= size, "Dealloc error");
        self.user -= layout.size();
        self.allocated -= (*mem_block).size;
        self.push_mem_block(block_begin, block_size);

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
        //log::debug!("mem debug begin******************");
        let mut cnt = 0;
        for block in self.free_list.iter_mut() {
            let baddr = block.value() as usize;
            let bsize = block.size();
            cnt = cnt + 1;
            //log::debug!("mem block: baddr = {:#x}, bsize = {:#?}, nxt = {:#x}",baddr,bsize,(*(baddr as *mut MemBlockNode)).nxt as usize);
        }
        log::debug!("mem debug end: cnt = {:#?}, total = {:#?}, used = {:#?}, available = {:#?}"
            ,cnt,self.total_bytes(),self.used_bytes(),self.available_bytes());
    }

}
