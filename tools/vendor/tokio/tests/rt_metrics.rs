#![warn(rust_2018_idioms)]
#![cfg(all(feature = "full", tokio_unstable))]

use tokio::runtime::Runtime;
use tokio::time::{self, Duration};

#[test]
fn num_workers() {
    let rt = basic();
    assert_eq!(1, rt.metrics().num_workers());

    let rt = threaded();
    assert_eq!(2, rt.metrics().num_workers());
}

#[test]
fn remote_schedule_count() {
    use std::thread;

    let rt = basic();
    let handle = rt.handle().clone();
    let task = thread::spawn(move || {
        handle.spawn(async {
            // DO nothing
        })
    })
    .join()
    .unwrap();

    rt.block_on(task).unwrap();

    assert_eq!(1, rt.metrics().remote_schedule_count());

    let rt = threaded();
    let handle = rt.handle().clone();
    let task = thread::spawn(move || {
        handle.spawn(async {
            // DO nothing
        })
    })
    .join()
    .unwrap();

    rt.block_on(task).unwrap();

    assert_eq!(1, rt.metrics().remote_schedule_count());
}

#[test]
fn worker_park_count() {
    let rt = basic();
    let metrics = rt.metrics();
    rt.block_on(async {
        time::sleep(Duration::from_millis(1)).await;
    });
    drop(rt);
    assert!(2 <= metrics.worker_park_count(0));

    let rt = threaded();
    let metrics = rt.metrics();
    rt.block_on(async {
        time::sleep(Duration::from_millis(1)).await;
    });
    drop(rt);
    assert!(1 <= metrics.worker_park_count(0));
    assert!(1 <= metrics.worker_park_count(1));
}

#[test]
fn worker_noop_count() {
    // There isn't really a great way to generate no-op parks as they happen as
    // false-positive events under concurrency.

    let rt = basic();
    let metrics = rt.metrics();
    rt.block_on(async {
        time::sleep(Duration::from_millis(1)).await;
    });
    drop(rt);
    assert!(2 <= metrics.worker_noop_count(0));

    let rt = threaded();
    let metrics = rt.metrics();
    rt.block_on(async {
        time::sleep(Duration::from_millis(1)).await;
    });
    drop(rt);
    assert!(1 <= metrics.worker_noop_count(0));
    assert!(1 <= metrics.worker_noop_count(1));
}

#[test]
fn worker_steal_count() {
    // This metric only applies to the multi-threaded runtime.
    //
    // We use a blocking channel to backup one worker thread.
    use std::sync::mpsc::channel;

    let rt = threaded();
    let metrics = rt.metrics();

    rt.block_on(async {
        let (tx, rx) = channel();

        // Move to the runtime.
        tokio::spawn(async move {
            // Spawn the task that sends to the channel
            tokio::spawn(async move {
                tx.send(()).unwrap();
            });

            // Spawn a task that bumps the previous task out of the "next
            // scheduled" slot.
            tokio::spawn(async {});

            // Blocking receive on the channel.
            rx.recv().unwrap();
        })
        .await
        .unwrap();
    });

    drop(rt);

    let n: u64 = (0..metrics.num_workers())
        .map(|i| metrics.worker_steal_count(i))
        .sum();

    assert_eq!(1, n);
}

#[test]
fn worker_poll_count() {
    const N: u64 = 5;

    let rt = basic();
    let metrics = rt.metrics();
    rt.block_on(async {
        for _ in 0..N {
            tokio::spawn(async {}).await.unwrap();
        }
    });
    drop(rt);
    assert_eq!(N, metrics.worker_poll_count(0));

    let rt = threaded();
    let metrics = rt.metrics();
    rt.block_on(async {
        for _ in 0..N {
            tokio::spawn(async {}).await.unwrap();
        }
    });
    drop(rt);
    // Account for the `block_on` task
    let n = (0..metrics.num_workers())
        .map(|i| metrics.worker_poll_count(i))
        .sum();

    assert_eq!(N, n);
}

#[test]
fn worker_total_busy_duration() {
    const N: usize = 5;

    let zero = Duration::from_millis(0);

    let rt = basic();
    let metrics = rt.metrics();

    rt.block_on(async {
        for _ in 0..N {
            tokio::spawn(async {
                tokio::task::yield_now().await;
            })
            .await
            .unwrap();
        }
    });

    drop(rt);

    assert!(zero < metrics.worker_total_busy_duration(0));

    let rt = threaded();
    let metrics = rt.metrics();

    rt.block_on(async {
        for _ in 0..N {
            tokio::spawn(async {
                tokio::task::yield_now().await;
            })
            .await
            .unwrap();
        }
    });

    drop(rt);

    for i in 0..metrics.num_workers() {
        assert!(zero < metrics.worker_total_busy_duration(i));
    }
}

#[test]
fn worker_local_schedule_count() {
    let rt = basic();
    let metrics = rt.metrics();
    rt.block_on(async {
        tokio::spawn(async {}).await.unwrap();
    });
    drop(rt);

    assert_eq!(1, metrics.worker_local_schedule_count(0));
    assert_eq!(0, metrics.remote_schedule_count());

    let rt = threaded();
    let metrics = rt.metrics();
    rt.block_on(async {
        // Move to the runtime
        tokio::spawn(async {
            tokio::spawn(async {}).await.unwrap();
        })
        .await
        .unwrap();
    });
    drop(rt);

    let n: u64 = (0..metrics.num_workers())
        .map(|i| metrics.worker_local_schedule_count(i))
        .sum();

    assert_eq!(2, n);
    assert_eq!(1, metrics.remote_schedule_count());
}

#[test]
fn worker_overflow_count() {
    // Only applies to the threaded worker
    let rt = threaded();
    let metrics = rt.metrics();
    rt.block_on(async {
        // Move to the runtime
        tokio::spawn(async {
            let (tx1, rx1) = std::sync::mpsc::channel();
            let (tx2, rx2) = std::sync::mpsc::channel();

            // First, we need to block the other worker until all tasks have
            // been spawned.
            tokio::spawn(async move {
                tx1.send(()).unwrap();
                rx2.recv().unwrap();
            });

            // Bump the next-run spawn
            tokio::spawn(async {});

            rx1.recv().unwrap();

            // Spawn many tasks
            for _ in 0..300 {
                tokio::spawn(async {});
            }

            tx2.send(()).unwrap();
        })
        .await
        .unwrap();
    });
    drop(rt);

    let n: u64 = (0..metrics.num_workers())
        .map(|i| metrics.worker_overflow_count(i))
        .sum();

    assert_eq!(1, n);
}

#[test]
fn injection_queue_depth() {
    use std::thread;

    let rt = basic();
    let handle = rt.handle().clone();
    let metrics = rt.metrics();

    thread::spawn(move || {
        handle.spawn(async {});
    })
    .join()
    .unwrap();

    assert_eq!(1, metrics.injection_queue_depth());

    let rt = threaded();
    let handle = rt.handle().clone();
    let metrics = rt.metrics();

    // First we need to block the runtime workers
    let (tx1, rx1) = std::sync::mpsc::channel();
    let (tx2, rx2) = std::sync::mpsc::channel();

    rt.spawn(async move { rx1.recv().unwrap() });
    rt.spawn(async move { rx2.recv().unwrap() });

    thread::spawn(move || {
        handle.spawn(async {});
    })
    .join()
    .unwrap();

    let n = metrics.injection_queue_depth();
    assert!(1 <= n, "{}", n);
    assert!(3 >= n, "{}", n);

    tx1.send(()).unwrap();
    tx2.send(()).unwrap();
}

#[test]
fn worker_local_queue_depth() {
    const N: usize = 100;

    let rt = basic();
    let metrics = rt.metrics();
    rt.block_on(async {
        for _ in 0..N {
            tokio::spawn(async {});
        }

        assert_eq!(N, metrics.worker_local_queue_depth(0));
    });

    let rt = threaded();
    let metrics = rt.metrics();
    rt.block_on(async move {
        // Move to the runtime
        tokio::spawn(async move {
            let (tx1, rx1) = std::sync::mpsc::channel();
            let (tx2, rx2) = std::sync::mpsc::channel();

            // First, we need to block the other worker until all tasks have
            // been spawned.
            tokio::spawn(async move {
                tx1.send(()).unwrap();
                rx2.recv().unwrap();
            });

            // Bump the next-run spawn
            tokio::spawn(async {});

            rx1.recv().unwrap();

            // Spawn some tasks
            for _ in 0..100 {
                tokio::spawn(async {});
            }

            let n: usize = (0..metrics.num_workers())
                .map(|i| metrics.worker_local_queue_depth(i))
                .sum();

            assert_eq!(n, N);

            tx2.send(()).unwrap();
        })
        .await
        .unwrap();
    });
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn io_driver_fd_count() {
    let rt = basic();
    let metrics = rt.metrics();

    // Since this is enabled w/ the process driver we always
    // have 1 fd registered.
    assert_eq!(metrics.io_driver_fd_registered_count(), 1);

    let stream = tokio::net::TcpStream::connect("google.com:80");
    let stream = rt.block_on(async move { stream.await.unwrap() });

    assert_eq!(metrics.io_driver_fd_registered_count(), 2);
    assert_eq!(metrics.io_driver_fd_deregistered_count(), 0);

    drop(stream);

    assert_eq!(metrics.io_driver_fd_deregistered_count(), 1);
    assert_eq!(metrics.io_driver_fd_registered_count(), 2);
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn io_driver_ready_count() {
    let rt = basic();
    let metrics = rt.metrics();

    let stream = tokio::net::TcpStream::connect("google.com:80");
    let _stream = rt.block_on(async move { stream.await.unwrap() });

    assert_eq!(metrics.io_driver_ready_count(), 2);
}

fn basic() -> Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

fn threaded() -> Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap()
}
