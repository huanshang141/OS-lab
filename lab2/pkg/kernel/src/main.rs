#![no_std]
#![no_main]

use ysos::*;
use ysos_kernel as ysos;

extern crate alloc;

boot::entry_point!(kernel_main);

pub fn kernel_main(boot_info: &'static boot::BootInfo) -> ! {
    ysos::init(boot_info);

    println!("正在尝试访问非法内存地址以触发page fault...");

    unsafe {
        let invalid_ptr: *mut u8 = 0xffffffff00000000 as *mut u8;
        println!("尝试访问地址: 0x{:x}", invalid_ptr as usize);
        let value = *invalid_ptr;
        println!("读取值: {}", value);
    }

    ysos::shutdown();
}
