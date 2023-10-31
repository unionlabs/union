#![cfg(feature = "macros")]
#![allow(clippy::blacklisted_name)]
use std::sync::Arc;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test as test;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test as maybe_tokio_test;

#[cfg(not(target_arch = "wasm32"))]
use tokio::test as maybe_tokio_test;

use tokio::sync::{oneshot, Semaphore};
use tokio_test::{assert_pending, assert_ready, task};

#[maybe_tokio_test]
async fn sync_one_lit_expr_comma() {
    let foo = tokio::join!(async { 1 },);

    assert_eq!(foo, (1,));
}

#[maybe_tokio_test]
async fn sync_one_lit_expr_no_comma() {
    let foo = tokio::join!(async { 1 });

    assert_eq!(foo, (1,));
}

#[maybe_tokio_test]
async fn sync_two_lit_expr_comma() {
    let foo = tokio::join!(async { 1 }, async { 2 },);

    assert_eq!(foo, (1, 2));
}

#[maybe_tokio_test]
async fn sync_two_lit_expr_no_comma() {
    let foo = tokio::join!(async { 1 }, async { 2 });

    assert_eq!(foo, (1, 2));
}

#[maybe_tokio_test]
async fn two_await() {
    let (tx1, rx1) = oneshot::channel::<&str>();
    let (tx2, rx2) = oneshot::channel::<u32>();

    let mut join = task::spawn(async {
        tokio::join!(async { rx1.await.unwrap() }, async { rx2.await.unwrap() })
    });

    assert_pending!(join.poll());

    tx2.send(123).unwrap();
    assert!(join.is_woken());
    assert_pending!(join.poll());

    tx1.send("hello").unwrap();
    assert!(join.is_woken());
    let res = assert_ready!(join.poll());

    assert_eq!(("hello", 123), res);
}

#[test]
fn join_size() {
    use futures::future;
    use std::mem;

    let fut = async {
        let ready = future::ready(0i32);
        tokio::join!(ready)
    };
    assert_eq!(mem::size_of_val(&fut), 20);

    let fut = async {
        let ready1 = future::ready(0i32);
        let ready2 = future::ready(0i32);
        tokio::join!(ready1, ready2)
    };
    assert_eq!(mem::size_of_val(&fut), 32);
}

async fn non_cooperative_task(permits: Arc<Semaphore>) -> usize {
    let mut exceeded_budget = 0;

    for _ in 0..5 {
        // Another task should run after after this task uses its whole budget
        for _ in 0..128 {
            let _permit = permits.clone().acquire_owned().await.unwrap();
        }

        exceeded_budget += 1;
    }

    exceeded_budget
}

async fn poor_little_task(permits: Arc<Semaphore>) -> usize {
    let mut how_many_times_i_got_to_run = 0;

    for _ in 0..5 {
        let _permit = permits.clone().acquire_owned().await.unwrap();
        how_many_times_i_got_to_run += 1;
    }

    how_many_times_i_got_to_run
}

#[tokio::test]
async fn join_does_not_allow_tasks_to_starve() {
    let permits = Arc::new(Semaphore::new(1));

    // non_cooperative_task should yield after its budget is exceeded and then poor_little_task should run.
    let (non_cooperative_result, little_task_result) = tokio::join!(
        non_cooperative_task(Arc::clone(&permits)),
        poor_little_task(permits)
    );

    assert_eq!(5, non_cooperative_result);
    assert_eq!(5, little_task_result);
}

#[tokio::test]
async fn a_different_future_is_polled_first_every_time_poll_fn_is_polled() {
    let poll_order = Arc::new(std::sync::Mutex::new(vec![]));

    let fut = |x, poll_order: Arc<std::sync::Mutex<Vec<i32>>>| async move {
        for _ in 0..4 {
            {
                let mut guard = poll_order.lock().unwrap();

                guard.push(x);
            }

            tokio::task::yield_now().await;
        }
    };

    tokio::join!(
        fut(1, Arc::clone(&poll_order)),
        fut(2, Arc::clone(&poll_order)),
        fut(3, Arc::clone(&poll_order)),
    );

    // Each time the future created by join! is polled, it should start
    // by polling a different future first.
    assert_eq!(
        vec![1, 2, 3, 2, 3, 1, 3, 1, 2, 1, 2, 3],
        *poll_order.lock().unwrap()
    );
}
