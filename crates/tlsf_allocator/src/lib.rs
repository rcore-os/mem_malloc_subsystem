#![feature(allocator_api)]
#![no_std]

extern crate alloc;

use alloc::alloc::{AllocError, Layout};
use core::mem::size_of;
use core::ptr::null_mut;
use core::cmp::max;

/// tlsf块头结构
pub struct BlockHeader {
    pub prev_phy: *mut BlockHeader,//物理上的上一个块
    pub size: usize,//“净”块大小，一定是8对齐的，所以末两位可以标记物理上的这个块/上一个块是否free
    //以上16个字节是分配出去的块要占的头部大小

    pub prev_free: *mut BlockHeader,//free链表的上一个块
    pub next_free: *mut BlockHeader,//free链表的下一个块
}

impl BlockHeader {
    pub fn get_now_free(&self) -> bool {
        return (self.size & 1) == 1;
    }
    pub fn get_prev_free(&self) -> bool {
        return (self.size & 2) == 2;
    }
    pub fn set_now_free(&mut self) {
        self.size |= 1;
    }
    pub fn set_now_used(&mut self) {
        self.size &= !(1 as usize);
    }
    pub fn set_prev_free(&mut self) {
        self.size |= 2;
    }
    pub fn set_prev_used(&mut self) {
        self.size &= !(2 as usize);
    }
    
    ///设置以及判断一个块是否为null
    pub fn set_null(&mut self){
        self.size = 0;
        self.prev_free = self;
        self.next_free = self;
        self.prev_phy = self;
    }
    pub fn is_null(&self) -> bool{
        return self.size < 4;
    }

    pub fn get_size(&self) -> usize{
        return self.size & (!(3 as usize));
    }
    pub fn set_size(&mut self, size: usize){
        self.size = size | (self.size & 3);
    }

    ///设置这个块为used，除了要设置自己还要设置物理上的下一个块
    pub unsafe fn set_used(&mut self){
        let next = get_block_phy_next(self);
        self.set_now_used();
        if !((*next).is_null()){
            (*next).set_prev_used();
        }
    }
    ///设置这个块为free，除了要设置自己还要设置物理上的下一个块
    pub unsafe fn set_free(&mut self){
        let next = get_block_phy_next(self);
        self.set_now_free();
        if !((*next).is_null()){
            (*next).set_prev_free();
        }
    }


}

///获取一个块物理上的下一个块
pub unsafe fn get_block_phy_next(block: *mut BlockHeader) -> *mut BlockHeader{
    return ((block as usize) + (*block).get_size() + 2 * size_of::<usize>()) as *mut BlockHeader;
}
///获取一个块物理上的上一个块
pub unsafe fn get_block_phy_prev(block: *mut BlockHeader) -> *mut BlockHeader{
    return (*block).prev_phy;
}


const FL_INDEX_COUNT: usize = 28;
const SL_INDEX_COUNT: usize = 32;
const FL_INDEX_SHIFT: usize = 8;
const SMALL_BLOCK_SIZE: usize = 256;
//地址的后3位一定是0
//对于不足256的块，直接8对齐
//对于超过256的块，最高位表示一级链表，接下来5位表示二级链表


/// tlsf 控制头结构
pub struct Controller {
    pub block_null: BlockHeader,//空块

	/* Bitmaps for free lists. */
	pub fl_bitmap: usize,//一级链表的bitmap，标记每个一级链表是否非空
	pub sl_bitmap: [usize; FL_INDEX_COUNT],//二级链表的bitmap，标记每个二级链表是否非空

	/* Head of free lists. */
	pub blocks: [[*mut BlockHeader; SL_INDEX_COUNT]; FL_INDEX_COUNT],//二级链表结构
	//SL_INDEX_COUNT=32表示二级链表将一级链表的一个区间拆分成了32段，也就是要根据最高位后的5个二进制位来判断
}

/// lowbit
fn my_lowbit(x: usize) -> usize{
    return x & ((!x) + 1);
}

/// log2
fn my_log2(x: usize) -> usize{
    let mut ans = 0;
    let mut y = x;
    if (y >> 32) > 0{y = y >> 32;ans += 32;}
    if (y >> 16) > 0{y = y >> 16;ans += 16;}
    if (y >> 8) > 0{y = y >> 8;ans += 8;}
    if (y >> 4) > 0{y = y >> 4;ans += 4;}
    if (y >> 2) > 0{y = y >> 2;ans += 2;}
    if (y >> 1) > 0{ans += 1;}
    return ans;
}

/// log lowbit
fn my_log_lowbit(x: usize) -> usize{
    return my_log2(my_lowbit(x));
}

/// 获取一个size对齐到align的结果
fn alignto(size: usize, align: usize) -> usize {
    (size + align - 1) / align * align
}

pub struct ListIndex {
    pub fl: usize,
    pub sl: usize,
    pub size: usize,
}
/// 获取一个块对应的一级和二级链表
fn get_fl_and_sl(size: usize) -> ListIndex{
    if size < SMALL_BLOCK_SIZE{//小块
        return ListIndex{
            fl: 0,
            sl: size >> 3,
            size,
        };
    }
    else{
        let tmp = (my_log2(size)) - FL_INDEX_SHIFT + 1;
        return ListIndex{
            fl: tmp,
            sl: (size >> (tmp + 2)) & (SL_INDEX_COUNT - 1),
            size,
        };
    }
}

/// 给定二级链表，求其最小块大小
fn get_block_begin_size(fl: usize,sl: usize) -> usize{
    if fl == 0{
        return sl << 3;
    }
    return ((1 as usize) << (fl + FL_INDEX_SHIFT - 1)) + (sl << (fl + 2)) as usize;
}

/// 获取一个块向上对齐到一级链表的最小块大小
fn get_up_size(size: usize) -> usize{
    let mut nsize = size;
    if size < SMALL_BLOCK_SIZE{//小块
        return alignto(size, 8);
    }
    else{
        let linkidx = get_fl_and_sl(size);
        let fl = linkidx.fl;
        let sl = linkidx.sl;
        if get_block_begin_size(fl,sl) != size{
            nsize += (1 as usize) << (fl + 2);
        }
        return nsize;
    }
}

/// 获取一个块向上对齐到一级链表的最小块大小，以及相应的fl和sl
fn get_up_fl_and_sl(size: usize) -> ListIndex{
    let nsize = get_up_size(size);
    return get_fl_and_sl(nsize);
}

impl Controller{
    ///init
    pub unsafe fn init(&mut self, addr: usize, size: usize){
        self.block_null.set_null();
        self.fl_bitmap = 0;
        for i in 0..FL_INDEX_COUNT{
            self.sl_bitmap[i] = 0;
        }
        for i in 0..FL_INDEX_COUNT{
            for j in 0..SL_INDEX_COUNT{
                self.blocks[i][j] = &mut self.block_null;
            }
        }
        // 把剩余的空间用于添加内存
        let offset = alignto(size_of::<Controller>(), 8);
        self.add_memory(addr + offset, size - offset);
    }

    /// add memory
    /// addr和size都应该是8对齐的
    pub unsafe fn add_memory(&mut self, addr: usize, size: usize){
        //log::debug!("TLSF: add memory: {:#x} {:#?}",addr, size - 6 * size_of::<usize>());
        //第一个块
        let first = addr as *mut BlockHeader;
        (*first).prev_phy = &mut self.block_null;
        (*first).next_free = &mut self.block_null;
        (*first).prev_free = &mut self.block_null;
        //set_size传入是这个块的“净大小”，要扣去头部的16个字节和尾部多一个null块的32字节
        (*first).set_now_free();
        (*first).set_prev_used();
        (*first).set_size(size - 6 * size_of::<usize>());
        //把第一个块插入到链表中
        self.add_into_list(first);
        //尾部再加一个null块，占32字节
        let tail = (addr + size - 4 * size_of::<usize>()) as *mut BlockHeader;
        (*tail).set_null();
    }

    /// 把一个块插入free list中，需要确保这个块是空闲的
    pub unsafe fn add_into_list(&mut self, block: *mut BlockHeader){
        //log::debug!("add into list******************: {:#x} {:#?}",block as usize, (*block).get_size());
        let size = (*block).get_size();
        let listidx = get_fl_and_sl(size);
        let fl = listidx.fl;
        let sl = listidx.sl;
        //获取了这个块的二级链表之后，插入
        let head = self.blocks[fl][sl];
        (*block).next_free = head;
        (*block).prev_free = &mut self.block_null;
        if !((*head).is_null()){
            (*head).prev_free = block;
        }
        // 别忘了修改链表头以及修改bitmap
        self.blocks[fl][sl] = block;
        self.sl_bitmap[fl] |= (1 as usize) << sl;
        self.fl_bitmap |= (1 as usize) << fl;
    }

    ///把一个块从list中删除，需要确保它之前确实在free list里
    pub unsafe fn del_into_list(&mut self, block: *mut BlockHeader){
        let size = (*block).get_size();
        let listidx = get_fl_and_sl(size);
        let fl = listidx.fl;
        let sl = listidx.sl;
        let prev = (*block).prev_free;
        let next = (*block).next_free;
        //log::debug!("del into list: {:#x}, prev = {:#x}, next = {:#x}, fl = {:#?}, sl = {:#?}",block as usize, prev as usize, next as usize, fl, sl);
        if !((*prev).is_null()){
            (*prev).next_free = next;
        }
        else{//要更新链表头
            self.blocks[fl][sl] = next;
            if (*next).is_null(){//要更新bitmap
                self.sl_bitmap[fl] &= !((1 as usize) << sl);
                if self.sl_bitmap[fl] == 0{
                    self.fl_bitmap &= !((1 as usize) << fl);
                }
            }
        }
        if !((*next).is_null()){
            (*next).prev_free = prev;
        }
    }

    /// 获取某一个链表的第一个块，并从链表中删除
    pub unsafe fn get_first_block(&mut self, fl: usize, sl: usize) -> *mut BlockHeader{
        if (*self.blocks[fl][sl]).is_null(){
            return &mut self.block_null;
        }
        let block = self.blocks[fl][sl];
        self.del_into_list(block);
        return block;
    }

    /// 给定大小，获取一个能用的块，并从链表中删除
    pub unsafe fn find_block(&mut self, size: usize) -> *mut BlockHeader{
        let listidx = get_up_fl_and_sl(size);
        let mut fl = listidx.fl;
        let mut sl = listidx.sl;
        let psl = !(((1 as usize) << sl) - 1);//第二级链表的掩码
        if (psl & self.sl_bitmap[fl]) != 0{//可以在当前一级链表里找到块
            sl = my_log_lowbit(psl & self.sl_bitmap[fl]);
        }
        else{
            let pfl = !(((1 as usize) << (fl + 1)) - 1);//第一级链表的掩码
            if (pfl & self.fl_bitmap) != 0{//可以在更高的一级链表里找到块
                fl = my_log_lowbit(pfl & self.fl_bitmap);
                sl = my_log_lowbit(self.sl_bitmap[fl]);
            }
            else {
                return &mut self.block_null;
            }
        }
        //log::debug!("find block: {:#?} {:#?} {:#?} {:#?}",size, fl, sl, get_block_begin_size(fl, sl));
        return self.get_first_block(fl,sl);
    }
}


pub struct Heap {
    head: *mut Controller,
    total_mem: usize, // 总共占用内存
    used_mem: usize, // 已经分配出去的内存
    avail_mem: usize, // 实际可用的内存
}

unsafe impl Send for Heap {}

impl Heap {
    /// Create an empty heap
    pub const fn new() -> Self {
        Heap {
            head: null_mut(),
            total_mem: 0,
            used_mem: 0,
            avail_mem: 0,
        }
    }

    ///init
    pub unsafe fn init(&mut self, heap_start_addr: usize, heap_size: usize) {
        //log::debug!("TLSF: init addr = {:#x}, size = {:#x}",heap_start_addr,heap_size);
        assert!(
            heap_start_addr % 4096 == 0,
            "Start address should be page aligned"
        );
        assert!(
            heap_size % 4096 == 0 && heap_size > 0,
            "Add Heap size should be a multiple of page size"
        );
        self.head = heap_start_addr as *mut Controller;
        unsafe{(*self.head).init(heap_start_addr, heap_size);}
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
    pub unsafe fn add_memory(&mut self, start_addr: usize, heap_size: usize) {
        //log::debug!("begin addr: {:#x}, end addr: {:#x}, size: {:#?}",start_addr,start_addr + heap_size,heap_size);
        assert!(
            start_addr % 4096 == 0,
            "Start address should be page aligned"
        );
        assert!(
            heap_size % 4096 == 0 && heap_size > 0,
            "Add Heap size should be a multiple of page size"
        );
        (*(self.head)).add_memory(start_addr, heap_size);
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

        unsafe{
            let mut block = (*(self.head)).find_block(size);
            if !((*block).is_null()){
                let mut nsize = (*block).get_size();
                assert!(nsize >= size,"Alloc error.");
                let mut addr = (block as usize) + 2 * size_of::<usize>();
                //log::debug!("*** {:#x} {:#?} {:#x}",block as usize, nsize, get_block_phy_next(block) as usize);

                //处理align更大的分配请求
                if layout.align() > size_of::<usize>(){
                    let mut new_addr = alignto(addr,layout.align());
                    if new_addr != addr{//要切出头部单独组成一块
                        while new_addr - (block as usize) < 6 * size_of::<usize>(){
                            //切出的头部不足以构成一个新块，于是把头部再扩大一个align
                            //因为new_addr是实际分配出去的起始地址，因此到原来块的开头至少要48个字节才能让中间再拆出一个块
                            new_addr += layout.align();
                        }
                        //创造一个新的块pre_block
                        let pre_block = block;
                        let nxt_block = get_block_phy_next(block);
                        block = (new_addr - 2 * size_of::<usize>()) as *mut BlockHeader;
                        //设置物理上的前一块
                        (*block).prev_phy = pre_block;
                        if !((*nxt_block).is_null()){
                            (*nxt_block).prev_phy = block;
                        }
                        //设置块大小
                        let pre_size = (block as usize) - addr;
                        nsize -= pre_size + 2 * size_of::<usize>();
                        (*pre_block).set_size(pre_size);
                        (*block).set_size(nsize);
                        //设置使用状态
                        (*pre_block).set_free();
                        //插回到链表中去
                        (*(self.head)).add_into_list(pre_block);
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
                (*block).set_used();
                
                //把块的尾部拆分之后扔回去
                if nsize >= size + 4 * size_of::<usize>(){//最小32字节才能切出一个新块
                    //新块
                    let new_block = (addr + size) as *mut BlockHeader;
                    (*new_block).prev_phy = block;
                    //原块的下一个块
                    let nxt_block = get_block_phy_next(block);
                    if !((*nxt_block).is_null()){
                        (*nxt_block).prev_phy = new_block;
                    }
                    //设置块大小
                    (*block).set_size(size);
                    (*new_block).set_size(nsize - size - 2 * size_of::<usize>());//别忘了减去新块的头部大小
                    //设置使用状态
                    (*block).set_used();
                    (*new_block).set_free();
                    //插回到链表中去
                    (*(self.head)).add_into_list(new_block);
                    self.avail_mem -= 2 * size_of::<usize>();
                    //log::debug!("new block = {:#x}, size = {:#?}",new_block as usize,(*new_block).get_size());
                }
                self.used_mem += layout.size();
                self.avail_mem -= (*block).get_size();
                //log::debug!("TLSF: successfully allocate: {:#x} {:#?}, pre = {:#x}, nxt = {:#x}, nxt nxt free = {:#x}"
                //    ,addr,(*block).get_size(),get_block_phy_prev(block) as usize,get_block_phy_next(block) as usize
                //    ,(*get_block_phy_next(block)).next_free as usize);
                return Ok(addr);
            }
            else{
                return Err(AllocError);
            }
        }
    }


    /// 把这个块和物理上后一个块合并，要求两个块都是空闲的，且已经从链表中摘下来了
    pub unsafe fn merge_block(&self, block: *mut BlockHeader){
        //log::debug!("TLSF: merge_block {:#x}",block as usize);
        let nxt = get_block_phy_next(block);
        //改block的size
        let size = (*block).get_size();
        let nsize = (*nxt).get_size();
        //log::debug!("{:#x} {:#x} {:#?} {:#?}",block as usize, nxt as usize, size, nsize);
        (*block).set_size(size + nsize + 2 * size_of::<usize>());
        //改block.nxt.nxt的pre指针为block自己
        let nnxt = get_block_phy_next(nxt);
        //log::debug!("{:#x}",nnxt as usize);
        if !((*nnxt).is_null()){
            (*nnxt).prev_phy = block;
        }
    }
     

    

    /// Frees the given allocation. `ptr` must be a pointer returned
    /// by a call to the `allocate` function with identical size and alignment. Undefined
    /// behavior may occur for invalid arguments, thus this function is unsafe.
    ///
    /// # Safety
    /// This function is unsafe because it can cause undefined behavior if the
    /// given address is invalid.
    pub unsafe fn deallocate(&mut self, ptr: usize, layout: Layout) {
        //log::debug!("TLSF: deallocate: ptr = {:#x}, size = {:#?}",ptr,layout.size());
        //log::debug!("qaq: {:#x}",self.free_list.head as usize);
        assert!(my_lowbit(layout.align()) == layout.align(),"align should be power of 2.");
        let size = alignto(max(
            layout.size(),
            max(layout.align(), 2 * size_of::<usize>()),
        ),max(layout.align(),size_of::<usize>()));
        let block = (ptr - 2 * size_of::<usize>()) as *mut BlockHeader;
        let block_size = (*block).get_size();
        //log::debug!("block = {:#x}, size = {:#?}, block_size = {:#?}",block as usize,size,block_size);
        assert!(block_size >= size && (*block).get_now_free() == false, "Dealloc error");
        (*block).set_free();
        self.used_mem -= layout.size();
        self.avail_mem += block_size;
        
        //把这个块与前后的块合并
        let mut nblock = block;
        let pre = get_block_phy_prev(block);
        let nxt = get_block_phy_next(block);
        //log::debug!("TLSF: dealloc block = {:#x}, pre = {:#x}, nxt = {:#x}",block as usize, pre as usize, nxt as usize);
        if !((*nxt).is_null()) && (*(nxt)).get_now_free(){
            //如果物理上的下一个块不是null且是空闲的，就合并
            (*(self.head)).del_into_list(nxt);
            self.merge_block(nblock);
            self.avail_mem += 2 * size_of::<usize>();
        }
        if !((*pre).is_null()) && (*(pre)).get_now_free(){
            //如果物理上的上一个块不是null且是空闲的，就合并
            (*(self.head)).del_into_list(pre);
            self.merge_block(pre);
            nblock = pre;
            self.avail_mem += 2 * size_of::<usize>();
        }
        //log::debug!("TLSF: dealloc nblock = {:#x}",nblock as usize);
        (*(self.head)).add_into_list(nblock);

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
