use crate::*;
use alloc::string::{String, ToString};
use alloc::vec;

pub struct Stdin;
pub struct Stdout;
pub struct Stderr;

impl Stdin {
    fn new() -> Self {
        Self
    }

    pub fn read_line(&self) -> String {
        // 分配字符串用于存储结果
        let mut result = String::new();

        // 创建一个单字符缓冲区
        let mut buf = [0u8; 1];

        loop {
            // 从标准输入(fd=0)读取一个字符
            if let Some(n) = sys_read(0, &mut buf) {
                if n == 0 {
                    continue;
                }

                match buf[0] {
                    // 回车键（换行），结束读取
                    b'\n' | b'\r' => {
                        // 输出换行以回显
                        sys_write(1, b"\n");
                        break;
                    }
                    // 退格键或删除键
                    8 | 127 => {
                        if !result.is_empty() {
                            // 删除最后一个字符
                            result.pop();
                            // 回显退格效果（退格、空格、再退格）
                            sys_write(1, b"\x08 \x08");
                        }
                    }
                    // 普通可打印字符
                    _ => {
                        // 将字符追加到结果字符串
                        result.push(buf[0] as char);
                        // 回显字符
                        sys_write(1, &buf);
                    }
                }
            }
        }

        result
    }
}

impl Stdout {
    fn new() -> Self {
        Self
    }

    pub fn write(&self, s: &str) {
        sys_write(1, s.as_bytes());
    }
}

impl Stderr {
    fn new() -> Self {
        Self
    }

    pub fn write(&self, s: &str) {
        sys_write(2, s.as_bytes());
    }
}

pub fn stdin() -> Stdin {
    Stdin::new()
}

pub fn stdout() -> Stdout {
    Stdout::new()
}

pub fn stderr() -> Stderr {
    Stderr::new()
}
