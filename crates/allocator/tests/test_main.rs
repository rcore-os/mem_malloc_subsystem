mod basic_test;
mod tlsf_c;
use basic_test::*;
use allocator::{BasicAllocator, SlabByteAllocator, BuddyByteAllocator};
use std::alloc::{GlobalAlloc, Layout, System};
use allocator::{AllocResult, BaseAllocator, ByteAllocator};
use std::mem::size_of;
use tlsf_c::TlsfCAllocator;

use core::{panic};
use spin::Mutex;
pub enum AllocType{
    SystemAlloc,
    BasicAlloc,
    BuddyAlloc,
    SlabAlloc,
    TlsfCAlloc,
}

pub struct GlobalAllocator {
    //balloc: SpinNoIrq<SlabByteAllocator>,
    basic_alloc: Mutex<BasicAllocator>,
    buddy_alloc: Mutex<BuddyByteAllocator>,
    slab_alloc: Mutex<SlabByteAllocator>,
    tlsf_c_alloc: Mutex<TlsfCAllocator>,
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
            tlsf_c_alloc: Mutex::new(TlsfCAllocator::new()),
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

    pub unsafe fn init_basic(&mut self,strategy: &str) {
        self.basic_alloc.lock().init(self.heap_arddress,self.heap_size);
        self.basic_alloc.lock().set_strategy(strategy);
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

    pub unsafe fn init_tlsf_c(&mut self) {
        self.tlsf_c_alloc.lock().init(self.heap_arddress,self.heap_size);
        self.alloc_type = AllocType::TlsfCAlloc;
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
            AllocType::TlsfCAlloc => {
                if let Ok(ptr) = self.tlsf_c_alloc.lock().alloc(size, align_pow2) {
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
            AllocType::TlsfCAlloc => {
                self.tlsf_c_alloc.lock().dealloc(pos, size, align_pow2);
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


use std::ffi::c_int;
pub type CallBack = unsafe extern fn(c_int) -> c_int;
#[link(name = "test")]
extern {
    pub fn hello(a: c_int, cb: CallBack) -> c_int;
}
pub unsafe extern fn cb_func(x: c_int) -> c_int{
    println!("hello rust! {:#?}",x);
    return x * x + 1;
}
pub fn call_back_test(x: c_int){
    unsafe{
        let y = hello(x,cb_func);
        println!("rust call_back test passed! {:#?}",y);
    }
    println!("*****************************");
}

#[test]
fn test_start() {
    axlog::init();
    axlog::set_max_level("debug");
    unsafe{GLOBAL_ALLOCATOR.init_heap();}
    call_back_test(233);
    println!("Running memory tests...");
    
    //return;

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

    println!("system alloc test:");
    unsafe{GLOBAL_ALLOCATOR.init_system();}
    basic_test();
    println!("system test passed!");
    println!("*****************************");

    println!("first fit alloc test:");
    unsafe{GLOBAL_ALLOCATOR.init_basic("first_fit");}
    basic_test();
    println!("first fit alloc test passed!");
    println!("*****************************");
    unsafe{GLOBAL_ALLOCATOR.init_system();}

    println!("best fit alloc test:");
    unsafe{GLOBAL_ALLOCATOR.init_basic("best_fit");}
    basic_test();
    println!("best fit alloc test passed!");
    println!("*****************************");
    unsafe{GLOBAL_ALLOCATOR.init_system();}

    println!("worst fit alloc test:");
    unsafe{GLOBAL_ALLOCATOR.init_basic("worst_fit");}
    basic_test();
    println!("worst fit alloc test passed!");
    println!("*****************************");
    unsafe{GLOBAL_ALLOCATOR.init_system();}

    println!("buddy alloc test:");
    unsafe{GLOBAL_ALLOCATOR.init_buddy();}
    basic_test();
    println!("buddy alloc test passed!");
    println!("*****************************");
    unsafe{GLOBAL_ALLOCATOR.init_system();}

    println!("slab alloc test:");
    unsafe{GLOBAL_ALLOCATOR.init_slab();}
    basic_test();
    println!("slab alloc test passed!");
    println!("*****************************");
    unsafe{GLOBAL_ALLOCATOR.init_system();}

    println!("tlsf_c alloc test:");
    unsafe{GLOBAL_ALLOCATOR.init_tlsf_c();}
    basic_test();
    println!("tlsf_c alloc test passed!");
    println!("*****************************");
    unsafe{GLOBAL_ALLOCATOR.init_system();}


    println!("Memory tests run OK!");
}
