
use std::collections::BTreeMap;
use std::vec::Vec;
use allocator::BasicAllocator;
use std::alloc::{GlobalAlloc, Layout, System};
use allocator::{AllocError, AllocResult, BaseAllocator, ByteAllocator};
use std::mem::size_of;


use core::{sync::atomic::{AtomicU64, Ordering::SeqCst}, panic};
use spin::Mutex;

static SEED: AtomicU64 = AtomicU64::new(0xa2ce_a2ce);

/// Sets the seed for the random number generator.
pub fn srand(seed: u32) {
    SEED.store(seed.wrapping_sub(1) as u64, SeqCst);
}

/// Returns a 32-bit unsigned pseudo random interger.
pub fn rand_u32() -> u32 {
    let new_seed = SEED.load(SeqCst).wrapping_mul(6364136223846793005) + 1;
    SEED.store(new_seed, SeqCst);
    (new_seed >> 33) as u32
}

pub fn rand_usize() -> usize {
    ((rand_u32() as usize) << 32) | (rand_u32() as usize)
}

fn test_vec(n: usize) {
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
    println!("test_vec() OK!");
}

fn test_btree_map(n: usize) {
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
    println!("test_btree_map() OK!");
}

fn test_vec_2(n: usize, m: usize){
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
    
    println!("test_vec2() OK!");
}

fn test_vec_3(n: usize,k1: usize, k2: usize){
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
    println!("test_vec3() OK!");
}


pub struct GlobalAllocator {
    //balloc: SpinNoIrq<SlabByteAllocator>,
    balloc: Mutex<BasicAllocator>,
    flag: bool,
}

const HEAP_SIZE: usize = 1 << 23;
static mut HEAP: [usize; HEAP_SIZE] = [0;HEAP_SIZE];

impl GlobalAllocator {
    pub const fn new() -> Self {
        Self {
            //balloc: SpinNoIrq::new(SlabByteAllocator::new()),
            balloc: Mutex::new(BasicAllocator::new()),
            flag: false,
        }
    }

    pub unsafe fn init(&mut self) {
        self.balloc.lock().init(HEAP.as_mut_ptr() as usize, HEAP_SIZE * size_of::<usize>());
        self.flag = true;
    }

    pub unsafe fn clear(&mut self) {
        self.flag = false;
    }

    pub fn alloc(&self, size: usize, align_pow2: usize) -> AllocResult<usize> {
        //默认alloc请求都是8对齐
        assert!(align_pow2 <= size_of::<usize>());
        //println!("alloc size: {:#?}, align: {:#?}",size,align_pow2);
        // simple two-level allocator: if no heap memory, allocate from the page allocator.
        let mut balloc = self.balloc.lock();
        
        if let Ok(ptr) = balloc.alloc(size, align_pow2) {
            //debug!("successfully alloc ptr: {:#x}",ptr);
            return Ok(ptr);
        } else {
            panic!("alloc err: no memery.");
        }
        
    }

    pub fn dealloc(&self, pos: usize, size: usize, align_pow2: usize) {
        //debug!("dealloc pos: {:#x}, size: {:#?}, align: {:#?}",pos, size, align_pow2);
        self.balloc.lock().dealloc(pos, size, align_pow2);
        //debug!("successfully dealloc.");
    }

    pub fn used_bytes(&self) -> usize {
        self.balloc.lock().used_bytes()
    }

    pub fn available_bytes(&self) -> usize {
        self.balloc.lock().available_bytes()
    }

}

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if !self.flag {
            System.alloc(layout)
        } else if let Ok(ptr) = GlobalAllocator::alloc(self, layout.size(), layout.align()) {
            ptr as _
        } else {
            panic!("alloc err.");
        }
        
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if !self.flag {
            System.dealloc(ptr, layout)
        } else {
            GlobalAllocator::dealloc(self, ptr as _, layout.size(), layout.align())
        }
    }
}

#[global_allocator]
static mut GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator::new();

#[test]
fn f() {

    axlog::init();
    axlog::set_max_level("debug");
    unsafe{GLOBAL_ALLOCATOR.init();}
    println!("Running memory tests...");

    let t0 = std::time::Instant::now();

    //test_vec(1000000);
    
    test_vec(3000000);
    
    //test_vec_2(5000,32);
    //test_vec_2(10000,4);
    //test_vec_2(20000,4);
    //test_vec_2(10000,32);
    //test_vec_2(5000,64);
    //test_vec_2(20000,64);
    test_vec_2(30000,64);

    //test_vec_2(100000,4);

    //test_vec_2(20000,64);

    test_vec_2(7500,520);
    //test_vec_2(10000,32);

    //test_btree_map(3);
    //test_btree_map(10000);
    //test_btree_map(20000);
    //test_btree_map(50000);
    test_btree_map(100000);
    
    //test_vec_3(5000,8,16);
    test_vec_3(10000,32,64);


    let t1 = std::time::Instant::now();
    println!("time: {:#?}",t1 - t0);
    println!("Memory tests run OK!");

    unsafe{GLOBAL_ALLOCATOR.clear();}
}
