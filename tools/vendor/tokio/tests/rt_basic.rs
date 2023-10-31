#![warn(rust_2018_idioms)]
#![cfg(feature = "full")]

use tokio::runtime::Runtime;
use tokio::sync::oneshot;
use tokio::time::{timeout, Duration};
use tokio_test::{assert_err, assert_ok};

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll};
use std::thread;

mod support {
    pub(crate) mod mpsc_stream;
}

macro_rules! cfg_metrics {
    ($($t:tt)*) => {
        #[cfg(tokio_unstable)]
        {
            $( $t )*
        }
    }
}

#[test]
fn spawned_task_does_not_progress_without_block_on() {
    let (tx, mut rx) = oneshot::channel();

    let rt = rt();

    rt.spawn(async move {
        assert_ok!(tx.send("hello"));
    });

    thread::sleep(Duration::from_millis(50));

    assert_err!(rx.try_recv());

    let out = rt.block_on(async { assert_ok!(rx.await) });

    assert_eq!(out, "hello");
}

#[test]
fn no_extra_poll() {
    use pin_project_lite::pin_project;
    use std::pin::Pin;
    use std::sync::{
        atomic::{AtomicUsize, Ordering::SeqCst},
        Arc,
    };
    use std::task::{Context, Poll};
    use tokio_stream::{Stream, StreamExt};

    pin_project! {
        struct TrackPolls<S> {
            npolls: Arc<AtomicUsize>,
            #[pin]
            s: S,
        }
    }

    impl<S> Stream for TrackPolls<S>
    where
        S: Stream,
    {
        type Item = S::Item;
        fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            let this = self.project();
            this.npolls.fetch_add(1, SeqCst);
            this.s.poll_next(cx)
        }
    }

    let (tx, rx) = support::mpsc_stream::unbounded_channel_stream::<()>();
    let rx = TrackPolls {
        npolls: Arc::new(AtomicUsize::new(0)),
        s: rx,
    };
    let npolls = Arc::clone(&rx.npolls);

    let rt = rt();

    // TODO: could probably avoid this, but why not.
    let mut rx = Box::pin(rx);

    rt.spawn(async move { while rx.next().await.is_some() {} });
    rt.block_on(async {
        tokio::task::yield_now().await;
    });

    // should have been polled exactly once: the initial poll
    assert_eq!(npolls.load(SeqCst), 1);

    tx.send(()).unwrap();
    rt.block_on(async {
        tokio::task::yield_now().await;
    });

    // should have been polled twice more: once to yield Some(), then once to yield Pending
    assert_eq!(npolls.load(SeqCst), 1 + 2);

    drop(tx);
    rt.block_on(async {
        tokio::task::yield_now().await;
    });

    // should have been polled once more: to yield None
    assert_eq!(npolls.load(SeqCst), 1 + 2 + 1);
}

#[test]
fn acquire_mutex_in_drop() {
    use futures::future::pending;
    use tokio::task;

    let (tx1, rx1) = oneshot::channel();
    let (tx2, rx2) = oneshot::channel();

    let rt = rt();

    rt.spawn(async move {
        let _ = rx2.await;
        unreachable!();
    });

    rt.spawn(async move {
        let _ = rx1.await;
        tx2.send(()).unwrap();
        unreachable!();
    });

    // Spawn a task that will never notify
    rt.spawn(async move {
        pending::<()>().await;
        tx1.send(()).unwrap();
    });

    // Tick the loop
    rt.block_on(async {
        task::yield_now().await;
    });

    // Drop the rt
    drop(rt);
}

#[test]
fn drop_tasks_in_context() {
    static SUCCESS: AtomicBool = AtomicBool::new(false);

    struct ContextOnDrop;

    impl Future for ContextOnDrop {
        type Output = ();

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
            Poll::Pending
        }
    }

    impl Drop for ContextOnDrop {
        fn drop(&mut self) {
            if tokio::runtime::Handle::try_current().is_ok() {
                SUCCESS.store(true, Ordering::SeqCst);
            }
        }
    }

    let rt = rt();
    rt.spawn(ContextOnDrop);
    drop(rt);

    assert!(SUCCESS.load(Ordering::SeqCst));
}

#[test]
#[should_panic(expected = "boom")]
fn wake_in_drop_after_panic() {
    let (tx, rx) = oneshot::channel::<()>();

    struct WakeOnDrop(Option<oneshot::Sender<()>>);

    impl Drop for WakeOnDrop {
        fn drop(&mut self) {
            self.0.take().unwrap().send(()).unwrap();
        }
    }

    let rt = rt();

    rt.spawn(async move {
        let _wake_on_drop = WakeOnDrop(Some(tx));
        // wait forever
        futures::future::pending::<()>().await;
    });

    let _join = rt.spawn(async move { rx.await });

    rt.block_on(async {
        tokio::task::yield_now().await;
        panic!("boom");
    });
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
        assert_eq!(0, metrics.remote_schedule_count());

        let mut local = 0;
        for i in 0..metrics.num_workers() {
            local += metrics.worker_local_schedule_count(i);
        }

        assert_eq!(2, local);
    }
}

#[test]
fn spawn_remote() {
    let rt = rt();

    let out = rt.block_on(async {
        let (tx, rx) = oneshot::channel();

        let handle = tokio::spawn(async move {
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(10));
                tx.send("ZOMG").unwrap();
            });

            rx.await.unwrap()
        });

        handle.await.unwrap()
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
#[should_panic(
    expected = "A Tokio 1.x context was found, but timers are disabled. Call `enable_time` on the runtime builder to enable timers."
)]
fn timeout_panics_when_no_time_handle() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    rt.block_on(async {
        let (_tx, rx) = oneshot::channel::<()>();
        let dur = Duration::from_millis(20);
        let _ = timeout(dur, rx).await;
    });
}

#[cfg(tokio_unstable)]
mod unstable {
    use tokio::runtime::{Builder, UnhandledPanic};

    #[test]
    #[should_panic(
        expected = "a spawned task panicked and the runtime is configured to shut down on unhandled panic"
    )]
    fn shutdown_on_panic() {
        let rt = Builder::new_current_thread()
            .unhandled_panic(UnhandledPanic::ShutdownRuntime)
            .build()
            .unwrap();

        rt.block_on(async {
            tokio::spawn(async {
                panic!("boom");
            });

            futures::future::pending::<()>().await;
        })
    }

    #[test]
    fn spawns_do_nothing() {
        use std::sync::Arc;

        let rt = Builder::new_current_thread()
            .unhandled_panic(UnhandledPanic::ShutdownRuntime)
            .build()
            .unwrap();

        let rt1 = Arc::new(rt);
        let rt2 = rt1.clone();

        let _ = std::thread::spawn(move || {
            rt2.block_on(async {
                tokio::spawn(async {
                    panic!("boom");
                });

                futures::future::pending::<()>().await;
            })
        })
        .join();

        let task = rt1.spawn(async {});
        let res = futures::executor::block_on(task);
        assert!(res.is_err());
    }

    #[test]
    fn shutdown_all_concurrent_block_on() {
        const N: usize = 2;
        use std::sync::{mpsc, Arc};

        let rt = Builder::new_current_thread()
            .unhandled_panic(UnhandledPanic::ShutdownRuntime)
            .build()
            .unwrap();

        let rt = Arc::new(rt);
        let mut ths = vec![];
        let (tx, rx) = mpsc::channel();

        for _ in 0..N {
            let rt = rt.clone();
            let tx = tx.clone();
            ths.push(std::thread::spawn(move || {
                rt.block_on(async {
                    tx.send(()).unwrap();
                    futures::future::pending::<()>().await;
                });
            }));
        }

        for _ in 0..N {
            rx.recv().unwrap();
        }

        rt.spawn(async {
            panic!("boom");
        });

        for th in ths {
            assert!(th.join().is_err());
        }
    }
}

fn rt() -> Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}
