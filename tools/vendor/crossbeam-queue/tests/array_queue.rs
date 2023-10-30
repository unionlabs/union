use std::sync::atomic::{AtomicUsize, Ordering};

use crossbeam_queue::ArrayQueue;
use crossbeam_utils::thread::scope;
use rand::{thread_rng, Rng};

#[test]
fn smoke() {
    let q = ArrayQueue::new(1);

    q.push(7).unwrap();
    assert_eq!(q.pop(), Some(7));

    q.push(8).unwrap();
    assert_eq!(q.pop(), Some(8));
    assert!(q.pop().is_none());
}

#[test]
fn capacity() {
    for i in 1..10 {
        let q = ArrayQueue::<i32>::new(i);
        assert_eq!(q.capacity(), i);
    }
}

#[test]
#[should_panic(expected = "capacity must be non-zero")]
fn zero_capacity() {
    let _ = ArrayQueue::<i32>::new(0);
}

#[test]
fn len_empty_full() {
    let q = ArrayQueue::new(2);

    assert_eq!(q.len(), 0);
    assert!(q.is_empty());
    assert!(!q.is_full());

    q.push(()).unwrap();

    assert_eq!(q.len(), 1);
    assert!(!q.is_empty());
    assert!(!q.is_full());

    q.push(()).unwrap();

    assert_eq!(q.len(), 2);
    assert!(!q.is_empty());
    assert!(q.is_full());

    q.pop().unwrap();

    assert_eq!(q.len(), 1);
    assert!(!q.is_empty());
    assert!(!q.is_full());
}

#[cfg_attr(miri, ignore)] // Miri is too slow
#[test]
fn len() {
    const COUNT: usize = 25_000;
    const CAP: usize = 1000;

    let q = ArrayQueue::new(CAP);
    assert_eq!(q.len(), 0);

    for _ in 0..CAP / 10 {
        for i in 0..50 {
            q.push(i).unwrap();
            assert_eq!(q.len(), i + 1);
        }

        for i in 0..50 {
            q.pop().unwrap();
            assert_eq!(q.len(), 50 - i - 1);
        }
    }
    assert_eq!(q.len(), 0);

    for i in 0..CAP {
        q.push(i).unwrap();
        assert_eq!(q.len(), i + 1);
    }

    for _ in 0..CAP {
        q.pop().unwrap();
    }
    assert_eq!(q.len(), 0);

    scope(|scope| {
        scope.spawn(|_| {
            for i in 0..COUNT {
                loop {
                    if let Some(x) = q.pop() {
                        assert_eq!(x, i);
                        break;
                    }
                }
                let len = q.len();
                assert!(len <= CAP);
            }
        });

        scope.spawn(|_| {
            for i in 0..COUNT {
                while q.push(i).is_err() {}
                let len = q.len();
                assert!(len <= CAP);
            }
        });
    })
    .unwrap();
    assert_eq!(q.len(), 0);
}

#[cfg_attr(miri, ignore)] // Miri is too slow
#[test]
fn spsc() {
    const COUNT: usize = 100_000;

    let q = ArrayQueue::new(3);

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
                while q.push(i).is_err() {}
            }
        });
    })
    .unwrap();
}

#[cfg_attr(miri, ignore)] // Miri is too slow
#[test]
fn spsc_ring_buffer() {
    const COUNT: usize = 100_000;

    let t = AtomicUsize::new(1);
    let q = ArrayQueue::<usize>::new(3);
    let v = (0..COUNT).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>();

    scope(|scope| {
        scope.spawn(|_| loop {
            match t.load(Ordering::SeqCst) {
                0 if q.is_empty() => break,

                _ => {
                    while let Some(n) = q.pop() {
                        v[n].fetch_add(1, Ordering::SeqCst);
                    }
                }
            }
        });

        scope.spawn(|_| {
            for i in 0..COUNT {
                if let Some(n) = q.force_push(i) {
                    v[n].fetch_add(1, Ordering::SeqCst);
                }
            }

            t.fetch_sub(1, Ordering::SeqCst);
        });
    })
    .unwrap();

    for c in v {
        assert_eq!(c.load(Ordering::SeqCst), 1);
    }
}

#[cfg_attr(miri, ignore)] // Miri is too slow
#[test]
fn mpmc() {
    const COUNT: usize = 25_000;
    const THREADS: usize = 4;

    let q = ArrayQueue::<usize>::new(3);
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
                    while q.push(i).is_err() {}
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
fn mpmc_ring_buffer() {
    const COUNT: usize = 25_000;
    const THREADS: usize = 4;

    let t = AtomicUsize::new(THREADS);
    let q = ArrayQueue::<usize>::new(3);
    let v = (0..COUNT).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>();

    scope(|scope| {
        for _ in 0..THREADS {
            scope.spawn(|_| loop {
                match t.load(Ordering::SeqCst) {
                    0 if q.is_empty() => break,

                    _ => {
                        while let Some(n) = q.pop() {
                            v[n].fetch_add(1, Ordering::SeqCst);
                        }
                    }
                }
            });
        }

        for _ in 0..THREADS {
            scope.spawn(|_| {
                for i in 0..COUNT {
                    if let Some(n) = q.force_push(i) {
                        v[n].fetch_add(1, Ordering::SeqCst);
                    }
                }

                t.fetch_sub(1, Ordering::SeqCst);
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
        let additional = rng.gen_range(0..50);

        DROPS.store(0, Ordering::SeqCst);
        let q = ArrayQueue::new(50);

        scope(|scope| {
            scope.spawn(|_| {
                for _ in 0..steps {
                    while q.pop().is_none() {}
                }
            });

            scope.spawn(|_| {
                for _ in 0..steps {
                    while q.push(DropCounter).is_err() {
                        DROPS.fetch_sub(1, Ordering::SeqCst);
                    }
                }
            });
        })
        .unwrap();

        for _ in 0..additional {
            q.push(DropCounter).unwrap();
        }

        assert_eq!(DROPS.load(Ordering::SeqCst), steps);
        drop(q);
        assert_eq!(DROPS.load(Ordering::SeqCst), steps + additional);
    }
}

#[test]
fn linearizable() {
    #[cfg(miri)]
    const COUNT: usize = 500;
    #[cfg(not(miri))]
    const COUNT: usize = 25_000;
    const THREADS: usize = 4;

    let q = ArrayQueue::new(THREADS);

    scope(|scope| {
        for _ in 0..THREADS / 2 {
            scope.spawn(|_| {
                for _ in 0..COUNT {
                    while q.push(0).is_err() {}
                    q.pop().unwrap();
                }
            });

            scope.spawn(|_| {
                for _ in 0..COUNT {
                    if q.force_push(0).is_none() {
                        q.pop().unwrap();
                    }
                }
            });
        }
    })
    .unwrap();
}

#[test]
fn into_iter() {
    let q = ArrayQueue::new(100);
    for i in 0..100 {
        q.push(i).unwrap();
    }
    for (i, j) in q.into_iter().enumerate() {
        assert_eq!(i, j);
    }
}
