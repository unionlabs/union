use std::ptr;
use std::thread;

fn main() {
    let x = ptr::null_mut();

    thread::spawn(|| {
        println!("{:?}", x)
    });
}
