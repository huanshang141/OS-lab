use std::thread;
use std::time::Duration;

pub fn count_down(second:u64){
    for i in (0..=second).rev(){
        println!("{}", i);
        thread::sleep(Duration::from_secs(1));
    }
    println!("Countdown finished!");
}
#[cfg(test)]
mod tests {
    use crate::count_down;

    #[test]
    fn count_down_test() {
        count_down(1);
    }
}