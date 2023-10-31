use std::sync::atomic::{AtomicUsize, Ordering};

use crossbeam_queue::SegQueue;
use crossbeam_utils::thread::scope;
use rand::{thread_rng, Rng};

#[test]
fn smoke() {
    let q = SegQueue::new();
    q.push(7);
    assert_eq!(q.pop(), Some(7));

    q.push(8);
    assert_eq!(q.pop(), Some(8));
    assert!(q.pop().is_none());
}

#[test]
fn len_empty_full() {
    let q = SegQueue::new();

    assert_eq!(q.len(), 0);
    assert!(q.is_empty());

    q.push(());

    assert_eq!(q.len(), 1);
    assert!(!q.is_empty());

    q.pop().unwrap();

    assert_eq!(q.len(), 0);
    assert!(q.is_empty());
}

#[test]
fn len() {
    let q = SegQueue::new();

    assert_eq!(q.len(), 0);

    for i in 0..50 {
        q.push(i);
        assert_eq!(q.len(), i + 1);
    }

    for i in 0..50 {
        q.pop().unwrap();
        assert_eq!(q.len(), 50 - i - 1);
    }

    assert_eq!(q.len(), 0);
}

#[cfg_attr(miri, ignore)] // Miri is too slow
#[test]
fn spsc() {
    const COUNT: usize = 100_000;

    let q = SegQueue::new();

    scope(|scope| {
        scope.spawn(|_| {
            for i in 0..COUNT {
                loop {
                    if let Some(x) = q.pop() {
                        assert_eq!(x, i);
                        break;
                    }
                }
            }
            assert!(q.pop().is_none());
        });
        scope.spawn(|_| {
            for i in 0..COUNT {
                q.push(i);
            }
        });
    })
    .unwrap();
}

#[cfg_attr(miri, ignore)] // Miri is too slow
#[test]
fn mpmc() {
    const COUNT: usize = 25_000;
    const THREADS: usize = 4;

    let q = SegQueue::<usize>::new();
    let v = (0..COUNT).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>();

    scope(|scope| {
        for _ in 0..THREADS {
            scope.spawn(|_| {
                for _ in 0..COUNT {
                    let n = loop {
                        if let Some(x) = q.pop() {
                            break x;
                        }
                    };
                    v[n].fetch_add(1, Ordering::SeqCst);
                }
            });
        }
        for _ in 0..THREADS {
            scope.spawn(|_| {
                for i in 0..COUNT {
                    q.push(i);
                }
            });
        }
    })
    .unwrap();

    for c in v {
        assert_eq!(c.load(Ordering::SeqCst), THREADS);
    }
}

#[cfg_attr(miri, ignore)] // Miri is too slow
#[test]
fn drops() {
    const RUNS: usize = 100;

    static DROPS: AtomicUsize = AtomicUsize::new(0);

    #[derive(Debug, PartialEq)]
    struct DropCounter;

    impl Drop for DropCounter {
        fn drop(&mut self) {
            DROPS.fetch_add(1, Ordering::SeqCst);
        }
    }

    let mut rng = thread_rng();

    for _ in 0..RUNS {
        let steps = rng.gen_range(0..10_000);
        let additional = rng.gen_range(0..1000);

        DROPS.store(0, Ordering::SeqCst);
        let q = SegQueue::new();

        scope(|scope| {
            scope.spawn(|_| {
                for _ in 0..steps {
                    while q.pop().is_none() {}
                }
            });

            scope.spawn(|_| {
                for _ in 0..steps {
                    q.push(DropCounter);
                }
            });
        })
        .unwrap();

        for _ in 0..additional {
            q.push(DropCounter);
        }

        assert_eq!(DROPS.load(Ordering::SeqCst), steps);
        drop(q);
        assert_eq!(DROPS.load(Ordering::SeqCst), steps + additional);
    }
}

#[test]
fn into_iter() {
    let q = SegQueue::new();
    for i in 0..100 {
        q.push(i);
    }
    for (i, j) in q.into_iter().enumerate() {
        assert_eq!(i, j);
    }
}

#[test]
fn into_iter_drop() {
    let q = SegQueue::new();
    for i in 0..100 {
        q.push(i);
    }
    for (i, j) in q.into_iter().enumerate().take(50) {
        assert_eq!(i, j);
    }
}
