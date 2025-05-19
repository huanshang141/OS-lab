#![no_std]
#![no_main]

use lib::*;

extern crate lib;

fn main() -> isize {
    loop {
        print!("[>] ");
        let line = stdin().read_line();
        match line.trim() {
            "exit" => break,
            "app" => sys_list_app(),
            "ps" => sys_stat(),
            "hello" => {
                sys_wait_pid(sys_spawn("hello"));
            }
            "fac" => {
                sys_wait_pid(sys_spawn("fac"));
            }
            "clear" => {
                print!("\x1b[2J\x1b[1;1H");
            }
            "fork" => {
                sys_wait_pid(sys_spawn("fork"));
            }
            "shell" => {
                sys_wait_pid(sys_spawn("shell"));
            }
            "help" => {
                print_help();
            }
            _ => println!("[=] {}", line),
        }
    }
    0
}

entry!(main);
fn print_help() {
    println!(
        "22361058\n\
        Commands available:\n\
        exit            - 退出 shell\n\
        app             - 列出所有可用的应用程序\n\
        ps              - 显示系统进程状态\n\
        hello           - 运行 hello world 应用程序\n\
        fac             - 运行阶乘计算应用程序\n\
        clear           - 清除屏幕\n\
        fork            - 运行 fork 测试应用程序\n\
        help            - 显示此帮助信息"
    );
}
