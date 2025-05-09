#![no_std]
#![no_main]

use log::{debug, trace};
use ysos::*;
use ysos_kernel as ysos;

extern crate alloc;

boot::entry_point!(kernel_main);

pub fn kernel_main(boot_info: &'static boot::BootInfo) -> ! {
    ysos::init(boot_info);
    wait(spawn_init());
    trace!("000");
    ysos::shutdown();
}

pub fn spawn_init() -> proc::ProcessId {
    proc::spawn("shell").unwrap()
}
