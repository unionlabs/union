use std::sync::{mpsc, Arc};
use std::thread;

use async_lock::Semaphore;
use futures_lite::future;

#[test]
fn try_acquire() {
    let s = Semaphore::new(2);
    let g1 = s.try_acquire().unwrap();
    let _g2 = s.try_acquire().unwrap();

    assert!(s.try_acquire().is_none());
    drop(g1);
    assert!(s.try_acquire().is_some());
}

#[test]
fn stress() {
    const COUNT: usize = if cfg!(miri) { 500 } else { 10_000 };

    let s = Arc::new(Semaphore::new(5));
    let (tx, rx) = mpsc::channel::<()>();

    for _ in 0..50 {
        let s = s.clone();
        let tx = tx.clone();

        thread::spawn(move || {
            future::block_on(async {
                for _ in 0..COUNT {
                    s.acquire().await;
                }
                drop(tx);
            })
        });
    }

    drop(tx);
    let _ = rx.recv();

    let _g1 = s.try_acquire().unwrap();
    let g2 = s.try_acquire().unwrap();
    let _g3 = s.try_acquire().unwrap();
    let _g4 = s.try_acquire().unwrap();
    let _g5 = s.try_acquire().unwrap();

    assert!(s.try_acquire().is_none());
    drop(g2);
    assert!(s.try_acquire().is_some());
}

#[test]
fn as_mutex() {
    let s = Arc::new(Semaphore::new(1));
    let s2 = s.clone();
    let _t = thread::spawn(move || {
        future::block_on(async {
            let _g = s2.acquire().await;
        });
    });
    future::block_on(async {
        let _g = s.acquire().await;
    });
}

#[test]
fn multi_resource() {
    let s = Arc::new(Semaphore::new(2));
    let s2 = s.clone();
    let (tx1, rx1) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();
    let _t = thread::spawn(move || {
        future::block_on(async {
            let _g = s2.acquire().await;
            let _ = rx2.recv();
            tx1.send(()).unwrap();
        });
    });
    future::block_on(async {
        let _g = s.acquire().await;
        tx2.send(()).unwrap();
        rx1.recv().unwrap();
    });
}

#[test]
fn lifetime() {
    // Show that the future keeps the semaphore alive.
    let _fut = {
        let mutex = Arc::new(Semaphore::new(2));
        mutex.acquire_arc()
    };
}
