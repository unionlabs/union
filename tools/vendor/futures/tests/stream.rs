use std::iter;
use std::sync::Arc;

use futures::channel::mpsc;
use futures::executor::block_on;
use futures::future::{self, Future};
use futures::lock::Mutex;
use futures::sink::SinkExt;
use futures::stream::{self, StreamExt};
use futures::task::Poll;
use futures::{ready, FutureExt};
use futures_test::task::noop_context;

#[test]
fn select() {
    fn select_and_compare(a: Vec<u32>, b: Vec<u32>, expected: Vec<u32>) {
        let a = stream::iter(a);
        let b = stream::iter(b);
        let vec = block_on(stream::select(a, b).collect::<Vec<_>>());
        assert_eq!(vec, expected);
    }

    select_and_compare(vec![1, 2, 3], vec![4, 5, 6], vec![1, 4, 2, 5, 3, 6]);
    select_and_compare(vec![1, 2, 3], vec![4, 5], vec![1, 4, 2, 5, 3]);
    select_and_compare(vec![1, 2], vec![4, 5, 6], vec![1, 4, 2, 5, 6]);
}

#[test]
fn flat_map() {
    block_on(async {
        let st =
            stream::iter(vec![stream::iter(0..=4u8), stream::iter(6..=10), stream::iter(0..=2)]);

        let values: Vec<_> =
            st.flat_map(|s| s.filter(|v| futures::future::ready(v % 2 == 0))).collect().await;

        assert_eq!(values, vec![0, 2, 4, 6, 8, 10, 0, 2]);
    });
}

#[test]
fn scan() {
    block_on(async {
        let values = stream::iter(vec![1u8, 2, 3, 4, 6, 8, 2])
            .scan(1, |state, e| {
                *state += 1;
                futures::future::ready(if e < *state { Some(e) } else { None })
            })
            .collect::<Vec<_>>()
            .await;

        assert_eq!(values, vec![1u8, 2, 3, 4]);
    });
}

#[test]
fn flatten_unordered() {
    use futures::executor::block_on;
    use futures::stream::*;
    use futures::task::*;
    use std::convert::identity;
    use std::pin::Pin;
    use std::thread;
    use std::time::Duration;

    struct DataStream {
        data: Vec<u8>,
        polled: bool,
        wake_immediately: bool,
    }

    impl Stream for DataStream {
        type Item = u8;

        fn poll_next(mut self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Option<Self::Item>> {
            if !self.polled {
                if !self.wake_immediately {
                    let waker = ctx.waker().clone();
                    let sleep_time =
                        Duration::from_millis(*self.data.first().unwrap_or(&0) as u64 / 10);
                    thread::spawn(move || {
                        thread::sleep(sleep_time);
                        waker.wake_by_ref();
                    });
                } else {
                    ctx.waker().wake_by_ref();
                }
                self.polled = true;
                Poll::Pending
            } else {
                self.polled = false;
                Poll::Ready(self.data.pop())
            }
        }
    }

    struct Interchanger {
        polled: bool,
        base: u8,
        wake_immediately: bool,
    }

    impl Stream for Interchanger {
        type Item = DataStream;

        fn poll_next(mut self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Option<Self::Item>> {
            if !self.polled {
                self.polled = true;
                if !self.wake_immediately {
                    let waker = ctx.waker().clone();
                    let sleep_time = Duration::from_millis(self.base as u64);
                    thread::spawn(move || {
                        thread::sleep(sleep_time);
                        waker.wake_by_ref();
                    });
                } else {
                    ctx.waker().wake_by_ref();
                }
                Poll::Pending
            } else {
                let data: Vec<_> = (0..6).rev().map(|v| v + self.base * 6).collect();
                self.base += 1;
                self.polled = false;
                Poll::Ready(Some(DataStream {
                    polled: false,
                    data,
                    wake_immediately: self.wake_immediately && self.base % 2 == 0,
                }))
            }
        }
    }

    // basic behaviour
    {
        block_on(async {
            let st = stream::iter(vec![
                stream::iter(0..=4u8),
                stream::iter(6..=10),
                stream::iter(10..=12),
            ]);

            let fl_unordered = st.flatten_unordered(3).collect::<Vec<_>>().await;

            assert_eq!(fl_unordered, vec![0, 6, 10, 1, 7, 11, 2, 8, 12, 3, 9, 4, 10]);
        });

        block_on(async {
            let st = stream::iter(vec![
                stream::iter(0..=4u8),
                stream::iter(6..=10),
                stream::iter(0..=2),
            ]);

            let mut fm_unordered = st
                .flat_map_unordered(1, |s| s.filter(|v| futures::future::ready(v % 2 == 0)))
                .collect::<Vec<_>>()
                .await;

            fm_unordered.sort_unstable();

            assert_eq!(fm_unordered, vec![0, 0, 2, 2, 4, 6, 8, 10]);
        });
    }

    // wake up immediately
    {
        block_on(async {
            let mut fl_unordered = Interchanger { polled: false, base: 0, wake_immediately: true }
                .take(10)
                .map(|s| s.map(identity))
                .flatten_unordered(10)
                .collect::<Vec<_>>()
                .await;

            fl_unordered.sort_unstable();

            assert_eq!(fl_unordered, (0..60).collect::<Vec<u8>>());
        });

        block_on(async {
            let mut fm_unordered = Interchanger { polled: false, base: 0, wake_immediately: true }
                .take(10)
                .flat_map_unordered(10, |s| s.map(identity))
                .collect::<Vec<_>>()
                .await;

            fm_unordered.sort_unstable();

            assert_eq!(fm_unordered, (0..60).collect::<Vec<u8>>());
        });
    }

    // wake up after delay
    {
        block_on(async {
            let mut fl_unordered = Interchanger { polled: false, base: 0, wake_immediately: false }
                .take(10)
                .map(|s| s.map(identity))
                .flatten_unordered(10)
                .collect::<Vec<_>>()
                .await;

            fl_unordered.sort_unstable();

            assert_eq!(fl_unordered, (0..60).collect::<Vec<u8>>());
        });

        block_on(async {
            let mut fm_unordered = Interchanger { polled: false, base: 0, wake_immediately: false }
                .take(10)
                .flat_map_unordered(10, |s| s.map(identity))
                .collect::<Vec<_>>()
                .await;

            fm_unordered.sort_unstable();

            assert_eq!(fm_unordered, (0..60).collect::<Vec<u8>>());
        });

        block_on(async {
            let (mut fm_unordered, mut fl_unordered) = futures_util::join!(
                Interchanger { polled: false, base: 0, wake_immediately: false }
                    .take(10)
                    .flat_map_unordered(10, |s| s.map(identity))
                    .collect::<Vec<_>>(),
                Interchanger { polled: false, base: 0, wake_immediately: false }
                    .take(10)
                    .map(|s| s.map(identity))
                    .flatten_unordered(10)
                    .collect::<Vec<_>>()
            );

            fm_unordered.sort_unstable();
            fl_unordered.sort_unstable();

            assert_eq!(fm_unordered, fl_unordered);
            assert_eq!(fm_unordered, (0..60).collect::<Vec<u8>>());
        });
    }

    // waker panics
    {
        let stream = Arc::new(Mutex::new(
            Interchanger { polled: false, base: 0, wake_immediately: true }
                .take(10)
                .flat_map_unordered(10, |s| s.map(identity)),
        ));

        struct PanicWaker;

        impl ArcWake for PanicWaker {
            fn wake_by_ref(_arc_self: &Arc<Self>) {
                panic!("WAKE UP");
            }
        }

        std::thread::spawn({
            let stream = stream.clone();
            move || {
                let mut st = poll_fn(|cx| {
                    let mut lock = ready!(stream.lock().poll_unpin(cx));

                    let panic_waker = waker(Arc::new(PanicWaker));
                    let mut panic_cx = Context::from_waker(&panic_waker);
                    let _ = ready!(lock.poll_next_unpin(&mut panic_cx));

                    Poll::Ready(Some(()))
                });

                block_on(st.next())
            }
        })
        .join()
        .unwrap_err();

        block_on(async move {
            let mut values: Vec<_> = stream.lock().await.by_ref().collect().await;
            values.sort_unstable();

            assert_eq!(values, (0..60).collect::<Vec<u8>>());
        });
    }

    // stream panics
    {
        let st = stream::iter(iter::once(
            once(Box::pin(async { panic!("Polled") })).left_stream::<DataStream>(),
        ))
        .chain(
            Interchanger { polled: false, base: 0, wake_immediately: true }
                .map(|stream| stream.right_stream())
                .take(10),
        );

        let stream = Arc::new(Mutex::new(st.flatten_unordered(10)));

        std::thread::spawn({
            let stream = stream.clone();
            move || {
                let mut st = poll_fn(|cx| {
                    let mut lock = ready!(stream.lock().poll_unpin(cx));
                    let data = ready!(lock.poll_next_unpin(cx));

                    Poll::Ready(data)
                });

                block_on(st.next())
            }
        })
        .join()
        .unwrap_err();

        block_on(async move {
            let mut values: Vec<_> = stream.lock().await.by_ref().collect().await;
            values.sort_unstable();

            assert_eq!(values, (0..60).collect::<Vec<u8>>());
        });
    }
}

#[test]
fn take_until() {
    fn make_stop_fut(stop_on: u32) -> impl Future<Output = ()> {
        let mut i = 0;
        future::poll_fn(move |_cx| {
            i += 1;
            if i <= stop_on {
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        })
    }

    block_on(async {
        // Verify stopping works:
        let stream = stream::iter(1u32..=10);
        let stop_fut = make_stop_fut(5);

        let stream = stream.take_until(stop_fut);
        let last = stream.fold(0, |_, i| async move { i }).await;
        assert_eq!(last, 5);

        // Verify take_future() works:
        let stream = stream::iter(1..=10);
        let stop_fut = make_stop_fut(5);

        let mut stream = stream.take_until(stop_fut);

        assert_eq!(stream.next().await, Some(1));
        assert_eq!(stream.next().await, Some(2));

        stream.take_future();

        let last = stream.fold(0, |_, i| async move { i }).await;
        assert_eq!(last, 10);

        // Verify take_future() returns None if stream is stopped:
        let stream = stream::iter(1u32..=10);
        let stop_fut = make_stop_fut(1);
        let mut stream = stream.take_until(stop_fut);
        assert_eq!(stream.next().await, Some(1));
        assert_eq!(stream.next().await, None);
        assert!(stream.take_future().is_none());

        // Verify TakeUntil is fused:
        let mut i = 0;
        let stream = stream::poll_fn(move |_cx| {
            i += 1;
            match i {
                1 => Poll::Ready(Some(1)),
                2 => Poll::Ready(None),
                _ => panic!("TakeUntil not fused"),
            }
        });

        let stop_fut = make_stop_fut(1);
        let mut stream = stream.take_until(stop_fut);
        assert_eq!(stream.next().await, Some(1));
        assert_eq!(stream.next().await, None);
        assert_eq!(stream.next().await, None);
    });
}

#[test]
#[should_panic]
fn chunks_panic_on_cap_zero() {
    let (_, rx1) = mpsc::channel::<()>(1);

    let _ = rx1.chunks(0);
}

#[test]
#[should_panic]
fn ready_chunks_panic_on_cap_zero() {
    let (_, rx1) = mpsc::channel::<()>(1);

    let _ = rx1.ready_chunks(0);
}

#[test]
fn ready_chunks() {
    let (mut tx, rx1) = mpsc::channel::<i32>(16);

    let mut s = rx1.ready_chunks(2);

    let mut cx = noop_context();
    assert!(s.next().poll_unpin(&mut cx).is_pending());

    block_on(async {
        tx.send(1).await.unwrap();

        assert_eq!(s.next().await.unwrap(), vec![1]);
        tx.send(2).await.unwrap();
        tx.send(3).await.unwrap();
        tx.send(4).await.unwrap();
        assert_eq!(s.next().await.unwrap(), vec![2, 3]);
        assert_eq!(s.next().await.unwrap(), vec![4]);
    });
}
