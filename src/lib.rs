use std::{fs, io, thread};
use std::path::Path;
use std::time::Duration;

pub fn count_down(second:u64){
    for i in (0..=second).rev(){
        println!("{}", i);
        thread::sleep(Duration::from_secs(1));
    }
    println!("Countdown finished!");
}

pub fn read_and_print(file_path: &str)->io::Result<()>{
    // 如何打开项目根目录下的文件
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(file_path);
    // 尝试使用 io::Result<()> 作为返回值，并使用 ? 将错误向上传递。
    let contents = fs::read_to_string(path)?;
    println!("{}", contents);
    Ok(())
}

pub fn file_size(file_path: &str) -> Result<u64, &'static str>{
    // 如何获取文件大小
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(file_path);
    match fs::metadata(path){
        Ok(metadata) => Ok(metadata.len()),
        Err(_) => Err("File not found!")
    }
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
        read_and_print("test1.txt").expect("File not found!");
    }
}

#[cfg(test)]
mod test_file_size {
    use crate::file_size;
    
    // // 参考错误处理的模式
    // fn file_size_test() {
    //     let size = file_size("test1.txt").unwrap_or_else(|err|{
    //         println!("File not found!{err}");
    //         // 似乎不太能用在测试里
    //         process::exit(1);
    //     });
    //     println!("File size is {}", size);
    // }
    #[test]
    fn another_test(){
        match file_size("test1.txt") {
            Ok(size) => println!("File size is {} bytes", size),
            Err(err) => panic!("Something went wrong! {}", err)
        }
        
    }
}