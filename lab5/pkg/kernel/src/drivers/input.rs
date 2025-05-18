use crate::drivers::serial;
use alloc::string::String;
use core::hint::spin_loop;
use crossbeam_queue::ArrayQueue;

type Key = u8;

lazy_static! {
    static ref INPUT_BUF: ArrayQueue<Key> = ArrayQueue::new(128);
}

#[inline]
pub fn push_key(key: Key) {
    if INPUT_BUF.push(key).is_err() {
        warn!("Input buffer is full. Dropping key '{:?}'", key);
    }
}

#[inline]
pub fn try_pop_key() -> Option<Key> {
    INPUT_BUF.pop()
}

pub fn pop_key() -> Key {
    loop {
        if let Some(key) = try_pop_key() {
            return key;
        }
        spin_loop();
    }
}

pub fn get_line() -> String {
    let mut line = String::with_capacity(128);

    loop {
        let key = pop_key();

        if key == b'\n' || key == b'\r' {
            println!();
            break;
        }

        if key == 0x08 || key == 0x7F {
            if !line.is_empty() {
                line.pop();
                serial::backspace();
            }
            continue;
        }

        print!("{}", key as char);
        line.push(key as char);
    }
    line
}
