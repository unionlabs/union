use std::thread;

use fragile::Sticky;

fn main() {
    fragile::stack_token!(tok);

    // creating and using a fragile object in the same thread works
    let val = Sticky::new(true);
    println!("debug print in same thread: {:?}", &val);
    println!("try_get in same thread: {:?}", val.try_get(tok));

    // once send to another thread it stops working
    thread::spawn(move || {
        fragile::stack_token!(tok);
        println!("debug print in other thread: {:?}", &val);
        println!("try_get in other thread: {:?}", val.try_get(tok));
    })
    .join()
    .unwrap();
}
