#[cfg(test)]
mod test_count_down {
    use lab0::count_down;

    #[test]
    fn count_down_test() {
        count_down(1);
    }
}

#[cfg(test)]
mod test_read_and_print {
    use lab0::read_and_print;

    #[test]
    fn read_and_print_test(){
        read_and_print("test1.txt").expect("File not found!");
    }
}

#[cfg(test)]
mod test_file_size {
    use lab0::file_size;

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
        let path = env!("CARGO_MANIFEST_DIR");
        println!("path: {}", path);
        match file_size("test.txt") {
            Ok(size) => println!("File size is {} bytes", size),
            Err(err) => panic!("Something went wrong! {}", err)
        }
        let num = file_size("").unwrap();

    }
}