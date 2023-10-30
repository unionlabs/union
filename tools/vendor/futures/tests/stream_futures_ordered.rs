use futures::channel::oneshot;
use futures::executor::{block_on, block_on_stream};
use futures::future::{self, join, Future, FutureExt, TryFutureExt};
use futures::stream::{FuturesOrdered, StreamExt};
use futures_test::task::noop_context;
use std::any::Any;

#[test]
fn works_1() {
    let (a_tx, a_rx) = oneshot::channel::<i32>();
    let (b_tx, b_rx) = oneshot::channel::<i32>();
    let (c_tx, c_rx) = oneshot::channel::<i32>();

    let mut stream = vec![a_rx, b_rx, c_rx].into_iter().collect::<FuturesOrdered<_>>();

    b_tx.send(99).unwrap();
    assert!(stream.poll_next_unpin(&mut noop_context()).is_pending());

    a_tx.send(33).unwrap();
    c_tx.send(33).unwrap();

    let mut iter = block_on_stream(stream);
    assert_eq!(Some(Ok(33)), iter.next());
    assert_eq!(Some(Ok(99)), iter.next());
    assert_eq!(Some(Ok(33)), iter.next());
    assert_eq!(None, iter.next());
}

#[cfg_attr(miri, ignore)] // https://github.com/rust-lang/miri/issues/1038
#[test]
fn works_2() {
    let (a_tx, a_rx) = oneshot::channel::<i32>();
    let (b_tx, b_rx) = oneshot::channel::<i32>();
    let (c_tx, c_rx) = oneshot::channel::<i32>();

    let mut stream = vec![a_rx.boxed(), join(b_rx, c_rx).map(|(a, b)| Ok(a? + b?)).boxed()]
        .into_iter()
        .collect::<FuturesOrdered<_>>();

    let mut cx = noop_context();
    a_tx.send(33).unwrap();
    b_tx.send(33).unwrap();
    assert!(stream.poll_next_unpin(&mut cx).is_ready());
    assert!(stream.poll_next_unpin(&mut cx).is_pending());
    c_tx.send(33).unwrap();
    assert!(stream.poll_next_unpin(&mut cx).is_ready());
}

#[test]
fn from_iterator() {
    let stream = vec![future::ready::<i32>(1), future::ready::<i32>(2), future::ready::<i32>(3)]
        .into_iter()
        .collect::<FuturesOrdered<_>>();
    assert_eq!(stream.len(), 3);
    assert_eq!(block_on(stream.collect::<Vec<_>>()), vec![1, 2, 3]);
}

#[cfg_attr(miri, ignore)] // https://github.com/rust-lang/miri/issues/1038
#[test]
fn queue_never_unblocked() {
    let (_a_tx, a_rx) = oneshot::channel::<Box<dyn Any + Send>>();
    let (b_tx, b_rx) = oneshot::channel::<Box<dyn Any + Send>>();
    let (c_tx, c_rx) = oneshot::channel::<Box<dyn Any + Send>>();

    let mut stream = vec![
        Box::new(a_rx) as Box<dyn Future<Output = _> + Unpin>,
        Box::new(
            future::try_select(b_rx, c_rx)
                .map_err(|e| e.factor_first().0)
                .and_then(|e| future::ok(Box::new(e) as Box<dyn Any + Send>)),
        ) as _,
    ]
    .into_iter()
    .collect::<FuturesOrdered<_>>();

    let cx = &mut noop_context();
    for _ in 0..10 {
        assert!(stream.poll_next_unpin(cx).is_pending());
    }

    b_tx.send(Box::new(())).unwrap();
    assert!(stream.poll_next_unpin(cx).is_pending());
    c_tx.send(Box::new(())).unwrap();
    assert!(stream.poll_next_unpin(cx).is_pending());
    assert!(stream.poll_next_unpin(cx).is_pending());
}
