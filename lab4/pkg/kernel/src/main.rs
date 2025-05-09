#![no_std]
#![no_main]

use log::debug;
use ysos::*;
use ysos_kernel as ysos;

extern crate alloc;

boot::entry_point!(kernel_main);

pub fn kernel_main(boot_info: &'static boot::BootInfo) -> ! {
    ysos::init(boot_info);
    spawn_init();
    //loop{}
    loop {
        print!("[>] ");
        let line = input::get_line();
        match line.trim() {
            "exit" => break,
            "ps" => {
                //print!("\n");
                ysos::proc::print_process_list();
            }
            _ => println!("[=] {}", line),
        }
    }
    ysos::shutdown();
}

pub fn spawn_init() -> proc::ProcessId {
    // NOTE: you may want to clear the screen before starting the shell
    // print_serial!("\x1b[1;1H\x1b[2J");

    proc::list_app();
    proc::spawn("hello").unwrap()
}
