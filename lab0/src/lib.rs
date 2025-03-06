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
        Ok(metadata) => {
            // metadata能处理文件夹，所以要判定一下，不然空输入也会读len
            if metadata.is_file(){
                Ok(metadata.len())
            }
            else {
                Err("Not a file")
            }
        },
        Err(_) => Err("File not found!")
    }
}
