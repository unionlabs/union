extern crate atty;

use atty::{is, Stream};

fn main() {
    println!("stdout? {}", is(Stream::Stdout));
    println!("stderr? {}", is(Stream::Stderr));
    println!("stdin? {}", is(Stream::Stdin));
}
