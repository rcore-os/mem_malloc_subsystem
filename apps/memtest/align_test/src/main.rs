#![no_std]
#![no_main]

#[macro_use]
extern crate libax;
extern crate alloc;

use alloc::vec::Vec;
use axalloc::GLOBAL_ALLOCATOR;
use libax::rand::rand_u32;

/// rand usize
pub fn rand_usize() -> usize {
    ((rand_u32() as usize) << 32) | (rand_u32() as usize)
}

/// memory chk
pub fn memory_chk() {
    let tot = GLOBAL_ALLOCATOR.total_bytes() as f64;
    let used = GLOBAL_ALLOCATOR.used_bytes() as f64;
    let avail = GLOBAL_ALLOCATOR.available_bytes() as f64;
    println!("total memory: {:#?} MB", tot / 1048576.0);
    println!("used memory: {:#?} MB", used / 1048576.0);
    println!("available memory: {:#?} MB", avail / 1048576.0);
    println!("occupied memory: {:#?} MB", (tot - avail) / 1048576.0);
    println!(
        "extra memory rate: {:#?}%",
        (tot - avail - used) / (tot - avail) * 100.0
    );
}

pub fn new_mem(size: usize, align: usize) -> usize {
    if let Ok(ptr) = GLOBAL_ALLOCATOR.alloc(size, align) {
        return ptr;
    }
    panic!("alloc err.");
}

/// align test
pub fn align_test() {
    println!("Align alloc test begin...");
    let t0 = libax::time::Instant::now();
    let mut v = Vec::new();
    let mut v2 = Vec::new();
    let mut v3 = Vec::new();
    let mut p = Vec::new();
    let n = 30000;
    let mut cnt = 0;
    let mut nw = 0;
    for _ in 0..n {
        if (rand_u32() % 3 != 0) | (nw == 0) {
            // add a block
            let size = (((1 << (rand_u32() & 15)) as f64)
                * (1.0 + (rand_u32() as f64) / (0xffffffff_u32 as f64)))
                as usize;
            let align = (1 << (rand_u32() & 7)) as usize;
            let addr = new_mem(size, align);
            v.push(addr);
            assert!((addr & (align - 1)) == 0, "align not correct.");
            v2.push(size);
            v3.push(align);
            p.push(cnt);
            cnt += 1;
            nw += 1;
        } else {
            // delete a block
            let idx = rand_usize() % nw;
            let addr = v[p[idx]];
            let size = v2[p[idx]];
            let align = v3[p[idx]];
            GLOBAL_ALLOCATOR.dealloc(addr, size, align);
            nw -= 1;
            p[idx] = p[nw];
            p.pop();
        }
    }
    memory_chk();
    for idx in 0..nw {
        let addr = v[p[idx]];
        let size = v2[p[idx]];
        let align = v3[p[idx]];
        GLOBAL_ALLOCATOR.dealloc(addr, size, align);
    }
    let t1 = libax::time::Instant::now();
    println!("time: {:#?}", t1.duration_since(t0));
    println!("Align alloc test OK!");
}

#[no_mangle]
fn main() {
    align_test();
}
