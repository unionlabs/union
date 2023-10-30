#![allow(clippy::cognitive_complexity)]
#![warn(rust_2018_idioms)]
#![cfg(feature = "sync")]

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test as test;

use tokio::sync::broadcast;
use tokio_test::task;
use tokio_test::{
    assert_err, assert_ok, assert_pending, assert_ready, assert_ready_err, assert_ready_ok,
};

use std::sync::Arc;

macro_rules! assert_recv {
    ($e:expr) => {
        match $e.try_recv() {
            Ok(value) => value,
            Err(e) => panic!("expected recv; got = {:?}", e),
        }
    };
}

macro_rules! assert_empty {
    ($e:expr) => {
        match $e.try_recv() {
            Ok(value) => panic!("expected empty; got = {:?}", value),
            Err(broadcast::error::TryRecvError::Empty) => {}
            Err(e) => panic!("expected empty; got = {:?}", e),
        }
    };
}

macro_rules! assert_lagged {
    ($e:expr, $n:expr) => {
        match assert_err!($e) {
            broadcast::error::TryRecvError::Lagged(n) => {
                assert_eq!(n, $n);
            }
            _ => panic!("did not lag"),
        }
    };
}

macro_rules! assert_closed {
    ($e:expr) => {
        match assert_err!($e) {
            broadcast::error::TryRecvError::Closed => {}
            _ => panic!("did not lag"),
        }
    };
}

trait AssertSend: Send + Sync {}
impl AssertSend for broadcast::Sender<i32> {}
impl AssertSend for broadcast::Receiver<i32> {}

#[test]
fn send_try_recv_bounded() {
    let (tx, mut rx) = broadcast::channel(16);

    assert_empty!(rx);

    let n = assert_ok!(tx.send("hello"));
    assert_eq!(n, 1);

    let val = assert_recv!(rx);
    assert_eq!(val, "hello");

    assert_empty!(rx);
}

#[test]
fn send_two_recv() {
    let (tx, mut rx1) = broadcast::channel(16);
    let mut rx2 = tx.subscribe();

    assert_empty!(rx1);
    assert_empty!(rx2);

    let n = assert_ok!(tx.send("hello"));
    assert_eq!(n, 2);

    let val = assert_recv!(rx1);
    assert_eq!(val, "hello");

    let val = assert_recv!(rx2);
    assert_eq!(val, "hello");

    assert_empty!(rx1);
    assert_empty!(rx2);
}

#[test]
fn send_recv_bounded() {
    let (tx, mut rx) = broadcast::channel(16);

    let mut recv = task::spawn(rx.recv());

    assert_pending!(recv.poll());

    assert_ok!(tx.send("hello"));

    assert!(recv.is_woken());
    let val = assert_ready_ok!(recv.poll());
    assert_eq!(val, "hello");
}

#[test]
fn send_two_recv_bounded() {
    let (tx, mut rx1) = broadcast::channel(16);
    let mut rx2 = tx.subscribe();

    let mut recv1 = task::spawn(rx1.recv());
    let mut recv2 = task::spawn(rx2.recv());

    assert_pending!(recv1.poll());
    assert_pending!(recv2.poll());

    assert_ok!(tx.send("hello"));

    assert!(recv1.is_woken());
    assert!(recv2.is_woken());

    let val1 = assert_ready_ok!(recv1.poll());
    let val2 = assert_ready_ok!(recv2.poll());
    assert_eq!(val1, "hello");
    assert_eq!(val2, "hello");

    drop((recv1, recv2));

    let mut recv1 = task::spawn(rx1.recv());
    let mut recv2 = task::spawn(rx2.recv());

    assert_pending!(recv1.poll());

    assert_ok!(tx.send("world"));

    assert!(recv1.is_woken());
    assert!(!recv2.is_woken());

    let val1 = assert_ready_ok!(recv1.poll());
    let val2 = assert_ready_ok!(recv2.poll());
    assert_eq!(val1, "world");
    assert_eq!(val2, "world");
}

#[test]
fn change_tasks() {
    let (tx, mut rx) = broadcast::channel(1);

    let mut recv = Box::pin(rx.recv());

    let mut task1 = task::spawn(&mut recv);
    assert_pending!(task1.poll());

    let mut task2 = task::spawn(&mut recv);
    assert_pending!(task2.poll());

    tx.send("hello").unwrap();

    assert!(task2.is_woken());
}

#[test]
fn send_slow_rx() {
    let (tx, mut rx1) = broadcast::channel(16);
    let mut rx2 = tx.subscribe();

    {
        let mut recv2 = task::spawn(rx2.recv());

        {
            let mut recv1 = task::spawn(rx1.recv());

            assert_pending!(recv1.poll());
            assert_pending!(recv2.poll());

            assert_ok!(tx.send("one"));

            assert!(recv1.is_woken());
            assert!(recv2.is_woken());

            assert_ok!(tx.send("two"));

            let val = assert_ready_ok!(recv1.poll());
            assert_eq!(val, "one");
        }

        let val = assert_ready_ok!(task::spawn(rx1.recv()).poll());
        assert_eq!(val, "two");

        let mut recv1 = task::spawn(rx1.recv());

        assert_pending!(recv1.poll());

        assert_ok!(tx.send("three"));

        assert!(recv1.is_woken());

        let val = assert_ready_ok!(recv1.poll());
        assert_eq!(val, "three");

        let val = assert_ready_ok!(recv2.poll());
        assert_eq!(val, "one");
    }

    let val = assert_recv!(rx2);
    assert_eq!(val, "two");

    let val = assert_recv!(rx2);
    assert_eq!(val, "three");
}

#[test]
fn drop_rx_while_values_remain() {
    let (tx, mut rx1) = broadcast::channel(16);
    let mut rx2 = tx.subscribe();

    assert_ok!(tx.send("one"));
    assert_ok!(tx.send("two"));

    assert_recv!(rx1);
    assert_recv!(rx2);

    drop(rx2);
    drop(rx1);
}

#[test]
fn lagging_rx() {
    let (tx, mut rx1) = broadcast::channel(2);
    let mut rx2 = tx.subscribe();

    assert_ok!(tx.send("one"));
    assert_ok!(tx.send("two"));

    assert_eq!("one", assert_recv!(rx1));

    assert_ok!(tx.send("three"));

    // Lagged too far
    let x = dbg!(rx2.try_recv());
    assert_lagged!(x, 1);

    // Calling again gets the next value
    assert_eq!("two", assert_recv!(rx2));

    assert_eq!("two", assert_recv!(rx1));
    assert_eq!("three", assert_recv!(rx1));

    assert_ok!(tx.send("four"));
    assert_ok!(tx.send("five"));

    assert_lagged!(rx2.try_recv(), 1);

    assert_ok!(tx.send("six"));

    assert_lagged!(rx2.try_recv(), 1);
}

#[test]
fn send_no_rx() {
    let (tx, _) = broadcast::channel(16);

    assert_err!(tx.send("hello"));

    let mut rx = tx.subscribe();

    assert_ok!(tx.send("world"));

    let val = assert_recv!(rx);
    assert_eq!("world", val);
}

#[test]
#[should_panic]
#[cfg(not(target_arch = "wasm32"))] // wasm currently doesn't support unwinding
fn zero_capacity() {
    broadcast::channel::<()>(0);
}

#[test]
#[should_panic]
#[cfg(not(target_arch = "wasm32"))] // wasm currently doesn't support unwinding
fn capacity_too_big() {
    use std::usize;

    broadcast::channel::<()>(1 + (usize::MAX >> 1));
}

#[test]
#[cfg(not(target_arch = "wasm32"))] // wasm currently doesn't support unwinding
fn panic_in_clone() {
    use std::panic::{self, AssertUnwindSafe};

    #[derive(Eq, PartialEq, Debug)]
    struct MyVal(usize);

    impl Clone for MyVal {
        fn clone(&self) -> MyVal {
            assert_ne!(0, self.0);
            MyVal(self.0)
        }
    }

    let (tx, mut rx) = broadcast::channel(16);

    assert_ok!(tx.send(MyVal(0)));
    assert_ok!(tx.send(MyVal(1)));

    let res = panic::catch_unwind(AssertUnwindSafe(|| {
        let _ = rx.try_recv();
    }));

    assert_err!(res);

    let val = assert_recv!(rx);
    assert_eq!(val, MyVal(1));
}

#[test]
fn dropping_tx_notifies_rx() {
    let (tx, mut rx1) = broadcast::channel::<()>(16);
    let mut rx2 = tx.subscribe();

    let tx2 = tx.clone();

    let mut recv1 = task::spawn(rx1.recv());
    let mut recv2 = task::spawn(rx2.recv());

    assert_pending!(recv1.poll());
    assert_pending!(recv2.poll());

    drop(tx);

    assert_pending!(recv1.poll());
    assert_pending!(recv2.poll());

    drop(tx2);

    assert!(recv1.is_woken());
    assert!(recv2.is_woken());

    let err = assert_ready_err!(recv1.poll());
    assert!(is_closed(err));

    let err = assert_ready_err!(recv2.poll());
    assert!(is_closed(err));
}

#[test]
fn unconsumed_messages_are_dropped() {
    let (tx, rx) = broadcast::channel(16);

    let msg = Arc::new(());

    assert_ok!(tx.send(msg.clone()));

    assert_eq!(2, Arc::strong_count(&msg));

    drop(rx);

    assert_eq!(1, Arc::strong_count(&msg));
}

#[test]
fn single_capacity_recvs() {
    let (tx, mut rx) = broadcast::channel(1);

    assert_ok!(tx.send(1));

    assert_eq!(assert_recv!(rx), 1);
    assert_empty!(rx);
}

#[test]
fn single_capacity_recvs_after_drop_1() {
    let (tx, mut rx) = broadcast::channel(1);

    assert_ok!(tx.send(1));
    drop(tx);

    assert_eq!(assert_recv!(rx), 1);
    assert_closed!(rx.try_recv());
}

#[test]
fn single_capacity_recvs_after_drop_2() {
    let (tx, mut rx) = broadcast::channel(1);

    assert_ok!(tx.send(1));
    assert_ok!(tx.send(2));
    drop(tx);

    assert_lagged!(rx.try_recv(), 1);
    assert_eq!(assert_recv!(rx), 2);
    assert_closed!(rx.try_recv());
}

#[test]
fn dropping_sender_does_not_overwrite() {
    let (tx, mut rx) = broadcast::channel(2);

    assert_ok!(tx.send(1));
    assert_ok!(tx.send(2));
    drop(tx);

    assert_eq!(assert_recv!(rx), 1);
    assert_eq!(assert_recv!(rx), 2);
    assert_closed!(rx.try_recv());
}

#[test]
fn lagging_receiver_recovers_after_wrap_closed_1() {
    let (tx, mut rx) = broadcast::channel(2);

    assert_ok!(tx.send(1));
    assert_ok!(tx.send(2));
    assert_ok!(tx.send(3));
    drop(tx);

    assert_lagged!(rx.try_recv(), 1);
    assert_eq!(assert_recv!(rx), 2);
    assert_eq!(assert_recv!(rx), 3);
    assert_closed!(rx.try_recv());
}

#[test]
fn lagging_receiver_recovers_after_wrap_closed_2() {
    let (tx, mut rx) = broadcast::channel(2);

    assert_ok!(tx.send(1));
    assert_ok!(tx.send(2));
    assert_ok!(tx.send(3));
    assert_ok!(tx.send(4));
    drop(tx);

    assert_lagged!(rx.try_recv(), 2);
    assert_eq!(assert_recv!(rx), 3);
    assert_eq!(assert_recv!(rx), 4);
    assert_closed!(rx.try_recv());
}

#[test]
fn lagging_receiver_recovers_after_wrap_open() {
    let (tx, mut rx) = broadcast::channel(2);

    assert_ok!(tx.send(1));
    assert_ok!(tx.send(2));
    assert_ok!(tx.send(3));

    assert_lagged!(rx.try_recv(), 1);
    assert_eq!(assert_recv!(rx), 2);
    assert_eq!(assert_recv!(rx), 3);
    assert_empty!(rx);
}

#[test]
fn receiver_len_with_lagged() {
    let (tx, mut rx) = broadcast::channel(3);

    tx.send(10).unwrap();
    tx.send(20).unwrap();
    tx.send(30).unwrap();
    tx.send(40).unwrap();

    assert_eq!(rx.len(), 4);
    assert_eq!(assert_recv!(rx), 10);

    tx.send(50).unwrap();
    tx.send(60).unwrap();

    assert_eq!(rx.len(), 5);
    assert_lagged!(rx.try_recv(), 1);
}

fn is_closed(err: broadcast::error::RecvError) -> bool {
    matches!(err, broadcast::error::RecvError::Closed)
}

#[test]
fn resubscribe_points_to_tail() {
    let (tx, mut rx) = broadcast::channel(3);
    tx.send(1).unwrap();

    let mut rx_resub = rx.resubscribe();

    // verify we're one behind at the start
    assert_empty!(rx_resub);
    assert_eq!(assert_recv!(rx), 1);

    // verify we do not affect rx
    tx.send(2).unwrap();
    assert_eq!(assert_recv!(rx_resub), 2);
    tx.send(3).unwrap();
    assert_eq!(assert_recv!(rx), 2);
    assert_eq!(assert_recv!(rx), 3);
    assert_empty!(rx);

    assert_eq!(assert_recv!(rx_resub), 3);
    assert_empty!(rx_resub);
}

#[test]
fn resubscribe_lagged() {
    let (tx, mut rx) = broadcast::channel(1);
    tx.send(1).unwrap();
    tx.send(2).unwrap();

    let mut rx_resub = rx.resubscribe();
    assert_lagged!(rx.try_recv(), 1);
    assert_empty!(rx_resub);

    assert_eq!(assert_recv!(rx), 2);
    assert_empty!(rx);
    assert_empty!(rx_resub);
}
