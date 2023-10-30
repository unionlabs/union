#![warn(rust_2018_idioms)]
#![cfg(feature = "full")]

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::{self, Runtime};
use tokio::sync::oneshot;
use tokio_test::{assert_err, assert_ok};

use futures::future::poll_fn;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{mpsc, Arc, Mutex};
use std::task::{Context, Poll, Waker};

macro_rules! cfg_metrics {
    ($($t:tt)*) => {
        #[cfg(tokio_unstable)]
        {
            $( $t )*
        }
    }
}

#[test]
fn single_thread() {
    // No panic when starting a runtime w/ a single thread
    let _ = runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(1)
        .build();
}

#[test]
fn many_oneshot_futures() {
    // used for notifying the main thread
    const NUM: usize = 1_000;

    for _ in 0..5 {
        let (tx, rx) = mpsc::channel();

        let rt = rt();
        let cnt = Arc::new(AtomicUsize::new(0));

        for _ in 0..NUM {
            let cnt = cnt.clone();
            let tx = tx.clone();

            rt.spawn(async move {
                let num = cnt.fetch_add(1, Relaxed) + 1;

                if num == NUM {
                    tx.send(()).unwrap();
                }
            });
        }

        rx.recv().unwrap();

        // Wait for the pool to shutdown
        drop(rt);
    }
}

#[test]
fn spawn_two() {
    let rt = rt();

    let out = rt.block_on(async {
        let (tx, rx) = oneshot::channel();

        tokio::spawn(async move {
            tokio::spawn(async move {
                tx.send("ZOMG").unwrap();
            });
        });

        assert_ok!(rx.await)
    });

    assert_eq!(out, "ZOMG");

    cfg_metrics! {
        let metrics = rt.metrics();
        drop(rt);
        assert_eq!(1, metrics.remote_schedule_count());

        let mut local = 0;
        for i in 0..metrics.num_workers() {
            local += metrics.worker_local_schedule_count(i);
        }

        assert_eq!(1, local);
    }
}

#[test]
fn many_multishot_futures() {
    const CHAIN: usize = 200;
    const CYCLES: usize = 5;
    const TRACKS: usize = 50;

    for _ in 0..50 {
        let rt = rt();
        let mut start_txs = Vec::with_capacity(TRACKS);
        let mut final_rxs = Vec::with_capacity(TRACKS);

        for _ in 0..TRACKS {
            let (start_tx, mut chain_rx) = tokio::sync::mpsc::channel(10);

            for _ in 0..CHAIN {
                let (next_tx, next_rx) = tokio::sync::mpsc::channel(10);

                // Forward all the messages
                rt.spawn(async move {
                    while let Some(v) = chain_rx.recv().await {
                        next_tx.send(v).await.unwrap();
                    }
                });

                chain_rx = next_rx;
            }

            // This final task cycles if needed
            let (final_tx, final_rx) = tokio::sync::mpsc::channel(10);
            let cycle_tx = start_tx.clone();
            let mut rem = CYCLES;

            rt.spawn(async move {
                for _ in 0..CYCLES {
                    let msg = chain_rx.recv().await.unwrap();

                    rem -= 1;

                    if rem == 0 {
                        final_tx.send(msg).await.unwrap();
                    } else {
                        cycle_tx.send(msg).await.unwrap();
                    }
                }
            });

            start_txs.push(start_tx);
            final_rxs.push(final_rx);
        }

        {
            rt.block_on(async move {
                for start_tx in start_txs {
                    start_tx.send("ping").await.unwrap();
                }

                for mut final_rx in final_rxs {
                    final_rx.recv().await.unwrap();
                }
            });
        }
    }
}

#[test]
fn spawn_shutdown() {
    let rt = rt();
    let (tx, rx) = mpsc::channel();

    rt.block_on(async {
        tokio::spawn(client_server(tx.clone()));
    });

    // Use spawner
    rt.spawn(client_server(tx));

    assert_ok!(rx.recv());
    assert_ok!(rx.recv());

    drop(rt);
    assert_err!(rx.try_recv());
}

async fn client_server(tx: mpsc::Sender<()>) {
    let server = assert_ok!(TcpListener::bind("127.0.0.1:0").await);

    // Get the assigned address
    let addr = assert_ok!(server.local_addr());

    // Spawn the server
    tokio::spawn(async move {
        // Accept a socket
        let (mut socket, _) = server.accept().await.unwrap();

        // Write some data
        socket.write_all(b"hello").await.unwrap();
    });

    let mut client = TcpStream::connect(&addr).await.unwrap();

    let mut buf = vec![];
    client.read_to_end(&mut buf).await.unwrap();

    assert_eq!(buf, b"hello");
    tx.send(()).unwrap();
}

#[test]
fn drop_threadpool_drops_futures() {
    for _ in 0..1_000 {
        let num_inc = Arc::new(AtomicUsize::new(0));
        let num_dec = Arc::new(AtomicUsize::new(0));
        let num_drop = Arc::new(AtomicUsize::new(0));

        struct Never(Arc<AtomicUsize>);

        impl Future for Never {
            type Output = ();

            fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
                Poll::Pending
            }
        }

        impl Drop for Never {
            fn drop(&mut self) {
                self.0.fetch_add(1, Relaxed);
            }
        }

        let a = num_inc.clone();
        let b = num_dec.clone();

        let rt = runtime::Builder::new_multi_thread()
            .enable_all()
            .on_thread_start(move || {
                a.fetch_add(1, Relaxed);
            })
            .on_thread_stop(move || {
                b.fetch_add(1, Relaxed);
            })
            .build()
            .unwrap();

        rt.spawn(Never(num_drop.clone()));

        // Wait for the pool to shutdown
        drop(rt);

        // Assert that only a single thread was spawned.
        let a = num_inc.load(Relaxed);
        assert!(a >= 1);

        // Assert that all threads shutdown
        let b = num_dec.load(Relaxed);
        assert_eq!(a, b);

        // Assert that the future was dropped
        let c = num_drop.load(Relaxed);
        assert_eq!(c, 1);
    }
}

#[test]
fn start_stop_callbacks_called() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    let after_start = Arc::new(AtomicUsize::new(0));
    let before_stop = Arc::new(AtomicUsize::new(0));

    let after_inner = after_start.clone();
    let before_inner = before_stop.clone();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .on_thread_start(move || {
            after_inner.clone().fetch_add(1, Ordering::Relaxed);
        })
        .on_thread_stop(move || {
            before_inner.clone().fetch_add(1, Ordering::Relaxed);
        })
        .build()
        .unwrap();

    let (tx, rx) = oneshot::channel();

    rt.spawn(async move {
        assert_ok!(tx.send(()));
    });

    assert_ok!(rt.block_on(rx));

    drop(rt);

    assert!(after_start.load(Ordering::Relaxed) > 0);
    assert!(before_stop.load(Ordering::Relaxed) > 0);
}

#[test]
fn blocking() {
    // used for notifying the main thread
    const NUM: usize = 1_000;

    for _ in 0..10 {
        let (tx, rx) = mpsc::channel();

        let rt = rt();
        let cnt = Arc::new(AtomicUsize::new(0));

        // there are four workers in the pool
        // so, if we run 4 blocking tasks, we know that handoff must have happened
        let block = Arc::new(std::sync::Barrier::new(5));
        for _ in 0..4 {
            let block = block.clone();
            rt.spawn(async move {
                tokio::task::block_in_place(move || {
                    block.wait();
                    block.wait();
                })
            });
        }
        block.wait();

        for _ in 0..NUM {
            let cnt = cnt.clone();
            let tx = tx.clone();

            rt.spawn(async move {
                let num = cnt.fetch_add(1, Relaxed) + 1;

                if num == NUM {
                    tx.send(()).unwrap();
                }
            });
        }

        rx.recv().unwrap();

        // Wait for the pool to shutdown
        block.wait();
    }
}

#[test]
fn multi_threadpool() {
    use tokio::sync::oneshot;

    let rt1 = rt();
    let rt2 = rt();

    let (tx, rx) = oneshot::channel();
    let (done_tx, done_rx) = mpsc::channel();

    rt2.spawn(async move {
        rx.await.unwrap();
        done_tx.send(()).unwrap();
    });

    rt1.spawn(async move {
        tx.send(()).unwrap();
    });

    done_rx.recv().unwrap();
}

// When `block_in_place` returns, it attempts to reclaim the yielded runtime
// worker. In this case, the remainder of the task is on the runtime worker and
// must take part in the cooperative task budgeting system.
//
// The test ensures that, when this happens, attempting to consume from a
// channel yields occasionally even if there are values ready to receive.
#[test]
fn coop_and_block_in_place() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        // Setting max threads to 1 prevents another thread from claiming the
        // runtime worker yielded as part of `block_in_place` and guarantees the
        // same thread will reclaim the worker at the end of the
        // `block_in_place` call.
        .max_blocking_threads(1)
        .build()
        .unwrap();

    rt.block_on(async move {
        let (tx, mut rx) = tokio::sync::mpsc::channel(1024);

        // Fill the channel
        for _ in 0..1024 {
            tx.send(()).await.unwrap();
        }

        drop(tx);

        tokio::spawn(async move {
            // Block in place without doing anything
            tokio::task::block_in_place(|| {});

            // Receive all the values, this should trigger a `Pending` as the
            // coop limit will be reached.
            poll_fn(|cx| {
                while let Poll::Ready(v) = {
                    tokio::pin! {
                        let fut = rx.recv();
                    }

                    Pin::new(&mut fut).poll(cx)
                } {
                    if v.is_none() {
                        panic!("did not yield");
                    }
                }

                Poll::Ready(())
            })
            .await
        })
        .await
        .unwrap();
    });
}

// Testing this does not panic
#[test]
fn max_blocking_threads() {
    let _rt = tokio::runtime::Builder::new_multi_thread()
        .max_blocking_threads(1)
        .build()
        .unwrap();
}

#[test]
#[should_panic]
fn max_blocking_threads_set_to_zero() {
    let _rt = tokio::runtime::Builder::new_multi_thread()
        .max_blocking_threads(0)
        .build()
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn hang_on_shutdown() {
    let (sync_tx, sync_rx) = std::sync::mpsc::channel::<()>();
    tokio::spawn(async move {
        tokio::task::block_in_place(|| sync_rx.recv().ok());
    });

    tokio::spawn(async {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        drop(sync_tx);
    });
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
}

/// Demonstrates tokio-rs/tokio#3869
#[test]
fn wake_during_shutdown() {
    struct Shared {
        waker: Option<Waker>,
    }

    struct MyFuture {
        shared: Arc<Mutex<Shared>>,
        put_waker: bool,
    }

    impl MyFuture {
        fn new() -> (Self, Self) {
            let shared = Arc::new(Mutex::new(Shared { waker: None }));
            let f1 = MyFuture {
                shared: shared.clone(),
                put_waker: true,
            };
            let f2 = MyFuture {
                shared,
                put_waker: false,
            };
            (f1, f2)
        }
    }

    impl Future for MyFuture {
        type Output = ();

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
            let me = Pin::into_inner(self);
            let mut lock = me.shared.lock().unwrap();
            println!("poll {}", me.put_waker);
            if me.put_waker {
                println!("putting");
                lock.waker = Some(cx.waker().clone());
            }
            Poll::Pending
        }
    }

    impl Drop for MyFuture {
        fn drop(&mut self) {
            println!("drop {} start", self.put_waker);
            let mut lock = self.shared.lock().unwrap();
            if !self.put_waker {
                lock.waker.take().unwrap().wake();
            }
            drop(lock);
            println!("drop {} stop", self.put_waker);
        }
    }

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();

    let (f1, f2) = MyFuture::new();

    rt.spawn(f1);
    rt.spawn(f2);

    rt.block_on(async { tokio::time::sleep(tokio::time::Duration::from_millis(20)).await });
}

#[should_panic]
#[tokio::test]
async fn test_block_in_place1() {
    tokio::task::block_in_place(|| {});
}

#[tokio::test(flavor = "multi_thread")]
async fn test_block_in_place2() {
    tokio::task::block_in_place(|| {});
}

#[should_panic]
#[tokio::main(flavor = "current_thread")]
#[test]
async fn test_block_in_place3() {
    tokio::task::block_in_place(|| {});
}

#[tokio::main]
#[test]
async fn test_block_in_place4() {
    tokio::task::block_in_place(|| {});
}

fn rt() -> Runtime {
    Runtime::new().unwrap()
}
