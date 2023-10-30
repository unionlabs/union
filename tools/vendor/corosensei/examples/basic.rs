use corosensei::{Coroutine, CoroutineResult};

fn main() {
    println!("[main] creating coroutine");

    let mut coroutine = Coroutine::new(|yielder, input| {
        println!("[coroutine] coroutine started with input {}", input);
        for i in 0..5 {
            println!("[coroutine] yielding {}", i);
            let input = yielder.suspend(i);
            println!("[coroutine] got {} from parent", input)
        }
        println!("[coroutine] exiting coroutine");
    });

    let mut counter = 100;
    loop {
        println!("[main] resuming coroutine with argument {}", counter);
        match coroutine.resume(counter) {
            CoroutineResult::Yield(i) => println!("[main] got {:?} from coroutine", i),
            CoroutineResult::Return(()) => break,
        }

        counter += 1;
    }

    println!("[main] exiting");
}

#[test]
fn basic() {
    main()
}
