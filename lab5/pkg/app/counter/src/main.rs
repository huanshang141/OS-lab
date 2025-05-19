#![no_std]
#![no_main]

use lib::*;

extern crate lib;

use sync::{Semaphore, SpinLock};
static LOCK: SpinLock = SpinLock::new();
static SEMA: Semaphore = Semaphore::new(0);
const THREAD_COUNT: usize = 8;
static mut COUNTER: isize = 0;

fn main() -> isize {
    SEMA.init(1);

    let mut pids = [0u16; THREAD_COUNT];

    for i in 0..THREAD_COUNT {
        let pid = sys_fork();
        if pid == 0 {
            do_counter_inc();
            sys_exit(0);
        } else {
            pids[i] = pid; // only parent knows child's pid
        }
    }

    let cpid = sys_get_pid();
    println!("process #{} holds threads: {:?}", cpid, &pids);
    sys_stat();

    for i in 0..THREAD_COUNT {
        println!("#{} waiting for #{}...", cpid, pids[i]);
        sys_wait_pid(pids[i]);
    }

    println!("COUNTER result: {}", unsafe { COUNTER });

    SEMA.remove();
    0
}

fn do_counter_inc() {
    for _ in 0..100 {
        // LOCK.acquire();
        SEMA.wait();
        inc_counter();
        SEMA.signal();
        // LOCK.release();
    }
}

/// Increment the counter
///
/// this function simulate a critical section by delay
/// DO NOT MODIFY THIS FUNCTION
fn inc_counter() {
    unsafe {
        delay();
        let mut val = COUNTER;
        delay();
        val += 1;
        delay();
        COUNTER = val;
    }
}

#[inline(never)]
#[unsafe(no_mangle)]
fn delay() {
    for _ in 0..0x100 {
        core::hint::spin_loop();
    }
}

entry!(main);
