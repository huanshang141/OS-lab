#![no_std]
#![no_main]

extern crate lib;
use lib::*;
use sync::{Semaphore, SpinLock};

const PHILOSOPHER_COUNT: usize = 5;

// 筷子的互斥锁
static CHOPSTICKS: [SpinLock; PHILOSOPHER_COUNT] = [
    SpinLock::new(),
    SpinLock::new(),
    SpinLock::new(),
    SpinLock::new(),
    SpinLock::new(),
];

// 限制同时进餐的哲学家数量，避免死锁
// static DINING_SEM: Semaphore = Semaphore::new(1);

fn main() -> isize {
    // DINING_SEM.init(PHILOSOPHER_COUNT - 1); // 最多允许 N-1 个哲学家同时尝试拿筷子

    println!("哲学家就餐问题开始");

    let mut pids = [0u16; PHILOSOPHER_COUNT];

    for i in 0..PHILOSOPHER_COUNT {
        let pid = sys_fork();
        if pid == 0 {
            // 子进程: 哲学家行为
            philosopher(i);
            sys_exit(0);
        } else {
            // 父进程: 记录子进程ID
            pids[i] = pid;
        }
    }

    // 等待所有哲学家就餐结束
    for pid in pids {
        sys_wait_pid(pid);
    }

    // DINING_SEM.remove();
    println!("晚餐结束");
    0
}

fn philosopher(id: usize) {
    // 使用进程ID和其他数据作为简单的随机数源
    let seed = sys_get_pid() as u64 + id as u64;

    for round in 0..5 {
        // 每个哲学家尝试就餐5次
        // 思考
        println!("哲学家 {} 正在思考... (第 {} 轮)", id, round);
        random_delay(seed);

        println!("哲学家 {} 饿了... (第 {} 轮)", id, round);

        // 请求进入临界区
        // DINING_SEM.wait();

        // 尝试拿起左边筷子
        let left = id;
        println!("哲学家 {} 尝试拿起左边筷子 {}", id, left);
        CHOPSTICKS[left].acquire();
        println!("哲学家 {} 拿起了左边筷子 {}", id, left);

        // 引入随机延迟，增加死锁概率
        // small_delay();

        // 尝试拿起右边筷子
        let right = (id + 1) % PHILOSOPHER_COUNT;
        println!("哲学家 {} 尝试拿起右边筷子 {}", id, right);
        CHOPSTICKS[right].acquire();
        println!("哲学家 {} 拿起了右边筷子 {}", id, right);

        // 进餐
        println!("哲学家 {} 正在进餐... (第 {} 轮)", id, round);
        random_delay(seed + 100);

        // 放下筷子
        CHOPSTICKS[right].release();
        println!("哲学家 {} 放下了右边筷子 {}", id, right);

        CHOPSTICKS[left].release();
        println!("哲学家 {} 放下了左边筷子 {}", id, left);

        // 离开临界区
        // DINING_SEM.signal();

        // 随机休息一段时间
        random_delay(seed + 200);
    }

    println!("哲学家 {} 离开了餐桌", id);
}

// 简单的伪随机延迟
fn random_delay(seed: u64) {
    let factor = (seed % 5 + 1) * 10;
    for _ in 0..factor {
        delay();
    }
}

// 较小的延迟
fn small_delay() {
    for _ in 0..5 {
        delay();
    }
}

// 延迟函数
fn delay() {
    for _ in 0..0x1000 {
        core::hint::spin_loop();
    }
}

entry!(main);
