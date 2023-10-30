#![feature(test)]

extern crate test;

use std::future::Future;

use async_executor::Executor;
use futures_lite::{future, prelude::*};

const TASKS: usize = 300;
const STEPS: usize = 300;
const LIGHT_TASKS: usize = 25_000;

static EX: Executor<'_> = Executor::new();

fn run(f: impl FnOnce()) {
    let (s, r) = async_channel::bounded::<()>(1);
    easy_parallel::Parallel::new()
        .each(0..num_cpus::get(), |_| future::block_on(EX.run(r.recv())))
        .finish(move || {
            let _s = s;
            f()
        });
}

#[bench]
fn create(b: &mut test::Bencher) {
    b.iter(move || {
        let ex = Executor::new();
        let task = ex.spawn(async {});
        future::block_on(ex.run(task));
    });
}

#[bench]
fn spawn_one(b: &mut test::Bencher) {
    run(|| {
        b.iter(move || {
            future::block_on(async { EX.spawn(async {}).await });
        });
    });
}

#[bench]
fn spawn_many(b: &mut test::Bencher) {
    run(|| {
        b.iter(move || {
            future::block_on(async {
                let mut tasks = Vec::new();
                for _ in 0..LIGHT_TASKS {
                    tasks.push(EX.spawn(async {}));
                }
                for task in tasks {
                    task.await;
                }
            });
        });
    });
}

#[bench]
fn spawn_recursively(b: &mut test::Bencher) {
    fn go(i: usize) -> impl Future<Output = ()> + Send + 'static {
        async move {
            if i != 0 {
                EX.spawn(async move {
                    let fut = go(i - 1).boxed();
                    fut.await;
                })
                .await;
            }
        }
    }

    run(|| {
        b.iter(move || {
            future::block_on(async {
                let mut tasks = Vec::new();
                for _ in 0..TASKS {
                    tasks.push(EX.spawn(go(STEPS)));
                }
                for task in tasks {
                    task.await;
                }
            });
        });
    });
}

#[bench]
fn yield_now(b: &mut test::Bencher) {
    run(|| {
        b.iter(move || {
            future::block_on(async {
                let mut tasks = Vec::new();
                for _ in 0..TASKS {
                    tasks.push(EX.spawn(async move {
                        for _ in 0..STEPS {
                            future::yield_now().await;
                        }
                    }));
                }
                for task in tasks {
                    task.await;
                }
            });
        });
    });
}
