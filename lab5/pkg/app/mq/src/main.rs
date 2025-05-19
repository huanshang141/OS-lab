#![no_std]
#![no_main]

use lib::*;

extern crate lib;

use sync::{Semaphore, SpinLock};

// 进程数量配置
const PRODUCER_COUNT: usize = 8;
const CONSUMER_COUNT: usize = 8;
const TOTAL_THREADS: usize = PRODUCER_COUNT + CONSUMER_COUNT;
const MESSAGES_PER_THREAD: usize = 10;
const QUEUE_CAPACITY: usize = 8; // 可以测试 1, 4, 8, 16

// 信号量定义
static MUTEX: Semaphore = Semaphore::new(1); // 互斥锁
static EMPTY: Semaphore = Semaphore::new(2); // 表示空槽位数量
static FULL: Semaphore = Semaphore::new(3); // 表示已用槽位数量
static PRINT_LOCK: SpinLock = SpinLock::new(); // 用于打印互斥

// 消息队列结构
static mut QUEUE: [usize; QUEUE_CAPACITY] = [0; QUEUE_CAPACITY];
static mut FRONT: usize = 0;
static mut REAR: usize = 0;
static mut ITEMS_COUNT: usize = 0;
static mut TOTAL_PRODUCED: usize = 0;
static mut TOTAL_CONSUMED: usize = 0;

fn main() -> isize {
    // 初始化信号量
    MUTEX.init(1);
    EMPTY.init(QUEUE_CAPACITY); // 初始时队列为空，所有槽位都可用
    FULL.init(0); // 初始时没有可消费的消息

    let mut pids = [0u16; TOTAL_THREADS];

    // 创建生产者进程
    for i in 0..PRODUCER_COUNT {
        let pid = sys_fork();
        if pid == 0 {
            producer(i);
            sys_exit(0);
        } else {
            pids[i] = pid;
        }
    }

    // 创建消费者进程
    for i in 0..CONSUMER_COUNT {
        let pid = sys_fork();
        if pid == 0 {
            consumer(i);
            sys_exit(0);
        } else {
            pids[i + PRODUCER_COUNT] = pid;
        }
    }

    let parent_pid = sys_get_pid();
    PRINT_LOCK.acquire();
    println!("父进程 #{} 创建了 {} 个子进程", parent_pid, TOTAL_THREADS);
    println!("生产者 PID: {:?}", &pids[0..PRODUCER_COUNT]);
    println!("消费者 PID: {:?}", &pids[PRODUCER_COUNT..]);
    PRINT_LOCK.release();

    // 输出系统进程状态
    sys_stat();

    // 等待所有子进程结束
    for i in 0..TOTAL_THREADS {
        PRINT_LOCK.acquire();
        println!("父进程 #{} 正在等待子进程 #{}", parent_pid, pids[i]);
        PRINT_LOCK.release();
        sys_wait_pid(pids[i]);
    }

    // 输出最终队列状态
    PRINT_LOCK.acquire();
    println!("所有进程已完成");

    println!("消息队列容量: {}", QUEUE_CAPACITY);
    unsafe {
        let total_produced = TOTAL_PRODUCED;
        let total_consumed = TOTAL_CONSUMED;
        let items_count = ITEMS_COUNT;

        println!("总共生产消息: {}", total_produced);
        println!("总共消费消息: {}", total_consumed);
        println!("最终队列状态: {} 个消息", items_count);

        if items_count == 0 {
            println!("队列为空，符合预期！");
        } else {
            println!("错误：队列应该为空，但仍有 {} 个消息", items_count);
        }
    }

    PRINT_LOCK.release();

    // 清理资源
    MUTEX.remove();
    EMPTY.remove();
    FULL.remove();

    0
}

fn producer(id: usize) {
    let pid = sys_get_pid();
    PRINT_LOCK.acquire();
    println!("生产者 #{} (PID: {}) 已启动", id, pid);
    PRINT_LOCK.release();

    for msg_id in 0..MESSAGES_PER_THREAD {
        // 生成消息 (id * 100 + 消息序号)
        let message = id * 100 + msg_id;

        // 等待空槽位
        PRINT_LOCK.acquire();
        // println!("生产者 #{} 等待空槽位...", id);
        PRINT_LOCK.release();
        EMPTY.wait();

        // 获取互斥锁
        MUTEX.wait();

        // 将消息放入队列
        unsafe {
            QUEUE[REAR] = message;
            REAR = (REAR + 1) % QUEUE_CAPACITY;
            ITEMS_COUNT += 1;
            TOTAL_PRODUCED += 1;

            let items_count = ITEMS_COUNT;

            PRINT_LOCK.acquire();
            println!(
                "生产者 #{} 生产消息: {} (队列大小: {}/{})",
                id, message, items_count, QUEUE_CAPACITY
            );
            PRINT_LOCK.release();
        }

        // 释放互斥锁
        MUTEX.signal();
        // 增加可消费消息计数
        FULL.signal();

        // 模拟工作时间
        delay();
    }

    PRINT_LOCK.acquire();
    println!("生产者 #{} (PID: {}) 已完成所有消息生产", id, pid);
    PRINT_LOCK.release();
}

fn consumer(id: usize) {
    let pid = sys_get_pid();
    PRINT_LOCK.acquire();
    println!("消费者 #{} (PID: {}) 已启动", id, pid);
    PRINT_LOCK.release();

    for _ in 0..MESSAGES_PER_THREAD {
        // 等待有消息可消费
        PRINT_LOCK.acquire();
        // println!("消费者 #{} 等待消息...", id);
        PRINT_LOCK.release();
        FULL.wait();

        // 获取互斥锁
        MUTEX.wait();

        // 从队列取出消息
        let message;
        unsafe {
            message = QUEUE[FRONT];
            FRONT = (FRONT + 1) % QUEUE_CAPACITY;
            ITEMS_COUNT -= 1;
            TOTAL_CONSUMED += 1;

            let items_count = ITEMS_COUNT;

            PRINT_LOCK.acquire();
            println!(
                "消费者 #{} 消费消息: {} (队列大小: {}/{})",
                id, message, items_count, QUEUE_CAPACITY
            );
            PRINT_LOCK.release();
        }

        // 释放互斥锁
        MUTEX.signal();
        // 增加空槽位计数
        EMPTY.signal();

        // 模拟处理时间
        delay();
    }

    PRINT_LOCK.acquire();
    println!("消费者 #{} (PID: {}) 已完成所有消息消费", id, pid);
    PRINT_LOCK.release();
}

#[inline(never)]
fn delay() {
    for _ in 0..0x100 {
        core::hint::spin_loop();
    }
}

entry!(main);
