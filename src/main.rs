use std::thread::sleep;
use std::time::Duration;

fn main() {
    let mut num = 10;

    for _ in 0..10 {
        println!("num: {}", num);

        num -= 1;

        sleep(Duration::from_secs(1))
    }
}
