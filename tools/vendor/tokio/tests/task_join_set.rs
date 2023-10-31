#![warn(rust_2018_idioms)]
#![cfg(all(feature = "full", tokio_unstable))]

use tokio::sync::oneshot;
use tokio::task::JoinSet;
use tokio::time::Duration;

use futures::future::FutureExt;

fn rt() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
}

#[tokio::test(start_paused = true)]
async fn test_with_sleep() {
    let mut set = JoinSet::new();

    for i in 0..10 {
        set.spawn(async move { i });
        assert_eq!(set.len(), 1 + i);
    }
    set.detach_all();
    assert_eq!(set.len(), 0);

    assert!(matches!(set.join_next().await, None));

    for i in 0..10 {
        set.spawn(async move {
            tokio::time::sleep(Duration::from_secs(i as u64)).await;
            i
        });
        assert_eq!(set.len(), 1 + i);
    }

    let mut seen = [false; 10];
    while let Some(res) = set.join_next().await.transpose().unwrap() {
        seen[res] = true;
    }

    for was_seen in &seen {
        assert!(was_seen);
    }
    assert!(matches!(set.join_next().await, None));

    // Do it again.
    for i in 0..10 {
        set.spawn(async move {
            tokio::time::sleep(Duration::from_secs(i as u64)).await;
            i
        });
    }

    let mut seen = [false; 10];
    while let Some(res) = set.join_next().await.transpose().unwrap() {
        seen[res] = true;
    }

    for was_seen in &seen {
        assert!(was_seen);
    }
    assert!(matches!(set.join_next().await, None));
}

#[tokio::test]
async fn test_abort_on_drop() {
    let mut set = JoinSet::new();

    let mut recvs = Vec::new();

    for _ in 0..16 {
        let (send, recv) = oneshot::channel::<()>();
        recvs.push(recv);

        set.spawn(async {
            // This task will never complete on its own.
            futures::future::pending::<()>().await;
            drop(send);
        });
    }

    drop(set);

    for recv in recvs {
        // The task is aborted soon and we will receive an error.
        assert!(recv.await.is_err());
    }
}

#[tokio::test]
async fn alternating() {
    let mut set = JoinSet::new();

    assert_eq!(set.len(), 0);
    set.spawn(async {});
    assert_eq!(set.len(), 1);
    set.spawn(async {});
    assert_eq!(set.len(), 2);

    for _ in 0..16 {
        let () = set.join_next().await.unwrap().unwrap();
        assert_eq!(set.len(), 1);
        set.spawn(async {});
        assert_eq!(set.len(), 2);
    }
}

#[tokio::test(start_paused = true)]
async fn abort_tasks() {
    let mut set = JoinSet::new();
    let mut num_canceled = 0;
    let mut num_completed = 0;
    for i in 0..16 {
        let abort = set.spawn(async move {
            tokio::time::sleep(Duration::from_secs(i as u64)).await;
            i
        });

        if i % 2 != 0 {
            // abort odd-numbered tasks.
            abort.abort();
        }
    }
    loop {
        match set.join_next().await {
            Some(Ok(res)) => {
                num_completed += 1;
                assert_eq!(res % 2, 0);
            }
            Some(Err(e)) => {
                assert!(e.is_cancelled());
                num_canceled += 1;
            }
            None => break,
        }
    }

    assert_eq!(num_canceled, 8);
    assert_eq!(num_completed, 8);
}

#[test]
fn runtime_gone() {
    let mut set = JoinSet::new();
    {
        let rt = rt();
        set.spawn_on(async { 1 }, rt.handle());
        drop(rt);
    }

    assert!(rt()
        .block_on(set.join_next())
        .unwrap()
        .unwrap_err()
        .is_cancelled());
}

// This ensures that `join_next` works correctly when the coop budget is
// exhausted.
#[tokio::test(flavor = "current_thread")]
async fn join_set_coop() {
    // Large enough to trigger coop.
    const TASK_NUM: u32 = 1000;

    static SEM: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(0);

    let mut set = JoinSet::new();

    for _ in 0..TASK_NUM {
        set.spawn(async {
            SEM.add_permits(1);
        });
    }

    // Wait for all tasks to complete.
    //
    // Since this is a `current_thread` runtime, there's no race condition
    // between the last permit being added and the task completing.
    let _ = SEM.acquire_many(TASK_NUM).await.unwrap();

    let mut count = 0;
    let mut coop_count = 0;
    loop {
        match set.join_next().now_or_never() {
            Some(Some(Ok(()))) => {}
            Some(Some(Err(err))) => panic!("failed: {}", err),
            None => {
                coop_count += 1;
                tokio::task::yield_now().await;
                continue;
            }
            Some(None) => break,
        }

        count += 1;
    }
    assert!(coop_count >= 1);
    assert_eq!(count, TASK_NUM);
}

#[tokio::test(start_paused = true)]
async fn abort_all() {
    let mut set: JoinSet<()> = JoinSet::new();

    for _ in 0..5 {
        set.spawn(futures::future::pending());
    }
    for _ in 0..5 {
        set.spawn(async {
            tokio::time::sleep(Duration::from_secs(1)).await;
        });
    }

    // The join set will now have 5 pending tasks and 5 ready tasks.
    tokio::time::sleep(Duration::from_secs(2)).await;

    set.abort_all();
    assert_eq!(set.len(), 10);

    let mut count = 0;
    while let Some(res) = set.join_next().await {
        if let Err(err) = res {
            assert!(err.is_cancelled());
        }
        count += 1;
    }
    assert_eq!(count, 10);
    assert_eq!(set.len(), 0);
}
