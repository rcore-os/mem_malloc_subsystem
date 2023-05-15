#![feature(allocator_api)]
#![no_std]

extern crate alloc;

use alloc::alloc::{AllocError, Layout};
use core::mem::size_of;
use core::cmp::max;

mod data;
use data::*;

pub struct Heap {
    head: AddrPointer,
    total_mem: usize, // 总共占用内存
    used_mem: usize, // 已经分配出去的内存
    avail_mem: usize, // 实际可用的内存
}

unsafe impl Send for Heap {}

impl Heap {
    /// Create an empty heap
    pub const fn new() -> Self {
        Heap {
            head: AddrPointer{addr: 0},
            total_mem: 0,
            used_mem: 0,
            avail_mem: 0,
        }
    }

    ///init
    pub fn init(&mut self, heap_start_addr: usize, heap_size: usize) {
        //log::debug!("TLSF: init addr = {:#x}, size = {:#x}",heap_start_addr,heap_size);
        assert!(
            heap_start_addr % 4096 == 0,
            "Start address should be page aligned"
        );
        assert!(
            heap_size % 4096 == 0 && heap_size > 0,
            "Add Heap size should be a multiple of page size"
        );
        self.head = get_addr_pointer(heap_start_addr);
        self.head.init_controller(heap_start_addr, heap_size);
        self.total_mem = heap_size;
        self.used_mem = 0;
        self.avail_mem = heap_size - alignto(size_of::<Controller>(), 8) - 6 * size_of::<usize>();
    }

    /// Adds memory to the heap. The start address must be valid
    /// and the memory in the `[mem_start_addr, mem_start_addr + heap_size)` range must not be used for
    /// anything else.
    /// In case of linked list allocator the memory can only be extended.
    /// # Safety
    /// This function is unsafe because it can cause undefined behavior if the
    /// given address is invalid.
    pub fn add_memory(&mut self, start_addr: usize, heap_size: usize) {
        //log::debug!("begin addr: {:#x}, end addr: {:#x}, size: {:#?}",start_addr,start_addr + heap_size,heap_size);
        assert!(
            start_addr % 4096 == 0,
            "Start address should be page aligned"
        );
        assert!(
            heap_size % 4096 == 0 && heap_size > 0,
            "Add Heap size should be a multiple of page size"
        );
        self.head.add_memory(start_addr, heap_size);
        self.total_mem += heap_size;
        self.avail_mem += heap_size - 6 * size_of::<usize>();
    }

    /// Allocates a chunk of the given size with the given alignment. Returns a pointer to the
    /// beginning of that chunk if it was successful. Else it returns `Err`.
    pub fn allocate(&mut self, layout: Layout) -> Result<usize, AllocError> {
        //log::debug!("TLSF: allocate: size = {:#?}",layout.size());
        //单次分配最小16字节
        assert!(my_lowbit(layout.align()) == layout.align(),"align should be power of 2.");
        let mut size = alignto(max(
            layout.size(),
            max(layout.align(), 2 * size_of::<usize>()),
        ),max(layout.align(),size_of::<usize>()));

        //处理align更大的分配请求
        if layout.align() > size_of::<usize>(){
            size = alignto(size + layout.align() + 4 * size_of::<usize>(),layout.align());
            //给size加上足够的大小，使得切出来的块的头部可以分裂成一个新的块
        }

        let mut block = self.head.find_block(size);
        if !(block.is_null()){
            let mut nsize = block.get_size();
            assert!(nsize >= size,"Alloc error.");
            let mut addr = block.addr + 2 * size_of::<usize>();
            //log::debug!("*** {:#x} {:#?} {:#x}",block as usize, nsize, get_block_phy_next(block) as usize);

            //处理align更大的分配请求
            if layout.align() > size_of::<usize>(){
                let mut new_addr = alignto(addr,layout.align());
                if new_addr != addr{//要切出头部单独组成一块
                    while new_addr - block.addr < 6 * size_of::<usize>(){
                        //切出的头部不足以构成一个新块，于是把头部再扩大一个align
                        //因为new_addr是实际分配出去的起始地址，因此到原来块的开头至少要48个字节才能让中间再拆出一个块
                        new_addr += layout.align();
                    }
                    //创造一个新的块pre_block
                    let pre_block = block;
                    let nxt_block = get_block_phy_next(block);
                    block = get_addr_pointer(new_addr - 2 * size_of::<usize>());
                    //设置物理上的前一块
                    block.set_prev_phy_pointer(pre_block);
                    if !(nxt_block.is_null()){
                        nxt_block.set_prev_phy_pointer(block);
                    }
                    //设置块大小
                    let pre_size = block.addr - addr;
                    nsize -= pre_size + 2 * size_of::<usize>();
                    pre_block.set_size(pre_size);
                    block.set_size(nsize);
                    //设置使用状态
                    pre_block.set_free();
                    //插回到链表中去
                    self.head.add_into_list(pre_block);
                    self.avail_mem -= 2 * size_of::<usize>();
                    addr = new_addr;
                    //log::debug!("split head: {:#x} {:#x}, size = {:#?} {:#?}, next_free = {:#x}"
                    //    , pre_block as usize, block as usize, pre_size, nsize, (*pre_block).next_free as usize);
                }

                //把size改回来，这里的size就是实际分配出去的大小了
                size = alignto(max(
                    layout.size(),
                    max(layout.align(), 2 * size_of::<usize>()),
                ),layout.align());
                assert!(nsize >= size,"Alloc error.");
            }
            block.set_used();
            
            //把块的尾部拆分之后扔回去
            if nsize >= size + 4 * size_of::<usize>(){//最小32字节才能切出一个新块
                //新块
                let new_block = get_addr_pointer(addr + size);
                new_block.set_prev_phy_pointer(block);
                //原块的下一个块
                let nxt_block = get_block_phy_next(block);
                if !(nxt_block.is_null()){
                    nxt_block.set_prev_phy_pointer(new_block);
                }
                //设置块大小
                block.set_size(size);
                new_block.set_size(nsize - size - 2 * size_of::<usize>());//别忘了减去新块的头部大小
                //设置使用状态
                block.set_used();
                new_block.set_free();
                //插回到链表中去
                self.head.add_into_list(new_block);
                self.avail_mem -= 2 * size_of::<usize>();
                //log::debug!("new block = {:#x}, size = {:#?}",new_block as usize,(*new_block).get_size());
            }
            self.used_mem += layout.size();
            self.avail_mem -= block.get_size();
            //log::debug!("TLSF: successfully allocate: {:#x} {:#?}, pre = {:#x}, nxt = {:#x}, nxt nxt free = {:#x}"
            //    ,addr,(*block).get_size(),get_block_phy_prev(block) as usize,get_block_phy_next(block) as usize
            //    ,(*get_block_phy_next(block)).next_free as usize);
            return Ok(addr);
        }
        else{
            return Err(AllocError);
        }
    }


    /// 把这个块和物理上后一个块合并，要求两个块都是空闲的，且已经从链表中摘下来了
    pub fn merge_block(&self, block: AddrPointer){
        //log::debug!("TLSF: merge_block {:#x}",block as usize);
        let nxt = get_block_phy_next(block);
        //改block的size
        let size = block.get_size();
        let nsize = nxt.get_size();
        //log::debug!("{:#x} {:#x} {:#?} {:#?}",block as usize, nxt as usize, size, nsize);
        block.set_size(size + nsize + 2 * size_of::<usize>());
        //改block.nxt.nxt的pre指针为block自己
        let nnxt = get_block_phy_next(nxt);
        //log::debug!("{:#x}",nnxt as usize);
        if !(nnxt.is_null()){
            nnxt.set_prev_phy_pointer(block);
        }
    }
     

    

    /// Frees the given allocation. `ptr` must be a pointer returned
    /// by a call to the `allocate` function with identical size and alignment. Undefined
    /// behavior may occur for invalid arguments, thus this function is unsafe.
    ///
    /// # Safety
    /// This function is unsafe because it can cause undefined behavior if the
    /// given address is invalid.
    pub fn deallocate(&mut self, ptr: usize, layout: Layout) {
        //log::debug!("TLSF: deallocate: ptr = {:#x}, size = {:#?}",ptr,layout.size());
        //log::debug!("qaq: {:#x}",self.free_list.head as usize);
        assert!(my_lowbit(layout.align()) == layout.align(),"align should be power of 2.");
        let size = alignto(max(
            layout.size(),
            max(layout.align(), 2 * size_of::<usize>()),
        ),max(layout.align(),size_of::<usize>()));
        let block = get_addr_pointer(ptr - 2 * size_of::<usize>());
        let block_size = block.get_size();
        //log::debug!("block = {:#x}, size = {:#?}, block_size = {:#?}",block as usize,size,block_size);
        assert!(block_size >= size && block.get_now_free() == false, "Dealloc error");
        block.set_free();
        self.used_mem -= layout.size();
        self.avail_mem += block_size;
        
        //把这个块与前后的块合并
        let mut nblock = block;
        let pre = get_block_phy_prev(block);
        let nxt = get_block_phy_next(block);
        //log::debug!("TLSF: dealloc block = {:#x}, pre = {:#x}, nxt = {:#x}",block as usize, pre as usize, nxt as usize);
        if !(nxt.is_null()) && nxt.get_now_free(){
            //如果物理上的下一个块不是null且是空闲的，就合并
            self.head.del_into_list(nxt);
            self.merge_block(nblock);
            self.avail_mem += 2 * size_of::<usize>();
        }
        if !pre.is_null() && pre.get_now_free(){
            //如果物理上的上一个块不是null且是空闲的，就合并
            self.head.del_into_list(pre);
            self.merge_block(pre);
            nblock = pre;
            self.avail_mem += 2 * size_of::<usize>();
        }
        //log::debug!("TLSF: dealloc nblock = {:#x}",nblock as usize);
        self.head.add_into_list(nblock);

        //log::debug!("TLSF: successfully deallocate.");
    }

    /// 查询内存使用情况
    pub fn total_bytes(&self) -> usize {
        self.total_mem
    }

    pub fn used_bytes(&self) -> usize {
        self.used_mem
    }

    pub fn available_bytes(&self) -> usize {
        self.avail_mem
    }
}
