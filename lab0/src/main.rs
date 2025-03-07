use lab0::{count_down, file_size, read_and_print};
use std::io;
fn main() {
    println!("\x1b[32mINFO: \x1b[0mHello, world!");
    //终端输出WARNING: I'm a teapot，其中整条信息为黄色加粗，WARNING添加下划线
    println!("\x1b[33;1;4mWARNING\x1b[0m: \x1b[33mI'm a teapot\x1b[0m");
    //终端输出ERROR: KERNEL PANIC!!!，颜色为红色，加粗，并居中
    let message = "ERROR: KERNEL PANIC!!!";
    let terminal_width = 80; // Assuming terminal width of 80 characters
    let padding = (terminal_width - message.len()) / 2;
    println!(
        "{:>width$}\x1b[31;1m{}\x1b[0m",
        "",
        message,
        width = padding
    );

    count_down(5);
    if let Err(e) = read_and_print("") {
        println!("Error: {:?}", e);
    }

    loop {
        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .expect("Failed to read line");
        let line = line.trim();
        // println!("{}",line);
        if line == "exit" {
            break;
        }
        match file_size(&line) {
            Ok(size) => {
                println!("{} bytes", size);
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        };
    }
}
