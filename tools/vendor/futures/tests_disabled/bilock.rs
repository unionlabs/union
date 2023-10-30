use futures::future;
use futures::stream;
use futures::task;
use futures_util::lock::BiLock;
use std::thread;

// mod support;
// use support::*;

#[test]
fn smoke() {
    let future = future::lazy(|_| {
        let (a, b) = BiLock::new(1);

        {
            let mut lock = match a.poll_lock() {
                Poll::Ready(l) => l,
                Poll::Pending => panic!("poll not ready"),
            };
            assert_eq!(*lock, 1);
            *lock = 2;

            assert!(b.poll_lock().is_pending());
            assert!(a.poll_lock().is_pending());
        }

        assert!(b.poll_lock().is_ready());
        assert!(a.poll_lock().is_ready());

        {
            let lock = match b.poll_lock() {
                Poll::Ready(l) => l,
                Poll::Pending => panic!("poll not ready"),
            };
            assert_eq!(*lock, 2);
        }

        assert_eq!(a.reunite(b).expect("bilock/smoke: reunite error"), 2);

        Ok::<(), ()>(())
    });

    assert!(task::spawn(future)
        .poll_future_notify(&notify_noop(), 0)
        .expect("failure in poll")
        .is_ready());
}

#[test]
fn concurrent() {
    const N: usize = 10000;
    let (a, b) = BiLock::new(0);

    let a = Increment { a: Some(a), remaining: N };
    let b = stream::iter_ok(0..N).fold(b, |b, _n| {
        b.lock().map(|mut b| {
            *b += 1;
            b.unlock()
        })
    });

    let t1 = thread::spawn(move || a.wait());
    let b = b.wait().expect("b error");
    let a = t1.join().unwrap().expect("a error");

    match a.poll_lock() {
        Poll::Ready(l) => assert_eq!(*l, 2 * N),
        Poll::Pending => panic!("poll not ready"),
    }
    match b.poll_lock() {
        Poll::Ready(l) => assert_eq!(*l, 2 * N),
        Poll::Pending => panic!("poll not ready"),
    }

    assert_eq!(a.reunite(b).expect("bilock/concurrent: reunite error"), 2 * N);

    struct Increment {
        remaining: usize,
        a: Option<BiLock<usize>>,
    }

    impl Future for Increment {
        type Item = BiLock<usize>;
        type Error = ();

        fn poll(&mut self) -> Poll<BiLock<usize>, ()> {
            loop {
                if self.remaining == 0 {
                    return Ok(self.a.take().unwrap().into());
                }

                let a = self.a.as_ref().unwrap();
                let mut a = match a.poll_lock() {
                    Poll::Ready(l) => l,
                    Poll::Pending => return Ok(Poll::Pending),
                };
                self.remaining -= 1;
                *a += 1;
            }
        }
    }
}
