use std::sync::Arc;
use std::thread;

use async_lock::Barrier;
use futures_lite::future;

#[test]
fn smoke() {
    future::block_on(async move {
        const N: usize = 10;

        let barrier = Arc::new(Barrier::new(N));

        for _ in 0..10 {
            let (tx, rx) = async_channel::unbounded();

            for _ in 0..N - 1 {
                let c = barrier.clone();
                let tx = tx.clone();

                thread::spawn(move || {
                    future::block_on(async move {
                        let res = c.wait().await;
                        tx.send(res.is_leader()).await.unwrap();
                    })
                });
            }

            // At this point, all spawned threads should be blocked,
            // so we shouldn't get anything from the cahnnel.
            let res = rx.try_recv();
            assert!(res.is_err());

            let mut leader_found = barrier.wait().await.is_leader();

            // Now, the barrier is cleared and we should get data.
            for _ in 0..N - 1 {
                if rx.recv().await.unwrap() {
                    assert!(!leader_found);
                    leader_found = true;
                }
            }
            assert!(leader_found);
        }
    });
}
