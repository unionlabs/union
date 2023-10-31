use std::panic::{catch_unwind, AssertUnwindSafe};

use corosensei::Coroutine;

fn main() {
    println!("[main] creating coroutine");

    let mut coroutine = Coroutine::new(|yielder, ()| {
        println!("[coroutine] yielding 42");
        yielder.suspend(42);

        println!("[coroutine] panicking");
        panic!("foobar");
    });

    println!("[main] resuming coroutine");
    let result = catch_unwind(AssertUnwindSafe(|| coroutine.resume(())));
    println!(
        "[main] got value {} from coroutine",
        result.unwrap().as_yield().unwrap()
    );

    println!("[main] resuming coroutine");
    let result = catch_unwind(AssertUnwindSafe(|| coroutine.resume(())));
    println!(
        "[main] caught panic \"{}\" from coroutine",
        result.unwrap_err().downcast_ref::<&'static str>().unwrap()
    );

    println!("[main] exiting");
}

#[cfg(feature = "unwind")]
#[test]
fn panic() {
    main()
}
