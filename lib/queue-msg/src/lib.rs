#![feature(trait_alias, extract_if)]
// #![warn(clippy::large_futures, clippy::large_stack_frames)]

use std::{
    collections::{BTreeMap, VecDeque},
    error::Error,
    fmt::Debug,
    future::Future,
    num::NonZeroU8,
    pin::Pin,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, Mutex,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use either::Either;
use frame_support_procedural::{CloneNoBound, DebugNoBound};
use futures::{
    future,
    stream::{self, FuturesUnordered},
    FutureExt, Stream, StreamExt, TryStreamExt,
};
pub use queue_msg_macro::SubsetOf;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::{pin, time::sleep};
use tracing::{debug, error, info, info_span, trace, warn, Instrument};
use unionlabs::{never::Never, ErrorReporter};

use crate::{
    aggregation::{DoCallback, HListTryFromIterator},
    optimize::{OptimizationResult, Pass, PurePass},
};

pub mod aggregation;
pub mod normalize;
pub mod optimize;

pub trait Queue<T: QueueMessage>: Debug + Clone + Send + Sync + Sized + 'static {
    /// Error type returned by this queue, representing errors that are out of control of the
    /// consumer (i.e. unable to connect to database, can't insert into row, can't deserialize row,
    /// etc)
    type Error: Error + Send + Sync + 'static;
    type Config: Debug + Clone + Serialize + DeserializeOwned;

    fn new(cfg: Self::Config) -> impl Future<Output = Result<Self, Self::Error>>;

    /// Enqueue an item into the queue, running a pure optimization pass on the item before enqueueing it.
    ///
    /// All items will be enqueued to be optimized, unless marked as ready by `O`.
    fn enqueue<'a, O: PurePass<T>>(
        &'a self,
        item: Op<T>,
        pre_enqueue_passes: &'a O,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'a;

    /// Process the item at the front of the queue, if there is one. New items will be pre-processed by `O` before being reenqueued.
    ///
    /// All items will be enqueued to be optimized, unless marked as ready by `O`.
    fn process<'a, F, Fut, R, O>(
        &'a self,
        pre_reenqueue_passes: &'a O,
        f: F,
    ) -> impl Future<Output = Result<Option<R>, Self::Error>> + Send + Captures<'a>
    where
        F: (FnOnce(Op<T>) -> Fut) + Send + Captures<'a>,
        Fut: Future<Output = (R, Result<Vec<Op<T>>, String>)> + Send + Captures<'a>,
        R: Send + Sync + 'static,
        O: PurePass<T>;

    fn optimize<'a, O: Pass<T>>(
        &'a self,
        tag: &'a str,
        optimizer: &'a O,
    ) -> impl Future<Output = Result<(), Either<Self::Error, O::Error>>> + Send + 'a;
}

#[derive(
    ::macros::Debug,
    ::frame_support_procedural::CloneNoBound,
    ::frame_support_procedural::PartialEqNoBound,
    ::serde::Serialize,
    ::serde::Deserialize,
)]
#[serde(
    tag = "@type",
    content = "@value",
    rename_all = "snake_case",
    bound(serialize = "", deserialize = ""),
    deny_unknown_fields
)]
#[debug(bound())]
pub enum Op<T: QueueMessage> {
    /// Inert data that will either be used in an [`Op::Promise`] or bubbled up to the top and sent as
    /// an output.
    Data(T::Data),
    /// Execute an action.
    Call(T::Call),
    Defer {
        until: u64,
    },
    /// Executes the contained message only if `timeout_timestamp` has not been hit.
    Timeout {
        timeout_timestamp: u64,
        msg: Box<Self>,
    },
    /// A sequence of messages to be executed in order. Messages are handled from the front, with
    /// new messages requeued at the front:
    //
    /// ```txt
    /// [A B C]
    /// D = handle(A)
    /// [D B C]
    /// ```
    Seq(VecDeque<Self>),
    /// A list of messages to be executed concurrently. If this is queued as a top-level message,
    /// each contained message will be requeued individually as a top-level message, however if it
    /// is nested within another message, it's semantics are as follows:
    ///
    /// ```txt
    /// [A B C]
    /// D = handle(A)
    /// [B C D]
    /// ```
    ///
    /// Note that this is similar to `Sequence`, except that the new messages are queued at the
    /// *back* of the list, allowing for uniform progress across all nested messages.
    Conc(VecDeque<Self>),
    /// Handle `msg`, retrying on failure. If `msg` fails, this will requeue itself with `remaining - 1`.
    Retry {
        remaining: NonZeroU8,
        msg: Box<Self>,
    },
    Promise {
        /// Messages that are expected to resolve to [`Op::Data`].
        queue: VecDeque<Self>,
        /// The resolved data messages.
        data: VecDeque<T::Data>,
        /// The message that will utilize the aggregated data.
        receiver: T::Callback,
    },
    /// Handle the contained message, voiding any returned `Data` messages that it returns.
    Void(Box<Self>),
    Noop,
}

/// Convenience constructor for [`Op::Retry`]
#[inline]
#[must_use = "constructing an instruction has no effect"]
pub fn retry<T: QueueMessage>(count: NonZeroU8, t: impl Into<Op<T>>) -> Op<T> {
    Op::Retry {
        remaining: count,
        msg: Box::new(t.into()),
    }
}

/// Convenience constructor for [`Op::Seq`]
#[inline]
#[must_use = "constructing an instruction has no effect"]
pub fn seq<T: QueueMessage>(ts: impl IntoIterator<Item = Op<T>>) -> Op<T> {
    Op::Seq(ts.into_iter().collect())
}

/// Convenience constructor for [`Op::Conc`]
#[inline]
#[must_use = "constructing an instruction has no effect"]
pub fn conc<T: QueueMessage>(ts: impl IntoIterator<Item = Op<T>>) -> Op<T> {
    Op::Conc(ts.into_iter().collect())
}

/// Convenience constructor for [`Op::Defer`]
#[inline]
#[must_use = "constructing an instruction has no effect"]
pub fn defer<T: QueueMessage>(timestamp: u64) -> Op<T> {
    Op::Defer { until: timestamp }
}

/// Convenience constructor for [`Op::Call`]
#[inline]
#[must_use = "constructing an instruction has no effect"]
pub fn call<T: QueueMessage>(t: impl Into<T::Call>) -> Op<T> {
    Op::Call(t.into())
}

/// Convenience constructor for [`Op::Data`]
#[inline]
#[must_use = "constructing an instruction has no effect"]
pub fn data<T: QueueMessage>(t: impl Into<T::Data>) -> Op<T> {
    Op::Data(t.into())
}

#[inline]
#[must_use = "constructing an instruction has no effect"]
pub fn promise<T: QueueMessage>(
    queue: impl IntoIterator<Item = Op<T>>,
    // TODO: Remove this and put data in `queue` instead
    data: impl IntoIterator<Item = T::Data>,
    callback: impl Into<T::Callback>,
) -> Op<T> {
    Op::Promise {
        queue: queue.into_iter().collect(),
        data: data.into_iter().collect(),
        receiver: callback.into(),
    }
}

#[inline]
#[must_use = "constructing an instruction has no effect"]
pub fn void<T: QueueMessage>(t: impl Into<Op<T>>) -> Op<T> {
    Op::Void(Box::new(t.into()))
}

#[inline]
#[must_use = "constructing an instruction has no effect"]
pub fn noop<T: QueueMessage>() -> Op<T> {
    Op::Noop
}

pub trait OpT =
    Debug + Clone + PartialEq + Serialize + for<'a> Deserialize<'a> + Send + Sync + Unpin;

pub trait QueueMessage: Sized + 'static {
    type Data: OpT;
    type Call: HandleCall<Self> + OpT;
    type Callback: HandleCallback<Self> + OpT;

    type Context: Context;
}

pub trait Context: Send + Sync {
    fn tags(&self) -> Vec<&str>;
}

impl Context for () {
    fn tags(&self) -> Vec<&str> {
        vec![]
    }
}

impl<T: QueueMessage> Op<T> {
    // NOTE: Box is required bc recursion
    #[allow(clippy::type_complexity)]
    pub fn handle<'a>(
        self,
        store: &'a T::Context,
        depth: usize,
    ) -> Pin<Box<dyn Future<Output = Result<Option<Op<T>>, QueueError>> + Send + 'a>> {
        trace!(%depth, "handling message");

        let fut = async move {
            match self {
                Op::Data(data) => {
                    // TODO: Use valuable here
                    info!(
                        data = %serde_json::to_string(&data).unwrap(),
                        "received data outside of an aggregation"
                    );
                    Ok(None)
                }

                Op::Call(fetch) => fetch.handle(store).await.map(Some),
                Op::Defer { until: seconds } => {
                    // if we haven't hit the time yet, requeue the defer msg
                    let current_ts_seconds = now();
                    if current_ts_seconds < seconds {
                        trace!(
                            %current_ts_seconds,
                            %seconds,
                            delta = %seconds - current_ts_seconds,
                            "defer timestamp not hit yet"
                        );

                        // TODO: Make the time configurable?
                        sleep(Duration::from_millis(10)).await;

                        Ok(Some(defer(seconds)))
                    } else {
                        Ok(None)
                    }
                }
                Op::Timeout {
                    timeout_timestamp,
                    msg,
                } => {
                    // if we haven't hit the timeout yet, handle the msg
                    if now() > timeout_timestamp {
                        warn!("message expired");

                        Ok(None)
                    } else {
                        // REVIEW: Should we handle the message here or return it to be handled on
                        // the next time it's popped from the queue?
                        msg.handle(store, depth + 1).await
                    }
                }
                Op::Seq(mut queue) => match queue.pop_front() {
                    Some(msg) => {
                        let msg = msg.handle(store, depth + 1).await?;

                        if let Some(msg) = msg {
                            queue.push_front(msg)
                        }

                        Ok(Some(seq(queue)))
                    }
                    None => Ok(None),
                },
                Op::Conc(mut queue) => match queue.pop_front() {
                    Some(msg) => {
                        let msg = msg.handle(store, depth + 1).await?;

                        if let Some(msg) = msg {
                            queue.push_back(msg)
                        }

                        Ok(Some(conc(queue)))
                    }
                    None => Ok(None),
                },
                Op::Retry { remaining, msg } => {
                    // TODO: Add some sort of exponential backoff functionality to this message type
                    const RETRY_DELAY_SECONDS: u64 = 1;

                    match msg.clone().handle(store, depth + 1).await {
                        Ok(ok) => Ok(ok),
                        Err(err) => match remaining.get().checked_sub(1).and_then(NonZeroU8::new) {
                            Some(retries_left) => {
                                warn!(
                                    retries_left,
                                    ?err,
                                    "msg failed, retrying in {RETRY_DELAY_SECONDS} seconds"
                                );

                                Ok(Some(seq([
                                    defer(now() + RETRY_DELAY_SECONDS),
                                    retry(retries_left, *msg),
                                ])))
                            }
                            None => {
                                error!("msg failed after all retries");
                                Err(err)
                            }
                        },
                    }
                }
                Op::Promise {
                    mut queue,
                    mut data,
                    receiver,
                } => {
                    if let Some(msg) = queue.pop_front() {
                        match msg {
                            Op::Data(d) => {
                                data.push_back(d);
                            }
                            msg => {
                                let msg = msg.handle(store, depth + 1).await?;

                                if let Some(msg) = msg {
                                    match msg {
                                        Op::Data(d) => {
                                            data.push_back(d);
                                        }
                                        m => {
                                            queue.push_back(m);
                                        }
                                    }
                                }
                            }
                        }

                        Ok(Some(promise(queue, data, receiver)))
                    } else {
                        // queue is empty, handle msg
                        receiver.handle(store, data).await.map(Some)
                    }
                }
                Op::Void(msg) => {
                    // TODO: distribute across seq/conc
                    Ok(msg.handle(store, depth + 1).await?.map(|msg| match msg {
                        Op::Data(data) => {
                            debug!(data = %serde_json::to_string(&data).unwrap(), "voiding data");
                            noop()
                        }
                        msg => void(msg),
                    }))
                }
                Op::Noop => Ok(None),
            }
        };

        Box::pin(fut)
    }
}

#[cfg(test)]
mod tests {
    use macros::model;

    use super::*;
    use crate::normalize::normalize;

    enum UnitMessage {}

    impl QueueMessage for UnitMessage {
        type Data = ();
        type Call = ();
        type Callback = ();

        type Context = ();
    }

    impl HandleCall<UnitMessage> for () {
        async fn handle(self, _: &()) -> Result<Op<UnitMessage>, QueueError> {
            Ok(noop())
        }
    }

    impl HandleCallback<UnitMessage> for () {
        async fn handle(self, _: &(), _: VecDeque<()>) -> Result<Op<UnitMessage>, QueueError> {
            Ok(noop())
        }
    }

    #[model]
    pub struct SimpleData {}
    #[model]
    pub struct SimpleCall {}
    #[model]
    pub struct SimpleCallback {}

    // macro_rules! vec_deque {
    //     ($($tt:tt)*) => {
    //         ::std::collections::VecDeque::from(vec![$($tt)*])
    //     };
    // }

    // async fn assert_steps<'a, T: QueueMessage, Q: Queue<T>, O: PurePass<T>>(
    //     engine: &Engine<'a, T, Q, O>,
    //     q: &InMemoryQueue<T>,
    //     steps: impl IntoIterator<Item = VecDeque<Op<T>>>,
    // ) {
    //     for (i, step) in steps.into_iter().enumerate() {
    //         engine.step().await.unwrap();
    //         assert_eq!(
    //             q.ready
    //                 .lock()
    //                 .unwrap()
    //                 .values()
    //                 .map(|item| item.msg.clone())
    //                 .collect::<VecDeque<_>>(),
    //             step,
    //             "step {i} incorrect"
    //         );
    //     }
    // }

    #[test]
    fn flatten() {
        let msg = seq::<UnitMessage>([
            defer(1),
            seq([defer(2), seq([defer(3)])]),
            seq([defer(4)]),
            defer(5),
        ]);
        assert_eq!(
            normalize(vec![msg]),
            vec![(
                vec![0],
                seq([defer(1), defer(2), defer(3), defer(4), defer(5)])
            )]
        );

        let msg = seq::<UnitMessage>([defer(1)]);
        assert_eq!(normalize(vec![msg]), vec![(vec![0], defer(1))]);

        let msg = conc::<UnitMessage>([defer(1)]);
        assert_eq!(normalize(vec![msg]), vec![(vec![0], defer(1))]);

        let msg = conc::<UnitMessage>([seq([defer(1)])]);
        assert_eq!(normalize(vec![msg]), vec![(vec![0], defer(1))]);

        let msg = seq::<UnitMessage>([noop()]);
        assert_eq!(normalize(vec![msg]), vec![]);

        let msg = conc::<UnitMessage>([seq([noop()])]);
        assert_eq!(normalize(vec![msg]), vec![]);

        let msg = conc::<UnitMessage>([conc([conc([noop()])])]);
        assert_eq!(normalize(vec![msg]), vec![]);
    }

    #[test]
    fn nested_seq_conc_single() {
        // any nesting level of seq and conc should be handled in a single pass

        let msg = conc::<UnitMessage>([seq([conc([noop()])])]);
        assert_eq!(normalize(vec![msg]), vec![]);

        let msg = conc::<UnitMessage>([seq([conc([seq([conc([seq([conc([noop()])])])])])])]);
        assert_eq!(normalize(vec![msg]), vec![]);

        let msg =
            conc::<UnitMessage>([seq([conc([seq([conc([seq([conc([seq([conc([
                data(()),
            ])])])])])])])])]);
        assert_eq!(normalize(vec![msg]), vec![(vec![0], data(()))]);

        let msg = seq::<UnitMessage>([conc([seq([conc([data(())])])])]);
        assert_eq!(normalize(vec![msg]), vec![(vec![0], data(()))]);

        let msg =
            seq::<UnitMessage>([conc([seq([conc([seq([conc([seq([conc([
                data(()),
            ])])])])])])])]);
        assert_eq!(normalize(vec![msg]), vec![(vec![0], data(()))]);

        let msg = seq::<UnitMessage>([conc([seq([conc([seq([conc([seq([conc([seq([
            conc([data(())]),
        ])])])])])])])])]);
        assert_eq!(normalize(vec![msg]), vec![(vec![0], data(()))]);
    }

    #[test]
    fn flatten_seq_conc_fixed_point_is_noop() {
        // this message can't be optimized any further, flattening operations should be a noop

        let msg = seq::<UnitMessage>([conc([defer(1), defer(2)]), defer(3)]);
        assert_eq!(normalize(vec![msg.clone()]), vec![(vec![0], msg.clone())]);
        assert_eq!(normalize(vec![msg.clone()]), vec![(vec![0], msg)]);
    }

    #[test]
    fn conc_seq_call_call_call() {
        let msg = conc::<UnitMessage>([seq([call(()), call(())]), call(())]);
        assert_eq!(
            normalize(vec![msg.clone()]),
            vec![(vec![0], seq([call(()), call(())])), (vec![0], call(()))]
        );
    }

    #[test]
    fn extract_data_simple() {
        let msg = seq::<UnitMessage>([
            data(()),
            seq([data(()), seq([data(())])]),
            seq([data(())]),
            data(()),
        ]);
        assert_eq!(
            normalize(vec![msg]),
            vec![
                (vec![0], data(())),
                (vec![0], data(())),
                (vec![0], data(())),
                (vec![0], data(())),
                (vec![0], data(())),
            ],
        );
    }

    #[test]
    fn extract_data_seq_in_promise_queue() {
        let msg = promise::<UnitMessage>([seq([call(()), data(())])], [], ());
        assert_eq!(normalize(vec![msg.clone()]), vec![(vec![0], msg)]);
    }

    #[test]
    fn seq_defer_call_data() {
        let msg = seq([seq::<UnitMessage>([defer(1), call(())]), data(())]);
        assert_eq!(
            normalize(vec![msg.clone()]),
            vec![(vec![0], seq([defer(1), call(()), data(())]))]
        );
    }

    #[test]
    fn extract_data_complex() {
        let msg = seq::<UnitMessage>([
            data(()),
            call(()),
            seq([call(()), data(()), seq([data(())])]),
            call(()),
            seq([data(()), call(())]),
            data(()),
        ]);
        assert_eq!(
            normalize(vec![msg]),
            vec![
                (vec![0], data(())),
                (
                    vec![0],
                    seq([
                        call(()),
                        call(()),
                        data(()),
                        data(()),
                        call(()),
                        data(()),
                        call(()),
                        data(()),
                    ])
                )
            ],
        );
    }

    // #[tokio::test]
    // async fn conc_seq_nested() {
    //     let q = InMemoryQueue::new(()).now_or_never().unwrap().unwrap();

    //     let engine = Engine::new(&(), &q, &NoopPass);

    //     let msgs = seq::<UnitMessage>([
    //         conc([call(()), call(())]),
    //         conc([call(()), call(())]),
    //         conc([
    //             repeat(None, call(())),
    //             repeat(None, call(())),
    //             call(()),
    //             seq([call(()), call(()), call(())]),
    //         ]),
    //     ]);

    //     q.enqueue(msgs, &NoopPass).await.unwrap();

    //     assert_steps(
    //         &engine,
    //         &q,
    //         [
    //             vec_deque![seq::<UnitMessage>([
    //                 // conc(a, b), handles a, conc(b) == b
    //                 call(()),
    //                 conc([call(()), call(())]),
    //                 conc([
    //                     repeat(None, call(())),
    //                     repeat(None, call(())),
    //                     call(()),
    //                     seq([call(()), call(()), call(())]),
    //                 ]),
    //             ])],
    //             vec_deque![seq::<UnitMessage>([
    //                 conc([call(()), call(())]),
    //                 conc([
    //                     repeat(None, call(())),
    //                     repeat(None, call(())),
    //                     call(()),
    //                     seq([call(()), call(()), call(())]),
    //                 ]),
    //             ])],
    //             vec_deque![seq::<UnitMessage>([
    //                 // conc(a, b), handles a, conc(b) == b
    //                 call(()),
    //                 conc([
    //                     repeat(None, call(())),
    //                     repeat(None, call(())),
    //                     call(()),
    //                     seq([call(()), call(()), call(())]),
    //                 ]),
    //             ])],
    //             // seq(a, conc(m...)), handles a, seq(conc(m...)) == m...
    //             vec_deque![
    //                 repeat(None, call(())),
    //                 repeat(None, call(())),
    //                 call(()),
    //                 seq([call(()), call(()), call(())]),
    //             ],
    //             vec_deque![
    //                 repeat(None, call(())),
    //                 call(()),
    //                 seq([call(()), call(()), call(())]),
    //                 // repeat(a) queues seq(a, repeat(a))
    //                 seq([call(()), repeat(None, call(()))])
    //             ],
    //             vec_deque![
    //                 call(()),
    //                 seq([call(()), call(()), call(())]),
    //                 seq([call(()), repeat(None, call(()))]),
    //                 // repeat(a) queues seq(a, repeat(a))
    //                 seq([call(()), repeat(None, call(()))])
    //             ],
    //             vec_deque![
    //                 seq([call(()), call(()), call(())]),
    //                 seq([call(()), repeat(None, call(()))]),
    //                 seq([call(()), repeat(None, call(()))]),
    //                 noop(),
    //             ],
    //             vec_deque![
    //                 seq([call(()), repeat(None, call(()))]),
    //                 seq([call(()), repeat(None, call(()))]),
    //                 noop(),
    //                 seq([call(()), call(())]),
    //             ],
    //             vec_deque![
    //                 seq([call(()), repeat(None, call(()))]),
    //                 noop(),
    //                 seq([call(()), call(())]),
    //                 repeat(None, call(())),
    //             ],
    //             vec_deque![
    //                 noop(),
    //                 seq([call(()), call(())]),
    //                 repeat(None, call(())),
    //                 repeat(None, call(())),
    //             ],
    //             vec_deque![
    //                 seq([call(()), call(())]),
    //                 repeat(None, call(())),
    //                 repeat(None, call(())),
    //             ],
    //             vec_deque![repeat(None, call(())), repeat(None, call(())), call(()),],
    //             vec_deque![
    //                 repeat(None, call(())),
    //                 call(()),
    //                 seq([call(()), repeat(None, call(()))]),
    //             ],
    //             vec_deque![
    //                 call(()),
    //                 seq([call(()), repeat(None, call(()))]),
    //                 seq([call(()), repeat(None, call(()))]),
    //             ],
    //             vec_deque![
    //                 seq([call(()), repeat(None, call(()))]),
    //                 seq([call(()), repeat(None, call(()))]),
    //                 noop(),
    //             ],
    //         ],
    //     )
    //     .await;
    // }
}

#[derive(Debug)]
pub enum QueueError {
    Fatal(BoxDynError),
    Retry(BoxDynError),
}

impl std::fmt::Display for QueueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fatal(_) => {
                write!(f, "fatal error while handling message")
            }
            Self::Retry(_) => {
                write!(f, "error while handling message")
            }
        }
    }
}

impl core::error::Error for QueueError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Fatal(fatal) => Some(&**fatal as &(dyn Error + 'static)),
            Self::Retry(retry) => Some(&**retry as &(dyn Error + 'static)),
        }
    }
}

pub trait HandleCall<T: QueueMessage> {
    fn handle(self, store: &T::Context) -> impl Future<Output = Result<Op<T>, QueueError>> + Send;
}

pub trait HandleCallback<T: QueueMessage> {
    fn handle(
        self,
        ctx: &T::Context,
        data: VecDeque<T::Data>,
    ) -> impl Future<Output = Result<Op<T>, QueueError>> + Send;
}

impl<T: QueueMessage> HandleCall<T> for Never {
    async fn handle(self, _: &<T as QueueMessage>::Context) -> Result<Op<T>, QueueError> {
        match self {}
    }
}

/// Returns the current unix timestamp in seconds.
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub struct Engine<'a, T: QueueMessage, Q: Queue<T>, O: PurePass<T>> {
    store: &'a T::Context,
    queue: &'a Q,
    optimizer: &'a O,
}

pub type BoxDynError = Box<dyn Error + Send + Sync + 'static>;

pub trait Captures<'a> {}
impl<T: ?Sized> Captures<'_> for T {}

impl<'a, T: QueueMessage, Q: Queue<T>, O: PurePass<T>> Engine<'a, T, Q, O> {
    pub fn new(store: &'a T::Context, queue: &'a Q, optimizer: &'a O) -> Self {
        Self {
            store,
            queue,
            optimizer,
        }
    }

    pub fn run(self) -> impl Stream<Item = Result<T::Data, BoxDynError>> + Send + Captures<'a> {
        futures::stream::try_unfold(self, |this| async move {
            sleep(Duration::from_millis(10)).await;
            let res = this.step().await;
            res.map(move |x| x.map(|x| (x, this)))
        })
        .flat_map(|x| stream::iter(x.transpose()))
    }

    pub(crate) fn step<'b>(
        &'b self,
    ) -> impl Future<Output = Result<Option<Option<T::Data>>, BoxDynError>>
           + Captures<'a>
           + Captures<'b>
           + Send {
        // yield back to the runtime and throttle a bit, prevents 100% cpu usage while still allowing for a fast spin-loop
        sleep(Duration::from_millis(10)).then(|()| {
            self.queue
                .process::<_, _, Option<T::Data>, O>(self.optimizer, |msg| {
                    msg.clone().handle(self.store, 0).map(|res| match res {
                        // Ok(Some(Op::Data(d))) => {
                        //     // // TODO: push data to a separate queue
                        //     // let data_output = d.clone().handle(self.store).unwrap();

                        //     // // run to a fixed point
                        //     // if data_output != data(d.clone()) {
                        //     //  (None, Ok(vec![data_output]))
                        //     // } else {
                        //     (Some(d), Ok(vec![]))
                        //     // }
                        // }
                        Ok(msg) => (None, Ok(msg.into_iter().collect())),
                        Err(QueueError::Fatal(fatal)) => {
                            let full_err = ErrorReporter(&*fatal);
                            error!(error = %full_err, "fatal error");
                            (None, Err(full_err.to_string()))
                        }
                        Err(QueueError::Retry(retry)) => {
                            // TODO: Add some backoff logic here based on `full_err`?
                            let full_err = ErrorReporter(&*retry);
                            error!(error = %full_err, "retryable error");
                            (None, Ok(vec![seq([defer(now() + 3), msg])]))
                        }
                    })
                })
                .map(|data| match data {
                    Ok(data) => Ok(Some(data.flatten())),
                    Err(err) => Err(err.into()),
                })
        })
    }
}

pub async fn run_to_completion<
    Cb: DoCallback<T, R>,
    T: QueueMessage,
    R,
    Q: Queue<T>,
    PrePass: PurePass<T>,
    PostPass: Pass<T>,
>(
    callback: Cb,
    ctx: T::Context,
    queue_config: Q::Config,
    msgs: impl IntoIterator<Item = Op<T>>,
    pre_pass_optimizer: PrePass,
    post_pass_optimizer: PostPass,
) -> R {
    let queue = Q::new(queue_config).await.unwrap();

    for msg in msgs {
        queue.enqueue(msg, &pre_pass_optimizer).await.unwrap();
    }

    debug!("spawning optimizer");

    let opt_futures = ctx
        .tags()
        .iter()
        .map(|tag| {
            future::Either::Left(async {
                loop {
                    trace!("optimizing");

                    let res = queue
                        .optimize(tag, &post_pass_optimizer)
                        .await
                        .map_err(|e| {
                            e.map_either::<_, _, BoxDynError, BoxDynError>(
                                |x| Box::new(x),
                                |x| Box::new(x),
                            )
                            .into_inner()
                        });

                    sleep(Duration::from_millis(100)).await;

                    match res {
                        Ok(()) => {}
                        Err(err) => break Err::<Never, _>(err),
                    }
                }
            })
        })
        .chain([future::Either::Right(future::pending())])
        .collect::<FuturesUnordered<_>>();

    debug!("running");

    let engine_output = Engine::new(&ctx, &queue, &pre_pass_optimizer)
        .run()
        .take(Cb::Params::LEN)
        .try_collect::<VecDeque<_>>();

    pin!(opt_futures);
    pin!(engine_output);

    let output = match future::select(opt_futures.next(), engine_output).await {
        future::Either::Left((err, _)) => {
            let Some(Err(err)) = err else {
                panic!("will always contain at least one future; qed;")
            };
            panic!("optimizer returned error: {}", ErrorReporter(&*err));
        }
        future::Either::Right((x, fut)) => {
            drop(fut);

            x.unwrap()
        }
    };

    let data = match HListTryFromIterator::try_from_iter(output) {
        Ok(ok) => ok,
        Err(_) => {
            panic!(
                "could not aggregate data into {}",
                std::any::type_name::<Cb>()
            )
        }
    };

    // dbg!(queue);

    Cb::call(callback, data)
}

#[derive(DebugNoBound, CloneNoBound)]
pub struct InMemoryQueue<T: QueueMessage> {
    idx: Arc<AtomicU32>,
    ready: Arc<Mutex<BTreeMap<u32, Item<T>>>>,
    done: Arc<Mutex<BTreeMap<u32, Item<T>>>>,
    #[allow(clippy::type_complexity)]
    optimizer_queue: Arc<Mutex<BTreeMap<String, BTreeMap<u32, Item<T>>>>>,
}

#[derive(DebugNoBound, CloneNoBound)]
struct Item<T: QueueMessage> {
    #[allow(dead_code)] // used in debug
    parents: Vec<u32>,
    msg: Op<T>,
}

impl<T: QueueMessage> Queue<T> for InMemoryQueue<T> {
    type Error = std::convert::Infallible;
    type Config = ();

    fn new(_cfg: Self::Config) -> impl Future<Output = Result<Self, Self::Error>> {
        futures::future::ok(Self {
            idx: Arc::new(AtomicU32::default()),
            done: Arc::new(Mutex::new(BTreeMap::default())),
            ready: Arc::new(Mutex::new(BTreeMap::default())),
            optimizer_queue: Arc::new(Mutex::new(BTreeMap::default())),
        })
    }

    fn enqueue<'a, O: PurePass<T>>(
        &'a self,
        item: Op<T>,
        pre_enqueue_passes: &'a O,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'a {
        debug!(?item, "enqueueing new item");

        let (parent_idxs, normalized) = normalize::normalize(vec![item])
            .into_iter()
            .unzip::<_, _, Vec<_>, Vec<_>>();

        let res = {
            let mut res = pre_enqueue_passes.run_pass_pure(normalized);

            for (parents, _) in &mut res.ready {
                *parents = parents
                    .iter()
                    .flat_map(|p| &parent_idxs[*p])
                    .copied()
                    .collect();
            }

            for (parents, _, _) in &mut res.optimize_further {
                *parents = parents
                    .iter()
                    .flat_map(|p| &parent_idxs[*p])
                    .copied()
                    .collect();
            }

            res
        };

        let mut optimizer_queue = self.optimizer_queue.lock().expect("mutex is poisoned");
        let mut ready = self.ready.lock().expect("mutex is poisoned");

        self.requeue(
            res,
            &mut optimizer_queue,
            &mut ready,
            // no parents
            |parent_idxs: Vec<usize>| {
                assert_eq!(parent_idxs, [0]);
                vec![]
            },
        );

        debug!("enqueued new item");

        futures::future::ok(())
    }

    async fn process<'a, F, Fut, R, O>(
        &'a self,
        pre_reenqueue_passes: &'a O,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: (FnOnce(Op<T>) -> Fut) + Send + Captures<'a>,
        Fut: Future<Output = (R, Result<Vec<Op<T>>, String>)> + Send + Captures<'a>,
        R: Send + Sync + 'static,
        O: PurePass<T>,
    {
        let msg = {
            let mut queue = self.ready.lock().expect("mutex is poisoned");
            let msg = queue.pop_first();

            drop(queue);

            msg
        };

        match msg {
            Some((id, item)) => {
                let span = info_span!("processing item", %id);
                // tracing::info!(
                //     json = %serde_json::to_string(&msg).unwrap(),
                // );

                self.done
                    .lock()
                    .expect("mutex is poisoned")
                    .insert(id, item.clone());

                let (r, res) = f(item.msg.clone()).instrument(span).await;
                match res {
                    Ok(new_msgs) => {
                        let (_, new_msgs) = normalize::normalize(new_msgs)
                            .into_iter()
                            .unzip::<_, _, Vec<_>, Vec<_>>();

                        let res = pre_reenqueue_passes.run_pass_pure(new_msgs);

                        let mut optimizer_queue =
                            self.optimizer_queue.lock().expect("mutex is poisoned");
                        let mut ready = self.ready.lock().expect("mutex is poisoned");

                        self.requeue(
                            res,
                            &mut optimizer_queue,
                            &mut ready,
                            |parent_idxs: Vec<usize>| {
                                // all new items are from the item that was just processed
                                assert_eq!(parent_idxs, [0]);
                                vec![id]
                            },
                        );

                        Ok(Some(r))
                    }
                    Err(why) => panic!("{why}"),
                }
            }
            None => {
                // trace!("queue is empty, sleeping for 1 second");

                // sleep(Duration::from_secs(1)).await;

                Ok(None)
            }
        }
    }

    #[allow(clippy::manual_async_fn)]
    fn optimize<'a, O: Pass<T>>(
        &'a self,
        tag: &'a str,
        optimizer: &'a O,
    ) -> impl Future<Output = Result<(), Either<Self::Error, O::Error>>> + 'a {
        async move {
            let tagged_optimizer_queue = {
                let mut optimizer_queue = self.optimizer_queue.lock().unwrap();
                let Some(tagged_optimizer_queue) = optimizer_queue.remove(tag) else {
                    warn!(%tag, "no items with tag");
                    return Ok(());
                };

                drop(optimizer_queue);

                tagged_optimizer_queue
            };

            let (ids, msgs): (Vec<_>, Vec<_>) = tagged_optimizer_queue.clone().into_iter().unzip();

            let res = optimizer
                .run_pass(msgs.into_iter().map(|item| item.msg).collect())
                .await
                .map_err(Either::Right)?;

            // dbg!(&res, std::any::type_name::<O>());

            let mut optimizer_queue = self.optimizer_queue.lock().unwrap();
            let mut ready = self.ready.lock().unwrap();
            let mut done = self.done.lock().unwrap();

            done.append(&mut tagged_optimizer_queue.clone());

            self.requeue(
                res,
                &mut optimizer_queue,
                &mut ready,
                |parent_idxs: Vec<usize>| {
                    ids.iter()
                        .enumerate()
                        .filter_map(|(idx, id)| parent_idxs.contains(&idx).then_some(*id))
                        .collect::<Vec<_>>()
                },
            );

            Ok(())
        }
    }
}

impl<T: QueueMessage> InMemoryQueue<T> {
    fn requeue(
        &self,
        res: OptimizationResult<T>,
        optimizer_queue: &mut BTreeMap<String, BTreeMap<u32, Item<T>>>,
        ready: &mut BTreeMap<u32, Item<T>>,
        get_parent_ids: impl Fn(Vec<usize>) -> Vec<u32>,
    ) {
        for (parent_idxs, new_msg, tag) in res.optimize_further {
            optimizer_queue.entry(tag).or_default().insert(
                self.idx.fetch_add(1, Ordering::SeqCst),
                Item {
                    parents: get_parent_ids(parent_idxs),
                    msg: new_msg,
                },
            );
        }

        for (parent_idxs, new_msg) in res.ready {
            ready.insert(
                self.idx.fetch_add(1, Ordering::SeqCst),
                Item {
                    parents: get_parent_ids(parent_idxs),
                    msg: new_msg,
                },
            );
        }
    }
}

#[cfg(test)]
pub mod test_utils {
    use std::collections::VecDeque;

    use enumorph::Enumorph;
    use frunk::{hlist_pat, HList};
    use macros::model;
    use queue_msg_macro::SubsetOf;

    use crate::{
        aggregation::{do_callback, DoCallback},
        call, data, noop, HandleCall, HandleCallback, Op, QueueError, QueueMessage,
    };

    // for macros
    pub mod queue_msg {
        pub use crate::*;
    }

    pub enum SimpleMessage {}

    impl QueueMessage for SimpleMessage {
        type Data = SimpleData;
        type Call = SimpleCall;
        type Callback = SimpleAggregate;

        type Context = ();
    }

    impl HandleCall<SimpleMessage> for SimpleCall {
        async fn handle(self, _: &()) -> Result<Op<SimpleMessage>, QueueError> {
            Ok(match self {
                SimpleCall::A(FetchA {}) => data(DataA {}),
                SimpleCall::B(FetchB {}) => data(DataB {}),
                SimpleCall::C(FetchC {}) => data(DataC {}),
                SimpleCall::D(FetchD {}) => data(DataD {}),
                SimpleCall::E(FetchE {}) => data(DataE {}),
                SimpleCall::PrintAbc(PrintAbc { a, b, c }) => {
                    println!("a = {a:?}, b = {b:?}, c = {c:?}");
                    noop()
                }
            })
        }
    }

    impl HandleCallback<SimpleMessage> for SimpleAggregate {
        async fn handle(
            self,
            _: &(),
            data: VecDeque<SimpleData>,
        ) -> Result<Op<SimpleMessage>, QueueError> {
            Ok(match self {
                Self::BuildPrintAbc(agg) => do_callback(agg, data),
            })
        }
    }

    #[model]
    #[derive(Enumorph, SubsetOf)]
    pub enum SimpleData {
        A(DataA),
        B(DataB),
        C(DataC),
        D(DataD),
        E(DataE),
    }
    #[model]
    pub struct DataA {}
    #[model]
    pub struct DataB {}
    #[model]
    pub struct DataC {}
    #[model]
    pub struct DataD {}
    #[model]
    pub struct DataE {}

    #[model]
    #[derive(Enumorph, SubsetOf)]
    pub enum SimpleCall {
        A(FetchA),
        B(FetchB),
        C(FetchC),
        D(FetchD),
        E(FetchE),
        PrintAbc(PrintAbc),
    }
    #[model]
    pub struct FetchA {}
    #[model]
    pub struct FetchB {}
    #[model]
    pub struct FetchC {}
    #[model]
    pub struct FetchD {}
    #[model]
    pub struct FetchE {}

    #[model]
    pub struct PrintAbc {
        pub a: DataA,
        pub b: DataB,
        pub c: DataC,
    }

    #[model]
    pub struct SimpleWait {}

    #[model]
    #[derive(Enumorph)]
    pub enum SimpleAggregate {
        BuildPrintAbc(BuildPrintAbc),
    }

    #[model]
    pub struct BuildPrintAbc {}

    impl DoCallback<SimpleMessage> for BuildPrintAbc {
        type Params = HList![DataA, DataB, DataC];

        fn call(BuildPrintAbc {}: Self, hlist_pat![a, b, c]: Self::Params) -> Op<SimpleMessage> {
            call(PrintAbc { a, b, c })
        }
    }
}
