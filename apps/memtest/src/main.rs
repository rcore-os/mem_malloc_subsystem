#![no_std]
#![no_main]

#[macro_use]
extern crate libax;
extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use libax::rand::{rand_u32,rand_usize};
use axalloc::GLOBAL_ALLOCATOR;

///memory chk
pub fn memory_chk(){}

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
    let t0 = libax::time::Instant::now();
    test_vec(3000000);
    test_vec_2(30000,64);
    test_vec_2(7500,520);
    test_btree_map(50000);
    test_vec_3(10000,32,64);
    let t1 = libax::time::Instant::now();
    println!("time: {:#?}",t1.duration_since(t0));
    println!("Basic alloc test OK!");
    println!("*****");
}


pub fn new_mem(size: usize, align: usize) -> usize{
    if let Ok(ptr) = GLOBAL_ALLOCATOR.alloc(size,align){
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
            GLOBAL_ALLOCATOR.dealloc(addr, size as usize,align);
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
        GLOBAL_ALLOCATOR.dealloc(addr, size as usize,align);
    }
    let t1 = libax::time::Instant::now();
    println!("time: {:#?}",t1.duration_since(t0));
    println!("Align alloc test OK!");
    println!("*****");
}





#[no_mangle]
fn main() {
    println!("Running memory tests...");
    basic_test();
    align_test();
    println!("Memory tests run OK!");
}
