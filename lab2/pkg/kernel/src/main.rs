#![no_std]
#![no_main]

use core::arch::asm;
use ysos::*;
use ysos_kernel as ysos;

extern crate alloc;

boot::entry_point!(kernel_main);

pub fn kernel_main(boot_info: &'static boot::BootInfo) -> ! {
    ysos::init(boot_info);

    // 触发除零异常
    println!("即将触发除零异常...");
    unsafe {
        asm!(
            "mov eax, 1",
            "mov ecx, 0",
            "div ecx",
            options(nomem, nostack)
        );
    }

    // 下面的代码不会被执行到
    ysos::shutdown();
}
