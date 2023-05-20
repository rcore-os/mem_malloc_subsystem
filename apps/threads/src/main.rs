#![no_std]
#![no_main]

#[macro_use]
extern crate libax;
extern crate alloc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use libax::task;
use libax::rand::{rand_u32,rand_usize};

const NUM_TASKS: usize = 10;
const M: usize = 10000;
static FINISHED_TASKS: AtomicUsize = AtomicUsize::new(0);

#[no_mangle]
fn main() {
    let mut sum = 0;
    for i in 0..NUM_TASKS {
        task::spawn(move || {
            println!("Hello, task {}! id = {:?}", i, task::current().id());
            let mut tmp: Vec<usize> = Vec::with_capacity(M);
            for _ in 0..M {
                let x = rand_usize();
                tmp.push(x);
                sum = sum + x;

            }

            //#[cfg(not(feature = "preempt"))]
            //task::yield_now();

            let order = FINISHED_TASKS.fetch_add(1, Ordering::Relaxed);
            if option_env!("SMP") == Some("1") {
                assert!(order == i); // FIFO scheduler
            }

            println!("task {:#?}: {:#?}",i,sum);
        });
    }
    println!("Hello, main task!");
    
    while FINISHED_TASKS.load(Ordering::Relaxed) < NUM_TASKS {
        #[cfg(not(feature = "preempt"))]
        task::yield_now();
    }
    println!("main task: {:#?}",sum);
    println!("Task yielding tests run OK!");
}
