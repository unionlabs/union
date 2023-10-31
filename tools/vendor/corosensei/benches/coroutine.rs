use corosensei::stack::DefaultStack;
use corosensei::{on_stack, Coroutine};
use criterion::measurement::Measurement;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn coroutine_switch<M: Measurement + 'static>(name: &str, c: &mut Criterion<M>) {
    let mut identity = Coroutine::new(|yielder, mut input| loop {
        input = yielder.suspend(input)
    });

    c.bench_function(name, |b| b.iter(|| identity.resume(black_box(0usize))));

    // Forcibly reset the coroutine so that this benchmarks works even when the
    // unwind feature is disabled.
    unsafe {
        identity.force_reset();
    }
}

fn coroutine_call<M: Measurement + 'static>(name: &str, c: &mut Criterion<M>) {
    // Don't count time spent allocating a stack.
    let mut stack = DefaultStack::default();

    c.bench_function(name, move |b| {
        b.iter(|| {
            let mut identity =
                Coroutine::<usize, (), usize, _>::with_stack(&mut stack, |_yielder, input| input);
            identity.resume(black_box(0usize))
        })
    });
}

fn stack_call<M: Measurement + 'static>(name: &str, c: &mut Criterion<M>) {
    // Don't count time spent allocating a stack.
    let mut stack = DefaultStack::default();

    c.bench_function(name, move |b| {
        b.iter(|| {
            on_stack(&mut stack, || {});
        })
    });
}

fn coroutine_switch_time(c: &mut Criterion) {
    coroutine_switch("coroutine_switch_time", c);
}
fn coroutine_call_time(c: &mut Criterion) {
    coroutine_call("coroutine_call_time", c);
}
fn stack_call_time(c: &mut Criterion) {
    stack_call("stack_call_time", c);
}

criterion_group!(
    name = time;
    config = Criterion::default();
    targets = coroutine_switch_time, coroutine_call_time, stack_call_time
);

cfg_if::cfg_if! {
    if #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] {
        use criterion_cycles_per_byte::CyclesPerByte;

        fn coroutine_switch_cycles(c: &mut Criterion<CyclesPerByte>) {
            coroutine_switch("coroutine_switch_cycles", c);
        }
        fn coroutine_call_cycles(c: &mut Criterion<CyclesPerByte>) {
            coroutine_call("coroutine_call_cycles", c);
        }
        fn stack_call_cycles(c: &mut Criterion<CyclesPerByte>) {
            stack_call("stack_call_cycles", c);
        }

        criterion_group!(
            name = cycles;
            config = Criterion::default().with_measurement(CyclesPerByte);
            targets = coroutine_switch_cycles, coroutine_call_cycles, stack_call_cycles
        );

        criterion_main!(cycles, time);
    } else {
        criterion_main!(time);
    }
}
