use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crossbeam_utils::sync::WaitGroup;

const THREADS: usize = 10;

#[test]
fn wait() {
    let wg = WaitGroup::new();
    let (tx, rx) = mpsc::channel();

    for _ in 0..THREADS {
        let wg = wg.clone();
        let tx = tx.clone();

        thread::spawn(move || {
            wg.wait();
            tx.send(()).unwrap();
        });
    }

    thread::sleep(Duration::from_millis(100));

    // At this point, all spawned threads should be blocked, so we shouldn't get anything from the
    // channel.
    assert!(rx.try_recv().is_err());

    wg.wait();

    // Now, the wait group is cleared and we should receive messages.
    for _ in 0..THREADS {
        rx.recv().unwrap();
    }
}

#[test]
#[cfg_attr(miri, ignore)] // this test makes timing assumptions, but Miri is so slow it violates them
fn wait_and_drop() {
    let wg = WaitGroup::new();
    let (tx, rx) = mpsc::channel();

    for _ in 0..THREADS {
        let wg = wg.clone();
        let tx = tx.clone();

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            tx.send(()).unwrap();
            drop(wg);
        });
    }

    // At this point, all spawned threads should be in `thread::sleep`, so we shouldn't get anything
    // from the channel.
    assert!(rx.try_recv().is_err());

    wg.wait();

    // Now, the wait group is cleared and we should receive messages.
    for _ in 0..THREADS {
        rx.try_recv().unwrap();
    }
}
