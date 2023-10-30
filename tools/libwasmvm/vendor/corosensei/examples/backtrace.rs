use backtrace::Backtrace;
use corosensei::Coroutine;

#[inline(never)]
fn sub_function(coroutine: &mut Coroutine<(), (), ()>) {
    coroutine.resume(());
}

fn main() {
    let mut coroutine = Coroutine::new(move |_, ()| {
        let trace = Backtrace::new();
        println!("{:?}", trace);
    });

    sub_function(&mut coroutine);
}

#[test]
fn backtrace() {
    main()
}
