#![no_std]
#![no_main]

use core::arch::asm;
use ysos::*;
use ysos_kernel as ysos;

extern crate alloc;

boot::entry_point!(kernel_main);

pub fn kernel_main(boot_info: &'static boot::BootInfo) -> ! {
    ysos::init(boot_info);

    // 触发Triple Fault
    trigger_triple_fault();

    // 以下代码不会被执行，因为系统会重启
    ysos::shutdown();
}

fn trigger_triple_fault() {
    // 创建并加载一个空的IDT
    // 这里我们只创建一个零大小的IDT，它是完全无效的
    unsafe {
        let null_idt = core::ptr::null::<u8>();
        let idt_descriptor = [0u8; 6]; // 空的IDTR

        // 加载无效的IDT
        asm!(
            "lidt [{}]",
            in(reg) &idt_descriptor,
            options(nostack)
        );

        // 触发一个异常（除以零异常）
        asm!(
            "mov eax, 1",
            "mov ecx, 0",
            "div ecx", // 除以零将触发异常
            options(nostack)
        );
    }
}
