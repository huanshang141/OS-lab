use lab0::{count_down, file_size, read_and_print};
use std::io;
fn main() {
    count_down(5);
    if let Err(e) = read_and_print(""){
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
            },
            Err(e) => {
                println!("Error: {:?}", e);
            }
        };
    }
}
