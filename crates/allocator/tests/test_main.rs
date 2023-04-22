mod basic_test;
use basic_test::*;
use allocator::{BasicAllocator, SlabByteAllocator, BuddyByteAllocator};
use std::alloc::{GlobalAlloc, Layout, System};
use allocator::{AllocResult, BaseAllocator, ByteAllocator};
use std::mem::size_of;


use core::{panic};
use spin::Mutex;
pub enum AllocType{
    SystemAlloc,
    BasicAlloc,
    BuddyAlloc,
    SlabAlloc,
}

pub struct GlobalAllocator {
    //balloc: SpinNoIrq<SlabByteAllocator>,
    basic_alloc: Mutex<BasicAllocator>,
    buddy_alloc: Mutex<BuddyByteAllocator>,
    slab_alloc: Mutex<SlabByteAllocator>,
    alloc_type: AllocType,
    heap_arddress: usize,
    heap_size: usize,
}

const PAGE_SIZE: usize = 1 << 12;
const HEAP_SIZE: usize = 1 << 24;
static mut HEAP: [usize; HEAP_SIZE + PAGE_SIZE] = [0;HEAP_SIZE + PAGE_SIZE];

impl GlobalAllocator {
    pub const fn new() -> Self {
        Self {
            basic_alloc: Mutex::new(BasicAllocator::new()),
            buddy_alloc: Mutex::new(BuddyByteAllocator::new()),
            slab_alloc: Mutex::new(SlabByteAllocator::new()),
            alloc_type: AllocType::SystemAlloc,
            heap_arddress: 0,
            heap_size: 0,
        }
    }

    pub unsafe fn init_heap(&mut self) {
        self.heap_arddress = (HEAP.as_ptr() as usize + PAGE_SIZE - 1) / PAGE_SIZE * PAGE_SIZE;
        self.heap_size = HEAP_SIZE * size_of::<usize>();
    }

    pub unsafe fn init_system(&mut self) {
        self.alloc_type = AllocType::SystemAlloc;
    }

    pub unsafe fn init_basic(&mut self) {
        self.basic_alloc.lock().init(self.heap_arddress,self.heap_size);
        self.alloc_type = AllocType::BasicAlloc;
    }

    pub unsafe fn init_buddy(&mut self) {
        self.buddy_alloc.lock().init(self.heap_arddress,self.heap_size);
        self.alloc_type = AllocType::BuddyAlloc;
    }

    pub unsafe fn init_slab(&mut self) {
        self.slab_alloc.lock().init(self.heap_arddress,self.heap_size);
        self.alloc_type = AllocType::SlabAlloc;
    }

    pub unsafe fn alloc(&self, layout: Layout) -> AllocResult<usize> {
        //默认alloc请求都是8对齐
        let size: usize = layout.size();
        let align_pow2: usize = layout.align();
        //assert!(align_pow2 <= size_of::<usize>());
        //println!("alloc size: {:#?}, align: {:#?}",size,align_pow2);
        match self.alloc_type{
            AllocType::SystemAlloc => {
                let ptr = System.alloc(layout);
                return Ok(ptr as usize);
            }
            AllocType::BasicAlloc => {
                if let Ok(ptr) = self.basic_alloc.lock().alloc(size, align_pow2) {
                    return Ok(ptr);
                } else { panic!("alloc err: no memery.");}
            }
            AllocType::BuddyAlloc => {
                if let Ok(ptr) = self.buddy_alloc.lock().alloc(size, align_pow2) {
                    return Ok(ptr);
                } else { panic!("alloc err: no memery.");}
            }
            AllocType::SlabAlloc => {
                if let Ok(ptr) = self.slab_alloc.lock().alloc(size, align_pow2) {
                    return Ok(ptr);
                } else { panic!("alloc err: no memery.");}
            }
            _ => { panic!("unknown alloc type.");}
        }
        
    }

    pub unsafe fn dealloc(&self, pos: usize, layout: Layout) {
        let size: usize = layout.size();
        let align_pow2: usize = layout.align();
        //debug!("dealloc pos: {:#x}, size: {:#?}, align: {:#?}",pos, size, align_pow2);

        match self.alloc_type{
            AllocType::SystemAlloc => {
                System.dealloc(pos as *mut u8, layout);
            }
            AllocType::BasicAlloc => {
                self.basic_alloc.lock().dealloc(pos, size, align_pow2);
            }
            AllocType::BuddyAlloc => {
                self.buddy_alloc.lock().dealloc(pos, size, align_pow2);
            }
            AllocType::SlabAlloc => {
                self.slab_alloc.lock().dealloc(pos, size, align_pow2);
            }
            _ => {
                panic!("unknown alloc type.");
            }
        }
        //debug!("successfully dealloc.");
    }

    pub fn used_bytes(&self) -> usize {
        self.basic_alloc.lock().used_bytes()
    }

    pub fn available_bytes(&self) -> usize {
        self.basic_alloc.lock().available_bytes()
    }

}

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if let Ok(ptr) = GlobalAllocator::alloc(self, layout) {
            ptr as _
        } else {
            panic!("alloc err.");
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        GlobalAllocator::dealloc(self, ptr as _, layout)
    }
}

#[global_allocator]
static mut GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator::new();

#[test]
fn test_start() {
    axlog::init();
    axlog::set_max_level("debug");
    unsafe{GLOBAL_ALLOCATOR.init_heap();}
    println!("Running memory tests...");

    //test_vec(1000000);
    
    //test_vec(3000000);
    
    //test_vec_2(5000,32);
    //test_vec_2(10000,4);
    //test_vec_2(20000,4);
    //test_vec_2(10000,32);
    //test_vec_2(5000,64);
    //test_vec_2(20000,64);
    //test_vec_2(30000,64);

    //test_vec_2(100000,4);

    //test_vec_2(20000,64);

    //test_vec_2(7500,520);
    //test_vec_2(10000,32);

    //test_btree_map(3);
    //test_btree_map(10000);
    //test_btree_map(20000);
    //test_btree_map(50000);
    //test_btree_map(100000);
    
    //test_vec_3(5000,8,16);
    //test_vec_3(10000,32,64);

    println!("basic alloc test:");
    let t0 = std::time::Instant::now();
    unsafe{GLOBAL_ALLOCATOR.init_basic();}
    test_vec(3000000);
    test_vec_2(30000,64);
    test_vec_2(7500,520);
    test_btree_map(100000);
    test_vec_3(10000,32,64);
    let t1 = std::time::Instant::now();
    println!("time: {:#?}",t1 - t0);
    println!("basic alloc test passed!");
    unsafe{GLOBAL_ALLOCATOR.init_system();}

    println!("buddy alloc test:");
    let t0 = std::time::Instant::now();
    unsafe{GLOBAL_ALLOCATOR.init_buddy();}
    test_vec(3000000);
    test_vec_2(30000,64);
    test_vec_2(7500,520);
    test_btree_map(100000);
    test_vec_3(10000,32,64);
    let t1 = std::time::Instant::now();
    println!("time: {:#?}",t1 - t0);
    println!("buddy alloc test passed!");
    unsafe{GLOBAL_ALLOCATOR.init_system();}

    println!("slab alloc test:");
    let t0 = std::time::Instant::now();
    unsafe{GLOBAL_ALLOCATOR.init_slab();}
    test_vec(3000000);
    test_vec_2(30000,64);
    test_vec_2(7500,520);
    test_btree_map(100000);
    test_vec_3(10000,32,64);
    let t1 = std::time::Instant::now();
    println!("time: {:#?}",t1 - t0);
    println!("slab alloc test passed!");
    unsafe{GLOBAL_ALLOCATOR.init_system();}


    println!("Memory tests run OK!");
}
