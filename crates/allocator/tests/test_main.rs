#![feature(ptr_alignment_type)]
mod tlsf_c;
mod test_lib;
use test_lib::*;
use std::collections::BTreeMap;
use std::vec::Vec;
use allocator::{BasicAllocator, SlabByteAllocator, BuddyByteAllocator, TLSFAllocator, TLSFCAllocator};
use std::{alloc::{GlobalAlloc, Layout, System}, ffi::c_ulonglong};
use allocator::{AllocResult, BaseAllocator, ByteAllocator};
use std::mem::size_of;
use tlsf_c::TlsfCAllocator;

use core::panic;
use spin::Mutex;
pub enum AllocType{
    SystemAlloc,
    BasicAlloc,
    BuddyAlloc,
    SlabAlloc,
    TlsfCAlloc,
    TlsfRustAlloc,
}

pub struct GlobalAllocator {
    basic_alloc: Mutex<BasicAllocator>,
    buddy_alloc: Mutex<BuddyByteAllocator>,
    slab_alloc: Mutex<SlabByteAllocator>,
    tlsf_c_alloc: Mutex<TLSFCAllocator>,
    tlsf_rust_alloc: Mutex<TLSFAllocator>,
    alloc_type: AllocType,
    heap_arddress: usize,
    heap_size: usize,
}

const PAGE_SIZE: usize = 1 << 12;
const HEAP_SIZE: usize = 1 << 24;
static mut HEAP: [usize; HEAP_SIZE + PAGE_SIZE] = [0; HEAP_SIZE + PAGE_SIZE];

static mut FLAG: bool = false;

impl GlobalAllocator {
    pub const fn new() -> Self {
        Self {
            basic_alloc: Mutex::new(BasicAllocator::new()),
            buddy_alloc: Mutex::new(BuddyByteAllocator::new()),
            slab_alloc: Mutex::new(SlabByteAllocator::new()),
            tlsf_c_alloc: Mutex::new(TLSFCAllocator::new()),
            tlsf_rust_alloc: Mutex::new(TLSFAllocator::new()),
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

    pub unsafe fn init_tlsf_rust(&mut self) {
        self.tlsf_rust_alloc.lock().init(self.heap_arddress,self.heap_size);
        self.alloc_type = AllocType::TlsfRustAlloc;
    }

    pub unsafe fn alloc(&self, layout: Layout) -> AllocResult<usize> {
        //默认alloc请求都是8对齐
        let size: usize = layout.size();
        let align_pow2: usize = layout.align();
        //assert!(align_pow2 <= size_of::<usize>());
        //println!("***");
        //axlog::debug!("alloc");
        //axlog::debug!("alloc size: {:#?}, align: {:#?}",size,align_pow2);
        if FLAG{
            let ptr = System.alloc(layout);
            return Ok(ptr as usize);
        }

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
            AllocType::TlsfRustAlloc => {
                FLAG = true;
                //log::debug!("alloc size: {:#?}, align: {:#?}",size,align_pow2);
                if let Ok(ptr) = self.tlsf_rust_alloc.lock().alloc(size, align_pow2) {
                    //log::debug!("successfully alloc: {:#x}",ptr);
                    FLAG = false;
                    return Ok(ptr);
                } else { panic!("alloc err: no memery.");}
            }


            _ => { panic!("unknown alloc type.");}
        }
        
    }

    pub unsafe fn dealloc(&self, pos: usize, layout: Layout) {
        let size: usize = layout.size();
        let align_pow2: usize = layout.align();
        if FLAG{
            System.dealloc(pos as *mut u8, layout);
            return;
        }

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
            AllocType::TlsfRustAlloc => {
                FLAG = true;
                //log::debug!("dealloc pos: {:#x}, size: {:#?}, align: {:#?}",pos, size, align_pow2);
                self.tlsf_rust_alloc.lock().dealloc(pos, size, align_pow2);
                //log::debug!("successfully dealloc.");
                FLAG = false;
            }
            _ => {
                panic!("unknown alloc type.");
            }
        }
    }

    pub fn total_bytes(&self) -> usize {
        match self.alloc_type{
            AllocType::SystemAlloc => {//不适用
                0
            }
            AllocType::BasicAlloc => {
                self.basic_alloc.lock().total_bytes()
            }
            AllocType::BuddyAlloc => {
                self.buddy_alloc.lock().total_bytes()
            }
            AllocType::SlabAlloc => {
                self.slab_alloc.lock().total_bytes()
            }
            AllocType::TlsfCAlloc => {
                self.tlsf_c_alloc.lock().total_bytes()
            }
            AllocType::TlsfRustAlloc => {
                self.tlsf_rust_alloc.lock().total_bytes()
            }
            _ => {
                panic!("unknown alloc type.");
            }
        }
    }

    pub fn used_bytes(&self) -> usize {
        match self.alloc_type{
            AllocType::SystemAlloc => {//不适用
                0
            }
            AllocType::BasicAlloc => {
                self.basic_alloc.lock().used_bytes()
            }
            AllocType::BuddyAlloc => {
                self.buddy_alloc.lock().used_bytes()
            }
            AllocType::SlabAlloc => {
                self.slab_alloc.lock().used_bytes()
            }
            AllocType::TlsfCAlloc => {
                self.tlsf_c_alloc.lock().used_bytes()
            }
            AllocType::TlsfRustAlloc => {
                self.tlsf_rust_alloc.lock().used_bytes()
            }
            _ => {
                panic!("unknown alloc type.");
            }
        }
    }

    pub fn available_bytes(&self) -> usize {
        match self.alloc_type{
            AllocType::SystemAlloc => {//不适用
                0
            }
            AllocType::BasicAlloc => {
                self.basic_alloc.lock().available_bytes()
            }
            AllocType::BuddyAlloc => {
                self.buddy_alloc.lock().available_bytes()
            }
            AllocType::SlabAlloc => {
                self.slab_alloc.lock().available_bytes()
            }
            AllocType::TlsfCAlloc => {
                self.tlsf_c_alloc.lock().available_bytes()
            }
            AllocType::TlsfRustAlloc => {
                self.tlsf_rust_alloc.lock().available_bytes()
            }
            _ => {
                panic!("unknown alloc type.");
            }
        }
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
pub static mut GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator::new();


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

pub type CallBackMalloc = unsafe extern fn(size: c_ulonglong) -> c_ulonglong;
pub type CallBackMallocAligned = unsafe extern fn(size: c_ulonglong,align: c_ulonglong) -> c_ulonglong;
pub type CallBackFree = unsafe extern fn(ptr: c_ulonglong,size: c_ulonglong);
#[link(name = "mitest")]
extern {
    pub fn mi_test_start(cb1: CallBackMalloc, cb2: CallBackMallocAligned, cb3: CallBackFree);
}
pub unsafe extern fn cb_malloc_func(size: c_ulonglong) -> c_ulonglong{
    if let Ok(ptr) = GLOBAL_ALLOCATOR.alloc(Layout::from_size_align_unchecked(size as usize,8)){
        return ptr as c_ulonglong;
    }
    panic!("alloc err.");
}
pub unsafe extern fn cb_malloc_aligned_func(size: c_ulonglong,align: c_ulonglong) -> c_ulonglong{
    if let Ok(ptr) = GLOBAL_ALLOCATOR.alloc(Layout::from_size_align_unchecked(size as usize,align as usize)){
        return ptr as c_ulonglong;
    }
    panic!("alloc err.");
}
pub unsafe extern fn cb_free_func(ptr: c_ulonglong,size: c_ulonglong){
    GLOBAL_ALLOCATOR.dealloc(ptr as usize, Layout::from_size_align_unchecked(size as usize,8));
}
pub fn mi_test(){
    //return;
    println!("Mi alloc test begin...");
    let t0 = std::time::Instant::now();
    unsafe{ mi_test_start(cb_malloc_func,cb_malloc_aligned_func,cb_free_func);}
    let t1 = std::time::Instant::now();
    println!("time: {:#?}",t1 - t0);
    println!("Mi alloc test OK!");
}

///memory chk
pub fn memory_chk(){
    unsafe{
        let tot = GLOBAL_ALLOCATOR.total_bytes() as f64;
        let used = GLOBAL_ALLOCATOR.used_bytes() as f64;
        let avail = GLOBAL_ALLOCATOR.available_bytes() as f64;
        println!("total memory: {:#?} MB",tot / 1048576.0);
        println!("used memory: {:#?} MB",used / 1048576.0);
        println!("available memory: {:#?} MB",avail / 1048576.0);
        println!("occupied memory: {:#?} MB",(tot - avail) / 1048576.0);
        println!("extra memory rate: {:#?}%",(tot - avail - used) / (tot - avail) * 100.0);
    }
}

pub fn test_vec(n: usize) {
    //const N: usize = 1_000_000;
    //let mut v = Vec::with_capacity(N);
    println!("test_vec() begin...");
    let mut v = Vec::new();
    for _ in 0..n {
        //println!("vector push 1");
        v.push(rand_u32());
    }
    //v.sort();
    //for i in 0..n - 1 {
    //    assert!(v[i] <= v[i + 1]);
    //}
    memory_chk();
    println!("test_vec() OK!");
    println!("*****");
}

pub fn test_btree_map(n: usize) {
    println!("test_btree_map() begin...");
    //const N: usize = 20;
    let mut m = BTreeMap::new();
    for _ in 0..n {
        //println!("test btree map: {:#?}",i);
        if rand_usize() % 5 == 0 && !m.is_empty() {
            m.pop_first();
        } else {
            let value = rand_usize();
            let key = format!("key_{value}");
            m.insert(key, value);
        }
        //if i > 1 {break;}
    }
    for (k, v) in m.iter() {
        if let Some(k) = k.strip_prefix("key_") {
            assert_eq!(k.parse::<usize>().unwrap(), *v);
        }
    }
    //println!("{:#?}",m.len());
    memory_chk();
    println!("test_btree_map() OK!");
    println!("*****");
}

pub fn test_vec_2(n: usize, m: usize){
    println!("test_vec2() begin...");
    //let mut v = Vec::with_capacity(N);
    let mut v:Vec<Vec<usize>> = Vec::new();
    for _ in 0..n {
        //println!("vector push {:#?}",i);
        let mut tmp: Vec<usize> = Vec::with_capacity(m);
        for _ in 0..m {
            tmp.push(rand_usize());
        }
        tmp.sort();
        for j in 0..m - 1 {
            assert!(tmp[j] <= tmp[j + 1]);
        }
        v.push(tmp);
    }

    let mut p: Vec<usize> = Vec::with_capacity(n);
    for i in 0..n {
        p.push(i);
    }
    memory_chk();

    for i in 1..n {
        let o: usize = rand_usize() % (i + 1);
        let tmp = p[i];
        p[i] = p[o];
        p[o] = tmp;
    }
    for i in 0..n {
        let o = p[i];
        let tmp: Vec<usize> = Vec::new();
        v[o] = tmp;
    }
    //v.sort();
    /*
    for _ in 0..N {
        println!("vector push 2");
        v.push(rand::rand_u32());
    }
    */
    memory_chk();
    println!("test_vec2() OK!");
    println!("*****");
}

pub fn test_vec_3(n: usize,k1: usize, k2: usize){
    println!("test_vec3() begin...");
    let mut v:Vec<Vec<usize>> = Vec::new();
    for i in 0..n * 4{
        let nw = match i >= n * 2 {
            true => k1,
            false => match i % 2 {
                0 => k1,
                _ => k2,
            },
        };
        v.push(Vec::with_capacity(nw));
        for _ in 0..nw {
            v[i].push(rand_usize());
        }
    }
    memory_chk();
    for i in 0..n * 4{
        if i % 2 == 1 {
            let tmp: Vec<usize> = Vec::new();
            v[i] = tmp;
        }
    }
    for i in 0..n {
        let nw = k2;
        v.push(Vec::with_capacity(nw));
        for _ in 0..nw {
            v[4 * n + i].push(rand_usize());
        }
    }
    memory_chk();
    println!("test_vec3() OK!");
    println!("*****");
}


/// basic test
pub fn basic_test() {
    println!("Basic alloc test begin...");
    let t0 = std::time::Instant::now();
    test_vec(3000000);
    test_vec_2(30000,64);
    test_vec_2(7500,520);
    test_btree_map(50000);
    test_vec_3(10000,32,64);
    let t1 = std::time::Instant::now();
    println!("time: {:#?}",t1 - t0);
    println!("Basic alloc test OK!");
    println!("*****");
}

pub fn new_mem(size: usize, align: usize) -> usize{
    unsafe{
        if let Ok(ptr) = GLOBAL_ALLOCATOR.alloc(Layout::from_size_align_unchecked(size,align)){
            return ptr;
        }
        panic!("alloc err.");
    }
}

/// align test
pub fn align_test() {
    println!("Align alloc test begin...");
    let t0 = std::time::Instant::now();
    let mut v = Vec::new();
    let mut v2 = Vec::new();
    let mut v3 = Vec::new();
    let mut p = Vec::new();
    let n = 50000;
    let mut cnt = 0;
    let mut nw = 0;
    for _ in 0..n{
        if (rand_u32() % 3 != 0) | (nw == 0){//插入一个块
            let size = (((1 << (rand_u32() & 15)) as f64) * (1.0 + (rand_u32() as f64) / (0xffffffff as u32 as f64))) as usize;
            let align = (1 << (rand_u32() & 7)) as usize;
            //println!("alloc: size = {:#?}, align = {:#?}",size,align);
            let addr = new_mem(size, align);
            v.push(addr);
            //println!("successfully alloc: addr = {:#x}",addr);
            assert!((addr & (align - 1)) == 0,"align not correct.");
            v2.push(size);
            v3.push(align);
            p.push(cnt);
            cnt += 1;
            nw += 1;
        }
        else{//删除一个块
            let idx = rand_usize() % nw;
            let addr = v[p[idx]];
            let size = v2[p[idx]];
            let align = v3[p[idx]];
            //println!("dealloc: addr = {:#x}, size = {:#?}, align = {:#?}",addr,size,align);
            unsafe{GLOBAL_ALLOCATOR.dealloc(addr, Layout::from_size_align_unchecked(size as usize,align));}
            nw -= 1;
            p[idx] = p[nw];
            p.pop();
        }
    }
    memory_chk();
    for idx in 0..nw{
        let addr = v[p[idx]];
        let size = v2[p[idx]];
        let align = v3[p[idx]];
        unsafe{GLOBAL_ALLOCATOR.dealloc(addr, Layout::from_size_align_unchecked(size as usize,align));}
    }
    let t1 = std::time::Instant::now();
    println!("time: {:#?}",t1 - t0);
    println!("Align alloc test OK!");
    println!("*****");
}

#[test]
fn test_start() {
    axlog::init();
    axlog::set_max_level("debug");
    unsafe{GLOBAL_ALLOCATOR.init_heap();}
    call_back_test(233);
    println!("Running memory tests...");

    println!("system alloc test:");
    unsafe{GLOBAL_ALLOCATOR.init_system();}
    align_test();
    basic_test();
    mi_test();
    println!("system test passed!");
    println!("*****************************");

    println!("tlsf_rust alloc test:");
    unsafe{GLOBAL_ALLOCATOR.init_tlsf_rust();}
    align_test();
    basic_test();
    mi_test();
    println!("tlsf_rust alloc test passed!");
    println!("*****************************");
    unsafe{GLOBAL_ALLOCATOR.init_system();}

    //return;

    println!("first fit alloc test:");
    unsafe{GLOBAL_ALLOCATOR.init_basic("first_fit");}
    //align_test();
    basic_test();
    mi_test();
    println!("first fit alloc test passed!");
    println!("*****************************");
    unsafe{GLOBAL_ALLOCATOR.init_system();}

    println!("best fit alloc test:");
    unsafe{GLOBAL_ALLOCATOR.init_basic("best_fit");}
    //align_test();
    basic_test();
    mi_test();
    println!("best fit alloc test passed!");
    println!("*****************************");
    unsafe{GLOBAL_ALLOCATOR.init_system();}

    println!("worst fit alloc test:");
    unsafe{GLOBAL_ALLOCATOR.init_basic("worst_fit");}
    //align_test();
    basic_test();
    mi_test();
    println!("worst fit alloc test passed!");
    println!("*****************************");
    unsafe{GLOBAL_ALLOCATOR.init_system();}

    println!("buddy alloc test:");
    unsafe{GLOBAL_ALLOCATOR.init_buddy();}
    //align_test();
    basic_test();
    mi_test();
    println!("buddy alloc test passed!");
    println!("*****************************");
    unsafe{GLOBAL_ALLOCATOR.init_system();}

    println!("slab alloc test:");
    unsafe{GLOBAL_ALLOCATOR.init_slab();}
    //align_test();
    basic_test();
    mi_test();
    println!("slab alloc test passed!");
    println!("*****************************");
    unsafe{GLOBAL_ALLOCATOR.init_system();}

    println!("tlsf_c alloc test:");
    unsafe{GLOBAL_ALLOCATOR.init_tlsf_c();}
    align_test();
    basic_test();
    mi_test();
    println!("tlsf_c alloc test passed!");
    println!("*****************************");
    unsafe{GLOBAL_ALLOCATOR.init_system();}

    println!("Memory tests run OK!");
}
