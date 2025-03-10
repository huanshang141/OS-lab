use std::path::Path;
use std::time::Duration;
use std::{fs, io, thread};

pub fn count_down(second: u64) {
    for i in (0..=second).rev() {
        println!("{}", i);
        thread::sleep(Duration::from_secs(1));
    }
    println!("Countdown finished!");
}

pub fn read_and_print(file_path: &str) -> io::Result<()> {
    // 如何打开项目根目录下的文件
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(file_path);
    // 尝试使用 io::Result<()> 作为返回值，并使用 ? 将错误向上传递。
    let contents = fs::read_to_string(path)?;
    println!("{}", contents);
    Ok(())
}

pub fn file_size(file_path: &str) -> Result<u64, &'static str> {
    // Get the file size
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(file_path);
    let metadata = fs::metadata(path).map_err(|_| "File not found!")?;

    if metadata.is_file() {
        Ok(metadata.len())
    } else {
        Err("Not a file")
    }
}

pub fn humanized_size(size: u64) -> (f64, &'static str) {
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB"];
    let mut size = size as f64;
    let mut i = 0;
    while size >= 1024.0 && i < UNITS.len() - 1 {
        size /= 1024.0;
        i += 1;
    }
    (size, UNITS[i])
}
#[test]
fn test_humanized_size() {
    let byte_size = 1554056;
    let (size, unit) = humanized_size(byte_size);
    assert_eq!(
        "Size :  1.4821 MiB",
        format!("Size :  {:.4} {}", size, unit)
    );
}
//  use llm
pub enum Shape {
    //rectangle(f64,f64)
    //元组结构体与命名结构体
    Rectangle { width: f64, height: f64 },
    Circle { radius: f64 },
}

impl Shape {
    pub fn area(&self) -> f64 {
        match self {
            Shape::Rectangle { width, height } => width * height,
            Shape::Circle { radius } => std::f64::consts::PI * radius * radius,
        }
    }
}
#[derive(PartialEq, Eq, Debug)]
pub struct UniqueId(pub u16);

impl UniqueId {
    pub fn new() -> Self {
        // Static variable to keep track of the next ID
        static mut NEXT_ID: u16 = 0;

        // This is unsafe because we're modifying static mutable data
        // which could cause data races if called from multiple threads
        let id = unsafe {
            let id = NEXT_ID;
            NEXT_ID = NEXT_ID.wrapping_add(1);
            id
        };

        UniqueId(id)
    }

    pub fn get(&self) -> u16 {
        self.0
    }
}
#[test]
fn test_area() {
    let rectangle = Shape::Rectangle {
        width: 10.0,
        height: 20.0,
    };
    let circle = Shape::Circle { radius: 10.0 };

    assert_eq!(rectangle.area(), 200.0);
    assert_eq!(circle.area(), 314.1592653589793);
}
#[test]
fn test_unique_id() {
    let id1 = UniqueId::new();
    let id2 = UniqueId::new();
    assert_ne!(id1, id2);
}

#[test]
fn test_unique_id_thread_unsafe() {
    use std::collections::HashSet;
    use std::sync::{Arc, Mutex};
    use std::thread;

    // 创建一个线程安全的集合来存储所有生成的ID
    let ids = Arc::new(Mutex::new(HashSet::new()));

    // 创建多个线程，每个线程生成多个UniqueId
    let mut handles = vec![];
    for _ in 0..10 {
        let ids_clone = Arc::clone(&ids);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let id = UniqueId::new();
                let mut ids = ids_clone.lock().unwrap();
                ids.insert(id.get());
            }
        });
        handles.push(handle);
    }

    // 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }

    // 检查生成的ID数量
    let ids = ids.lock().unwrap();
    // 如果没有重复的ID，应该有 10 * 100 = 1000 个ID
    // 如果有重复的ID，数量会少于1000
    println!("生成了 {} 个唯一ID，期望值为1000", ids.len());
    assert!(
        ids.len() < 1000,
        "预期由于线程安全问题会出现重复ID，但未发现重复。此测试可能不稳定。"
    );
}
