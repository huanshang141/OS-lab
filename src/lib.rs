use std::{fs, thread};
use std::path::Path;
use std::time::Duration;

pub fn count_down(second:u64){
    for i in (0..=second).rev(){
        println!("{}", i);
        thread::sleep(Duration::from_secs(1));
    }
    println!("Countdown finished!");
}

pub fn read_and_print(file_path: &str){
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(file_path);
    let contents = fs::read_to_string(path).expect("File not found!");
    println!("{}", contents);
}

#[cfg(test)]
mod test_count_down {
    use crate::count_down;

    #[test]
    fn count_down_test() {
        count_down(1);
    }
}

#[cfg(test)]
mod test_read_and_print {
    use crate::read_and_print;

    #[test]
    fn read_and_print_test(){
        read_and_print("test.txt");
    }
}