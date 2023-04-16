#![no_std]
#![no_main]

#[macro_use]
extern crate libax;
extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use libax::rand;

fn test_vec(n: usize) {
    //const N: usize = 1_000_000;
    //let mut v = Vec::with_capacity(N);
    let mut v = Vec::new();
    for _ in 0..n {
        //println!("vector push 1");
        v.push(rand::rand_u32());
    }
    //v.sort();
    //for i in 0..n - 1 {
    //    assert!(v[i] <= v[i + 1]);
    //}
    println!("test_vec() OK!");
}

fn test_btree_map(n: usize) {
    //const N: usize = 20;
    let mut m = BTreeMap::new();
    for _ in 0..n {
        //println!("test btree map: {:#?}",i);
        if rand::rand_usize() % 5 == 0 && !m.is_empty() {
            m.pop_first();
        } else {
            let value = rand::rand_usize();
            let key = alloc::format!("key_{value}");
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
    //let mut v = Vec::with_capacity(N);
    let mut v:Vec<Vec<usize>> = Vec::new();
    for _ in 0..n {
        //println!("vector push {:#?}",i);
        let mut tmp: Vec<usize> = Vec::with_capacity(m);
        for _ in 0..m {
            tmp.push(rand::rand_usize());
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
        let o: usize = rand::rand_usize() % (i + 1);
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
            v[i].push(rand::rand_usize());
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
            v[4 * n + i].push(rand::rand_usize());
        }
    }
    println!("test_vec3() OK!");
}

#[no_mangle]
fn main() {
    println!("Running memory tests...");
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
    println!("Memory tests run OK!");
}
