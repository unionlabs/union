use std::thread;

use fragile::Fragile;

fn main() {
    // creating and using a fragile object in the same thread works
    let val = Fragile::new(true);
    println!("debug print in same thread: {:?}", &val);
    println!("try_get in same thread: {:?}", val.try_get());

    // once send to another thread it stops working
    thread::spawn(move || {
        println!("debug print in other thread: {:?}", &val);
        println!("try_get in other thread: {:?}", val.try_get());
    })
    .join()
    .unwrap();
}
