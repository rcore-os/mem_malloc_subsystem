#![no_std]
#![no_main]

#[macro_use]
extern crate libax;
use axalloc::GLOBAL_ALLOCATOR;
use core::sync::atomic::{AtomicUsize, Ordering};
use libax::rand::{rand_u32, srand};
use libax::thread;
use libax::time::Duration;
use libax::vec::Vec;

const NUM_TASKS: usize = 10;
const MUN_TURN: usize = 100;
const NUM_ARRAY_PRE_THREAD: usize = 1000;

static mut MEMORY_POOL: Vec<AtomicUsize> = Vec::new(); //NUM_TASKS * NUM_ARRAY_PRE_THREAD] = [AtomicUsize::new(0); NUM_TASKS * NUM_ARRAY_PRE_THREAD];
static mut MEMORY_SIZE: Vec<AtomicUsize> = Vec::new(); //NUM_TASKS * NUM_ARRAY_PRE_THREAD] = [AtomicUsize::new(0); NUM_TASKS * NUM_ARRAY_PRE_THREAD];

static FINISHED_TASKS: AtomicUsize = AtomicUsize::new(0);

#[no_mangle]
fn main() {
    srand(2333);
    println!("Multi thread memory allocation test begin.");
    unsafe {
        for i in 0..NUM_TASKS * NUM_ARRAY_PRE_THREAD {
            MEMORY_POOL.push(AtomicUsize::new(0));
            MEMORY_SIZE.push(AtomicUsize::new(0));
        }
    }

    for turn in 0..MUN_TURN {
        // alloc memory and free half (only free the memory allocated by itself)
        FINISHED_TASKS.store(0, Ordering::Relaxed);
        for i in 0..NUM_TASKS {
            thread::spawn(move || {
                unsafe {
                    let tid = i;
                    for j in 0..NUM_ARRAY_PRE_THREAD {
                        let size = (1_usize << (rand_u32() % 12)) + (1_usize << (rand_u32() % 12));
                        let idx = j * NUM_TASKS + tid;
                        if let Ok(ptr) = GLOBAL_ALLOCATOR.alloc(size, 8) {
                            //println!("successfully alloc: {:#?} {:#x} {:#?}", idx,ptr,size);
                            MEMORY_POOL[idx].store(ptr, Ordering::Relaxed);
                            MEMORY_SIZE[idx].store(size, Ordering::Relaxed);
                        } else {
                            panic!("multi thread test: alloc err,");
                        }
                    }

                    for j in (NUM_ARRAY_PRE_THREAD >> 1)..NUM_ARRAY_PRE_THREAD {
                        let idx = j * NUM_TASKS + tid;
                        let addr = MEMORY_POOL[idx].load(Ordering::Relaxed);
                        let size = MEMORY_SIZE[idx].load(Ordering::Relaxed);
                        //println!("dealloc: {:#?} {:#x} {:#?}", idx,addr,size);
                        GLOBAL_ALLOCATOR.dealloc(addr as _, size, 8);
                        MEMORY_POOL[idx].store(0_usize, Ordering::Relaxed);
                        MEMORY_SIZE[idx].store(0_usize, Ordering::Relaxed);
                    }
                }
                FINISHED_TASKS.fetch_add(1, Ordering::Relaxed);
            });
        }

        while FINISHED_TASKS.load(Ordering::Relaxed) < NUM_TASKS {
            thread::sleep(Duration::from_millis(10));
        }

        // realloc memory and free all
        FINISHED_TASKS.store(0, Ordering::Relaxed);
        for i in 0..NUM_TASKS {
            thread::spawn(move || {
                unsafe {
                    let tid = i;
                    for j in 0..(NUM_ARRAY_PRE_THREAD >> 1) {
                        let size = (1_usize << (rand_u32() % 12)) + (1_usize << (rand_u32() % 12));
                        let idx = NUM_TASKS * NUM_ARRAY_PRE_THREAD / 2
                            + tid * NUM_ARRAY_PRE_THREAD / 2
                            + j;
                        if let Ok(ptr) = GLOBAL_ALLOCATOR.alloc(size, 8) {
                            MEMORY_POOL[idx].store(ptr, Ordering::Relaxed);
                            MEMORY_SIZE[idx].store(size, Ordering::Relaxed);
                        } else {
                            panic!("multi thread test: alloc err,");
                        }
                    }

                    for j in 0..NUM_ARRAY_PRE_THREAD {
                        let idx = j * NUM_TASKS + tid;
                        while MEMORY_SIZE[idx].load(Ordering::Relaxed) == 0 {
                            thread::sleep(Duration::from_millis(10));
                        }
                        let addr = MEMORY_POOL[idx].load(Ordering::Relaxed);
                        let size = MEMORY_SIZE[idx].load(Ordering::Relaxed);
                        GLOBAL_ALLOCATOR.dealloc(addr as _, size, 8);
                        MEMORY_POOL[idx].store(0_usize, Ordering::Relaxed);
                        MEMORY_SIZE[idx].store(0_usize, Ordering::Relaxed);
                    }
                }
                FINISHED_TASKS.fetch_add(1, Ordering::Relaxed);
            });
        }
        while FINISHED_TASKS.load(Ordering::Relaxed) < NUM_TASKS {
            thread::sleep(Duration::from_millis(10));
        }
    }
    println!("Multi thread memory allocation test OK!");
}
