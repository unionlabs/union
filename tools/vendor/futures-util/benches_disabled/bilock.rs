#![feature(test)]

#[cfg(feature = "bilock")]
mod bench {
    use futures::executor::LocalPool;
    use futures::task::{Context, Waker};
    use futures_util::lock::BiLock;
    use futures_util::lock::BiLockAcquire;
    use futures_util::lock::BiLockAcquired;
    use futures_util::task::ArcWake;

    use std::sync::Arc;
    use test::Bencher;

    fn notify_noop() -> Waker {
        struct Noop;

        impl ArcWake for Noop {
            fn wake(_: &Arc<Self>) {}
        }

        ArcWake::into_waker(Arc::new(Noop))
    }

    /// Pseudo-stream which simply calls `lock.poll()` on `poll`
    struct LockStream {
        lock: BiLockAcquire<u32>,
    }

    impl LockStream {
        fn new(lock: BiLock<u32>) -> Self {
            Self { lock: lock.lock() }
        }

        /// Release a lock after it was acquired in `poll`,
        /// so `poll` could be called again.
        fn release_lock(&mut self, guard: BiLockAcquired<u32>) {
            self.lock = guard.unlock().lock()
        }
    }

    impl Stream for LockStream {
        type Item = BiLockAcquired<u32>;
        type Error = ();

        fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Option<Self::Item>, Self::Error> {
            self.lock.poll(cx).map(|a| a.map(Some))
        }
    }

    #[bench]
    fn contended(b: &mut Bencher) {
        let pool = LocalPool::new();
        let mut exec = pool.executor();
        let waker = notify_noop();
        let mut map = task::LocalMap::new();
        let mut waker = task::Context::new(&mut map, &waker, &mut exec);

        b.iter(|| {
            let (x, y) = BiLock::new(1);

            let mut x = LockStream::new(x);
            let mut y = LockStream::new(y);

            for _ in 0..1000 {
                let x_guard = match x.poll_next(&mut waker) {
                    Ok(Poll::Ready(Some(guard))) => guard,
                    _ => panic!(),
                };

                // Try poll second lock while first lock still holds the lock
                match y.poll_next(&mut waker) {
                    Ok(Poll::Pending) => (),
                    _ => panic!(),
                };

                x.release_lock(x_guard);

                let y_guard = match y.poll_next(&mut waker) {
                    Ok(Poll::Ready(Some(guard))) => guard,
                    _ => panic!(),
                };

                y.release_lock(y_guard);
            }
            (x, y)
        });
    }

    #[bench]
    fn lock_unlock(b: &mut Bencher) {
        let pool = LocalPool::new();
        let mut exec = pool.executor();
        let waker = notify_noop();
        let mut map = task::LocalMap::new();
        let mut waker = task::Context::new(&mut map, &waker, &mut exec);

        b.iter(|| {
            let (x, y) = BiLock::new(1);

            let mut x = LockStream::new(x);
            let mut y = LockStream::new(y);

            for _ in 0..1000 {
                let x_guard = match x.poll_next(&mut waker) {
                    Ok(Poll::Ready(Some(guard))) => guard,
                    _ => panic!(),
                };

                x.release_lock(x_guard);

                let y_guard = match y.poll_next(&mut waker) {
                    Ok(Poll::Ready(Some(guard))) => guard,
                    _ => panic!(),
                };

                y.release_lock(y_guard);
            }
            (x, y)
        })
    }
}
