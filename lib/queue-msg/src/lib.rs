#![feature(trait_alias, extract_if)]
// #![warn(clippy::large_futures, clippy::large_stack_frames)]

use std::{
    collections::{BTreeMap, VecDeque},
    error::Error,
    fmt::Debug,
    future::Future,
    num::NonZeroU64,
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
    stream::{self, try_unfold},
    Stream, StreamExt, TryStreamExt,
};
pub use queue_msg_macro::queue_msg;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::time::sleep;
use tracing::{debug, error, info, info_span, trace, warn, Instrument};
use unionlabs::{never::Never, MaybeArbitrary};

use crate::{
    aggregation::{HListTryFromIterator, UseAggregate},
    optimize::{OptimizationResult, Pass, PurePass},
};

pub mod aggregation;
pub mod optimize;

pub trait Queue<T: QueueMessageTypes>: Debug + Clone + Send + Sync + Sized + 'static {
    /// Error type returned by this queue, representing errors that are out of control of the
    /// consumer (i.e. unable to connect to database, can't insert into row, can't deserialize row,
    /// etc)
    type Error: Error + Send + Sync + 'static;
    type Config: Debug + Clone + Serialize + DeserializeOwned;

    fn new(cfg: Self::Config) -> impl Future<Output = Result<Self, Self::Error>>;

    /// Enqueue an item into the queue, running a pure optimization pass on the item before enqueueing it.
    ///
    /// All items will be enqueued to be optimized, unless marked as ready by [`O`].
    fn enqueue<'a, O: PurePass<T>>(
        &'a self,
        item: QueueMsg<T>,
        pre_enqueue_passes: &'a O,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'a;

    /// Process the item at the front of the queue, if there is one. New items will be pre-processed by [`O`] before being reenqueued.
    ///
    /// All items will be enqueued to be optimized, unless marked as ready by [`O`].
    fn process<'a, F, Fut, R, O>(
        &'a self,
        pre_reenqueue_passes: &'a O,
        f: F,
    ) -> impl Future<Output = Result<Option<R>, Self::Error>> + Send + '_
    where
        F: (FnOnce(QueueMsg<T>) -> Fut) + Send + 'static,
        Fut: Future<Output = (R, Result<Vec<QueueMsg<T>>, String>)> + Send + 'static,
        R: Send + Sync + 'static,
        O: PurePass<T>;

    fn optimize<'a, O: Pass<T>>(
        &'a self,
        optimizer: &'a O,
    ) -> impl Future<Output = Result<(), Either<Self::Error, O::Error>>> + Send + 'a;
}

#[queue_msg]
#[debug(bound())]
pub enum QueueMsg<T: QueueMessageTypes> {
    /// An external event. This could be something like an IBC event, an external command, or
    /// anything else that occurs outside of the state machine. Can also be thought of as an "entry
    /// point".
    Event(T::Event),
    /// Inert data that will either be used in an aggregation or bubbled up to the top and sent as
    /// an output.
    Data(T::Data),
    /// Fetch some data from the outside world. This can also be thought of as a "read" operation.
    Fetch(T::Fetch),
    /// Send a message to the outside world. This can also be thought of as a "write" operation.
    Effect(T::Effect),
    /// Wait for some external condition.
    Wait(T::Wait),

    Defer(Defer),
    /// Repeats the contained message `times` times. If `times` is `None`, will repeat infinitely.
    Repeat {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        times: Option<NonZeroU64>,
        msg: Box<Self>,
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
    Sequence(VecDeque<Self>),
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
    /// Note that this is similar to `Sequence`, but the new messages are queued at the *back* of
    /// the list, allowing for uniform progress across all nested messages.
    Concurrent(VecDeque<Self>),
    // REVIEW: Remove? We don't use this
    Retry {
        remaining: u8,
        msg: Box<Self>,
    },
    Aggregate {
        /// Messages that are expected to resolve to [`Data`].
        queue: VecDeque<Self>,
        /// The resolved data messages.
        data: VecDeque<T::Data>,
        /// The message that will utilize the aggregated data.
        receiver: T::Aggregate,
    },
    /// Handle the contained message, voiding any returned `Data` messages that it returns.
    Void(Box<Self>),
    /// Race a list of messages. The head of the list is handled, and if it returns no new messages,
    /// then the rest of the list is dropped; otherwise, the new message is pushed to the back of the
    /// list (similar to [`Self::Concurrent`]).
    ///
    /// ```txt
    /// [A B C]
    /// D = handle(A)
    /// if D.is_none() noop else race([B C D])
    /// ```
    Race(VecDeque<Self>),
    Noop,
}

#[queue_msg]
pub enum Defer {
    Absolute(u64),
    Relative(u64),
}

#[inline]
#[must_use = "constructing an instruction has no effect"]
pub fn retry<T: QueueMessageTypes>(count: u8, t: impl Into<QueueMsg<T>>) -> QueueMsg<T> {
    QueueMsg::Retry {
        remaining: count,
        msg: Box::new(t.into()),
    }
}

#[inline]
#[must_use = "constructing an instruction has no effect"]
pub fn repeat<T: QueueMessageTypes>(
    times: impl Into<Option<NonZeroU64>>,
    t: impl Into<QueueMsg<T>>,
) -> QueueMsg<T> {
    QueueMsg::Repeat {
        times: times.into(),
        msg: Box::new(t.into()),
    }
}

#[inline]
#[must_use = "constructing an instruction has no effect"]
pub fn seq<T: QueueMessageTypes>(ts: impl IntoIterator<Item = QueueMsg<T>>) -> QueueMsg<T> {
    QueueMsg::Sequence(ts.into_iter().collect())
}

#[inline]
#[must_use = "constructing an instruction has no effect"]
pub fn conc<T: QueueMessageTypes>(ts: impl IntoIterator<Item = QueueMsg<T>>) -> QueueMsg<T> {
    QueueMsg::Concurrent(ts.into_iter().collect())
}

#[inline]
#[must_use = "constructing an instruction has no effect"]
pub fn defer_absolute<T: QueueMessageTypes>(timestamp: u64) -> QueueMsg<T> {
    QueueMsg::Defer(Defer::Absolute(timestamp))
}

#[inline]
#[must_use = "constructing an instruction has no effect"]
pub fn defer_relative<T: QueueMessageTypes>(seconds: u64) -> QueueMsg<T> {
    QueueMsg::Defer(Defer::Relative(seconds))
}

#[inline]
#[must_use = "constructing an instruction has no effect"]
pub fn fetch<T: QueueMessageTypes>(t: impl Into<T::Fetch>) -> QueueMsg<T> {
    QueueMsg::Fetch(t.into())
}

#[inline]
#[must_use = "constructing an instruction has no effect"]
pub fn effect<T: QueueMessageTypes>(t: impl Into<T::Effect>) -> QueueMsg<T> {
    QueueMsg::Effect(t.into())
}

#[inline]
#[must_use = "constructing an instruction has no effect"]
pub fn data<T: QueueMessageTypes>(t: impl Into<T::Data>) -> QueueMsg<T> {
    QueueMsg::Data(t.into())
}

#[inline]
#[must_use = "constructing an instruction has no effect"]
pub fn wait<T: QueueMessageTypes>(t: impl Into<T::Wait>) -> QueueMsg<T> {
    QueueMsg::Wait(t.into())
}

#[inline]
#[must_use = "constructing an instruction has no effect"]
pub fn event<T: QueueMessageTypes>(t: impl Into<T::Event>) -> QueueMsg<T> {
    QueueMsg::Event(t.into())
}

#[inline]
#[must_use = "constructing an instruction has no effect"]
pub fn aggregate<T: QueueMessageTypes>(
    queue: impl IntoIterator<Item = QueueMsg<T>>,
    data: impl IntoIterator<Item = T::Data>,
    receiver: impl Into<T::Aggregate>,
) -> QueueMsg<T> {
    QueueMsg::Aggregate {
        queue: queue.into_iter().collect(),
        data: data.into_iter().collect(),
        receiver: receiver.into(),
    }
}

#[inline]
#[must_use = "constructing an instruction has no effect"]
pub fn void<T: QueueMessageTypes>(t: impl Into<QueueMsg<T>>) -> QueueMsg<T> {
    QueueMsg::Void(Box::new(t.into()))
}

#[inline]
#[must_use = "constructing an instruction has no effect"]
pub fn race<T: QueueMessageTypes>(ts: impl IntoIterator<Item = QueueMsg<T>>) -> QueueMsg<T> {
    QueueMsg::Race(ts.into_iter().collect())
}

#[inline]
#[must_use = "constructing an instruction has no effect"]
pub fn noop<T: QueueMessageTypes>() -> QueueMsg<T> {
    QueueMsg::Noop
}

pub trait QueueMsgTypesTraits = Debug
    + Clone
    + PartialEq
    + Serialize
    + for<'a> Deserialize<'a>
    + Send
    + Sync
    + Unpin
    + MaybeArbitrary;

pub trait QueueMessageTypes: Sized + 'static {
    type Event: HandleEvent<Self> + QueueMsgTypesTraits;
    type Data: HandleData<Self> + QueueMsgTypesTraits;
    type Fetch: HandleFetch<Self> + QueueMsgTypesTraits;
    type Effect: HandleEffect<Self> + QueueMsgTypesTraits;
    type Wait: HandleWait<Self> + QueueMsgTypesTraits;
    type Aggregate: HandleAggregate<Self> + QueueMsgTypesTraits;

    type Store: Debug + Send + Sync;
}

impl<T: QueueMessageTypes> QueueMsg<T> {
    // NOTE: Box is required bc recursion
    #[allow(clippy::type_complexity)]
    pub fn handle<'a>(
        self,
        store: &'a T::Store,
        depth: usize,
    ) -> Pin<Box<dyn Future<Output = Result<Option<QueueMsg<T>>, QueueError>> + Send + 'a>> {
        debug!(depth, "handling message");

        let fut = async move {
            match self {
                Self::Event(event) => event.handle(store).map(Some),
                Self::Data(data) => data.handle(store).map(Some),

                Self::Fetch(fetch) => fetch.handle(store).await.map(Some),
                Self::Effect(msg) => msg.handle(store).await.map(Some),
                Self::Wait(wait) => wait.handle(store).await.map(Some),

                Self::Defer(Defer::Relative(seconds)) => Ok(Some(defer_absolute(now() + seconds))),
                Self::Defer(Defer::Absolute(seconds)) => {
                    // if we haven't hit the time yet, requeue the defer msg
                    if now() < seconds {
                        // TODO: Make the time configurable?
                        sleep(Duration::from_secs(1)).await;

                        Ok(Some(defer_absolute(seconds)))
                    } else {
                        Ok(None)
                    }
                }
                Self::Timeout {
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
                Self::Sequence(mut queue) => match queue.pop_front() {
                    Some(msg) => {
                        let msg = msg.handle(store, depth + 1).await?;

                        if let Some(msg) = msg {
                            queue.push_front(msg)
                        }

                        Ok(Some(seq(queue)))
                    }
                    None => Ok(None),
                },
                Self::Concurrent(mut queue) => match queue.pop_front() {
                    Some(msg) => {
                        let msg = msg.handle(store, depth + 1).await?;

                        if let Some(msg) = msg {
                            queue.push_back(msg)
                        }

                        Ok(Some(conc(queue)))
                    }
                    None => Ok(None),
                },
                Self::Retry { remaining, msg } => {
                    const RETRY_DELAY_SECONDS: u64 = 3;

                    match msg.clone().handle(store, depth + 1).await {
                        Ok(ok) => Ok(ok),
                        Err(err) => {
                            if remaining > 0 {
                                let retries_left = remaining - 1;

                                warn!(
                                    retries_left,
                                    ?err,
                                    "msg failed, retrying in {RETRY_DELAY_SECONDS} seconds"
                                );

                                Ok(Some(seq([
                                    defer_absolute(now() + RETRY_DELAY_SECONDS),
                                    retry(retries_left, *msg),
                                ])))
                            } else {
                                error!("msg failed after all retries");
                                Err(err)
                            }
                        }
                    }
                }
                Self::Aggregate {
                    mut queue,
                    mut data,
                    receiver,
                } => {
                    if let Some(msg) = queue.pop_front() {
                        let msg = msg.handle(store, depth + 1).await?;

                        if let Some(msg) = msg {
                            match msg {
                                Self::Data(d) => {
                                    data.push_back(d);
                                }
                                m => {
                                    queue.push_back(m);
                                }
                            }
                        }

                        Ok(Some(aggregate(queue, data, receiver)))
                    } else {
                        // queue is empty, handle msg
                        receiver.handle(data).map(Some)
                    }
                }
                Self::Repeat { times: None, msg } => {
                    Ok(Some(seq([*msg.clone(), repeat(None, *msg)])))
                }
                Self::Repeat {
                    times: Some(times),
                    msg,
                } => Ok(Some(seq([*msg.clone()].into_iter().chain(
                    // if times - 1 > 0, queue repeat with times - 1
                    NonZeroU64::new(times.get() - 1_u64).map(|times| repeat(Some(times), *msg)),
                )))),
                QueueMsg::Void(msg) => {
                    // TODO: distribute across seq/conc
                    Ok(msg.handle(store, depth + 1).await?.map(|msg| match msg {
                        QueueMsg::Data(data) => {
                            debug!(data = %serde_json::to_string(&data).unwrap(), "voiding data");
                            noop()
                        }
                        msg => void(msg),
                    }))
                }
                QueueMsg::Race(mut msgs) => match msgs.pop_front() {
                    Some(msg) => {
                        match msg.handle(store, depth + 1).await? {
                            Some(msg) => {
                                msgs.push_back(msg);
                                Ok(Some(race(msgs)))
                            }
                            // head won, drop the rest of the messages
                            None => {
                                info!("race won, dropping other messages");
                                Ok(None)
                            }
                        }
                    }
                    None => Ok(None),
                },
                QueueMsg::Noop => Ok(None),
            }
        };

        Box::pin(fut)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimize::{
        passes::{ExtractData, FlattenConc, FlattenSeq},
        PurePass,
    };

    enum UnitMessage {}

    impl QueueMessageTypes for UnitMessage {
        type Event = ();
        type Data = ();
        type Fetch = ();
        type Effect = ();
        type Wait = ();

        type Aggregate = ();

        type Store = ();
    }

    impl HandleEffect<UnitMessage> for () {
        async fn handle(self, _: &()) -> Result<QueueMsg<UnitMessage>, QueueError> {
            Ok(noop())
        }
    }

    impl HandleEvent<UnitMessage> for () {
        fn handle(self, _: &()) -> Result<QueueMsg<UnitMessage>, QueueError> {
            Ok(noop())
        }
    }

    impl HandleData<UnitMessage> for () {
        fn handle(self, _: &()) -> Result<QueueMsg<UnitMessage>, QueueError> {
            Ok(noop())
        }
    }

    impl HandleFetch<UnitMessage> for () {
        async fn handle(self, _: &()) -> Result<QueueMsg<UnitMessage>, QueueError> {
            Ok(noop())
        }
    }

    impl HandleWait<UnitMessage> for () {
        async fn handle(self, _: &()) -> Result<QueueMsg<UnitMessage>, QueueError> {
            Ok(noop())
        }
    }

    impl HandleAggregate<UnitMessage> for () {
        fn handle(self, _: VecDeque<()>) -> Result<QueueMsg<UnitMessage>, QueueError> {
            Ok(noop())
        }
    }

    #[queue_msg]
    pub struct SimpleEvent {}
    #[queue_msg]
    pub struct SimpleData {}
    #[queue_msg]
    pub struct SimpleFetch {}
    #[queue_msg]
    pub struct SimpleEffect {}
    #[queue_msg]
    pub struct SimpleWait {}

    #[queue_msg]
    pub struct SimpleAggregate {}

    // macro_rules! vec_deque {
    //     ($($tt:tt)*) => {
    //         ::std::collections::VecDeque::from(vec![$($tt)*])
    //     };
    // }

    // async fn assert_steps<T: QueueMessageTypes>(
    //     engine: &Engine<T>,
    //     q: &mut InMemoryQueue<T>,
    //     steps: impl IntoIterator<Item = VecDeque<QueueMsg<T>>>,
    // ) {
    //     for (i, step) in steps.into_iter().enumerate() {
    //         engine.step(q).await.unwrap();
    //         assert_eq!(*q.queue.lock().unwrap(), step, "step {i} incorrect");
    //     }
    // }

    #[test]
    fn flatten() {
        let msg = seq::<UnitMessage>([
            defer_absolute(1),
            seq([defer_absolute(2), seq([defer_absolute(3)])]),
            seq([defer_absolute(4)]),
            defer_absolute(5),
        ]);
        assert_eq!(
            FlattenSeq.run_pass_pure(vec![msg]).optimize_further,
            vec![(
                vec![0],
                seq([
                    defer_absolute(1),
                    defer_absolute(2),
                    defer_absolute(3),
                    defer_absolute(4),
                    defer_absolute(5)
                ])
            )]
        );

        let msg = seq::<UnitMessage>([defer_absolute(1)]);
        assert_eq!(
            FlattenSeq.run_pass_pure(vec![msg]).optimize_further,
            vec![(vec![0], defer_absolute(1))]
        );

        let msg = conc::<UnitMessage>([defer_absolute(1)]);
        assert_eq!(
            FlattenSeq.run_pass_pure(vec![msg]).optimize_further,
            vec![(vec![0], conc([defer_absolute(1)]))]
        );

        let msg = conc::<UnitMessage>([seq([defer_absolute(1)])]);
        assert_eq!(
            FlattenSeq.run_pass_pure(vec![msg]).optimize_further,
            vec![(vec![0], conc([defer_absolute(1)]))]
        );

        let msg = seq::<UnitMessage>([noop()]);
        assert_eq!(
            FlattenSeq.run_pass_pure(vec![msg]).optimize_further,
            vec![(vec![0], noop())]
        );

        let msg = conc::<UnitMessage>([seq([noop()])]);
        assert_eq!(
            FlattenSeq.run_pass_pure(vec![msg]).optimize_further,
            vec![(vec![0], conc([noop()]))]
        );

        let msg = conc::<UnitMessage>([conc([conc([noop()])])]);
        assert_eq!(
            FlattenConc.run_pass_pure(vec![msg]).optimize_further,
            vec![(vec![0], noop())]
        );
    }

    #[test]
    fn nested_seq_conc_single() {
        // any nesting level of seq and conc should be handled in a single pass of (seq, conc) or
        // (conc, seq)

        let msg = conc::<UnitMessage>([seq([conc([noop()])])]);
        assert_eq!(
            (FlattenConc, FlattenSeq)
                .run_pass_pure(vec![msg])
                .optimize_further,
            vec![(vec![0], noop())]
        );

        let msg = conc::<UnitMessage>([seq([conc([seq([conc([seq([conc([noop()])])])])])])]);
        assert_eq!(
            (FlattenConc, FlattenSeq)
                .run_pass_pure(vec![msg])
                .optimize_further,
            vec![(vec![0], noop())]
        );

        let msg =
            conc::<UnitMessage>([seq([conc([seq([conc([seq([conc([seq([conc([
                noop(),
            ])])])])])])])])]);
        assert_eq!(
            (FlattenConc, FlattenSeq)
                .run_pass_pure(vec![msg])
                .optimize_further,
            vec![(vec![0], noop())]
        );

        let msg = seq::<UnitMessage>([conc([seq([conc([noop()])])])]);
        assert_eq!(
            (FlattenConc, FlattenSeq)
                .run_pass_pure(vec![msg])
                .optimize_further,
            vec![(vec![0], noop())]
        );

        let msg = seq::<UnitMessage>([conc([seq([conc([seq([conc([seq([conc([noop()])])])])])])])]);
        assert_eq!(
            (FlattenConc, FlattenSeq)
                .run_pass_pure(vec![msg])
                .optimize_further,
            vec![(vec![0], noop())]
        );

        let msg = seq::<UnitMessage>([conc([seq([conc([seq([conc([seq([conc([seq([
            conc([noop()]),
        ])])])])])])])])]);
        assert_eq!(
            (FlattenConc, FlattenSeq)
                .run_pass_pure(vec![msg])
                .optimize_further,
            vec![(vec![0], noop())]
        );
    }

    #[test]
    fn flatten_seq_conc_fixed_point_is_noop() {
        // this message can't be optimized any further, flattening operations should be a noop

        let msg = seq::<UnitMessage>([
            conc([defer_absolute(1), defer_absolute(2)]),
            defer_absolute(3),
        ]);
        assert_eq!(
            (FlattenConc, FlattenSeq)
                .run_pass_pure(vec![msg.clone()])
                .optimize_further,
            vec![(vec![0], msg.clone())]
        );
        assert_eq!(
            (FlattenSeq, FlattenConc)
                .run_pass_pure(vec![msg.clone()])
                .optimize_further,
            vec![(vec![0], msg)]
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
            ExtractData.run_pass_pure(vec![msg]).optimize_further,
            vec![
                (vec![0], data(())),
                (vec![0], data(())),
                (vec![0], data(())),
                (vec![0], data(())),
                (vec![0], data(())),
                (vec![0], seq([seq([seq([])]), seq([])])),
            ],
        );
    }

    #[test]
    fn extract_data_complex() {
        let msg = seq::<UnitMessage>([
            data(()),
            effect(()),
            seq([fetch(()), data(()), seq([data(())])]),
            effect(()),
            seq([data(()), effect(())]),
            data(()),
        ]);
        assert_eq!(
            ExtractData.run_pass_pure(vec![msg]).optimize_further,
            vec![
                (vec![0], data(())),
                (vec![0], data(())),
                (vec![0], data(())),
                (vec![0], data(())),
                (vec![0], data(())),
                (
                    vec![0],
                    seq([
                        effect(()),
                        seq([fetch(()), seq([])]),
                        effect(()),
                        seq([effect(())])
                    ])
                ),
            ],
        );
    }

    // #[tokio::test]
    // async fn conc_seq_nested() {
    //     let engine = Engine::new(Arc::new(()));

    //     let mut q = InMemoryQueue::new(()).now_or_never().unwrap().unwrap();

    //     let msgs = seq::<UnitMessageTypes>([
    //         conc([fetch(()), fetch(())]),
    //         conc([wait(()), wait(())]),
    //         conc([
    //             repeat(None, fetch(())),
    //             repeat(None, fetch(())),
    //             effect(()),
    //             seq([fetch(()), fetch(()), effect(())]),
    //         ]),
    //     ]);

    //     q.enqueue(msgs).await.unwrap();

    //     // assert_steps(
    //     //     &engine,
    //     //     &mut q,
    //     //     [
    //     //         vec_deque![seq::<UnitMessageTypes>([
    //     //             // conc(a, b), handles a, conc(b) == b
    //     //             fetch(()),
    //     //             conc([wait(()), wait(())]),
    //     //             conc([
    //     //                 repeat(None, fetch(())),
    //     //                 repeat(None, fetch(())),
    //     //                 effect(()),
    //     //                 seq([fetch(()), fetch(()), effect(())]),
    //     //             ]),
    //     //         ])],
    //     //         vec_deque![seq::<UnitMessageTypes>([
    //     //             conc([wait(()), wait(())]),
    //     //             conc([
    //     //                 repeat(None, fetch(())),
    //     //                 repeat(None, fetch(())),
    //     //                 effect(()),
    //     //                 seq([fetch(()), fetch(()), effect(())]),
    //     //             ]),
    //     //         ])],
    //     //         vec_deque![seq::<UnitMessageTypes>([
    //     //             // conc(a, b), handles a, conc(b) == b
    //     //             wait(()),
    //     //             conc([
    //     //                 repeat(None, fetch(())),
    //     //                 repeat(None, fetch(())),
    //     //                 effect(()),
    //     //                 seq([fetch(()), fetch(()), effect(())]),
    //     //             ]),
    //     //         ])],
    //     //         // seq(a, conc(m...)), handles a, seq(conc(m...)) == m...
    //     //         vec_deque![
    //     //             repeat(None, fetch(())),
    //     //             repeat(None, fetch(())),
    //     //             effect(()),
    //     //             seq([fetch(()), fetch(()), effect(())]),
    //     //         ],
    //     //         vec_deque![
    //     //             repeat(None, fetch(())),
    //     //             effect(()),
    //     //             seq([fetch(()), fetch(()), effect(())]),
    //     //             // repeat(a) queues seq(a, repeat(a))
    //     //             seq([fetch(()), repeat(None, fetch(()))])
    //     //         ],
    //     //         vec_deque![
    //     //             effect(()),
    //     //             seq([fetch(()), fetch(()), effect(())]),
    //     //             seq([fetch(()), repeat(None, fetch(()))]),
    //     //             // repeat(a) queues seq(a, repeat(a))
    //     //             seq([fetch(()), repeat(None, fetch(()))])
    //     //         ],
    //     //         vec_deque![
    //     //             seq([fetch(()), fetch(()), effect(())]),
    //     //             seq([fetch(()), repeat(None, fetch(()))]),
    //     //             seq([fetch(()), repeat(None, fetch(()))]),
    //     //             noop(),
    //     //         ],
    //     //         vec_deque![
    //     //             seq([fetch(()), repeat(None, fetch(()))]),
    //     //             seq([fetch(()), repeat(None, fetch(()))]),
    //     //             noop(),
    //     //             seq([fetch(()), effect(())]),
    //     //         ],
    //     //         vec_deque![
    //     //             seq([fetch(()), repeat(None, fetch(()))]),
    //     //             noop(),
    //     //             seq([fetch(()), effect(())]),
    //     //             repeat(None, fetch(())),
    //     //         ],
    //     //         vec_deque![
    //     //             noop(),
    //     //             seq([fetch(()), effect(())]),
    //     //             repeat(None, fetch(())),
    //     //             repeat(None, fetch(())),
    //     //         ],
    //     //         vec_deque![
    //     //             seq([fetch(()), effect(())]),
    //     //             repeat(None, fetch(())),
    //     //             repeat(None, fetch(())),
    //     //         ],
    //     //         vec_deque![repeat(None, fetch(())), repeat(None, fetch(())), effect(()),],
    //     //         vec_deque![
    //     //             repeat(None, fetch(())),
    //     //             effect(()),
    //     //             seq([fetch(()), repeat(None, fetch(()))]),
    //     //         ],
    //     //         vec_deque![
    //     //             effect(()),
    //     //             seq([fetch(()), repeat(None, fetch(()))]),
    //     //             seq([fetch(()), repeat(None, fetch(()))]),
    //     //         ],
    //     //         vec_deque![
    //     //             seq([fetch(()), repeat(None, fetch(()))]),
    //     //             seq([fetch(()), repeat(None, fetch(()))]),
    //     //             noop(),
    //     //         ],
    //     //     ],
    //     // )
    //     // .await;
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

impl std::error::Error for QueueError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Fatal(fatal) => Some(&**fatal as &(dyn Error + 'static)),
            Self::Retry(retry) => Some(&**retry as &(dyn Error + 'static)),
        }
    }
}

pub trait HandleFetch<T: QueueMessageTypes> {
    fn handle(
        self,
        store: &T::Store,
    ) -> impl Future<Output = Result<QueueMsg<T>, QueueError>> + Send;
}

pub trait HandleData<T: QueueMessageTypes> {
    fn handle(self, store: &T::Store) -> Result<QueueMsg<T>, QueueError>;
}

pub trait HandleWait<T: QueueMessageTypes> {
    fn handle(
        self,
        store: &T::Store,
    ) -> impl Future<Output = Result<QueueMsg<T>, QueueError>> + Send;
}

pub trait HandleEvent<T: QueueMessageTypes> {
    fn handle(self, store: &T::Store) -> Result<QueueMsg<T>, QueueError>;
}

pub trait HandleEffect<T: QueueMessageTypes> {
    fn handle(
        self,
        store: &T::Store,
    ) -> impl Future<Output = Result<QueueMsg<T>, QueueError>> + Send;
}

pub trait HandleAggregate<T: QueueMessageTypes> {
    fn handle(self, data: VecDeque<T::Data>) -> Result<QueueMsg<T>, QueueError>;
}

impl<T: QueueMessageTypes> HandleFetch<T> for Never {
    async fn handle(self, _: &<T as QueueMessageTypes>::Store) -> Result<QueueMsg<T>, QueueError> {
        match self {}
    }
}

impl<T: QueueMessageTypes> HandleWait<T> for Never {
    async fn handle(self, _: &<T as QueueMessageTypes>::Store) -> Result<QueueMsg<T>, QueueError> {
        match self {}
    }
}

impl<T: QueueMessageTypes> HandleEvent<T> for Never {
    fn handle(self, _: &<T as QueueMessageTypes>::Store) -> Result<QueueMsg<T>, QueueError> {
        match self {}
    }
}

impl<T: QueueMessageTypes> HandleEffect<T> for Never {
    async fn handle(self, _: &<T as QueueMessageTypes>::Store) -> Result<QueueMsg<T>, QueueError> {
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

#[derive(DebugNoBound, CloneNoBound)]
pub struct Engine<T: QueueMessageTypes> {
    store: Arc<T::Store>,
}

pub type BoxDynError = Box<dyn Error + Send + Sync + 'static>;

impl<T: QueueMessageTypes> Engine<T> {
    pub fn new(store: Arc<T::Store>) -> Self {
        Self { store }
    }

    pub fn run<'a, Q, O>(
        &'a self,
        q: &'a Q,
        o: &'a O,
    ) -> impl Stream<Item = Result<T::Data, BoxDynError>> + Send + 'a
    where
        Q: Queue<T>,
        O: PurePass<T>,
    {
        try_unfold::<_, _, _, Option<T::Data>>((), move |()| async move {
            debug!("stepping");

            // dbg!(&q);

            // sleep(Duration::from_secs(1)).await;

            self.step(q, o)
                .await
                .map(|step_result| step_result.map(|maybe_data| (maybe_data, ())))
        })
        .flat_map(|x| stream::iter(x.transpose()))
    }

    pub(crate) async fn step<'a, Q: Queue<T>, O: PurePass<T>>(
        &'a self,
        q: &'a Q,
        o: &'a O,
    ) -> Result<Option<Option<T::Data>>, BoxDynError> {
        // yield back to the runtime
        futures::future::ready(()).await;

        let s = self.store.clone();

        let data = q
            .process::<_, _, Option<T::Data>, O>(o, move |msg| async move {
                match msg.handle(&*s, 0).await {
                    // TODO: Make this an optimization pass
                    Ok(Some(QueueMsg::Data(d))) => {
                        let data_output = d.clone().handle(&s).unwrap();

                        // run to a fixed point
                        if data_output != data(d.clone()) {
                            (None, Ok(vec![data_output]))
                        } else {
                            (Some(d), Ok(vec![]))
                        }
                    }
                    Ok(msg) => (None, Ok(msg.into_iter().collect())),
                    Err(QueueError::Fatal(fatal)) => {
                        error!(error = %fatal.to_string(), "unrecoverable error");
                        (None, Err(fatal.to_string()))
                    }
                    Err(QueueError::Retry(retry)) => panic!("{retry:?}"),
                }
            })
            .await;

        match data {
            Ok(data) => Ok(Some(data.flatten())),
            Err(err) => Err(err.into()),
        }
    }
}

pub async fn run_to_completion<
    A: UseAggregate<T, R>,
    T: QueueMessageTypes,
    R,
    Q: Queue<T>,
    PrePass: PurePass<T>,
    PostPass: Pass<T>,
>(
    a: A,
    store: Arc<T::Store>,
    queue_config: Q::Config,
    msgs: impl IntoIterator<Item = QueueMsg<T>>,
    pre_pass_optimizer: PrePass,
    post_pass_optimizer: PostPass,
) -> R {
    let queue = Q::new(queue_config).await.unwrap();

    for msg in msgs {
        queue.enqueue(msg, &pre_pass_optimizer).await.unwrap();
    }

    debug!("spawning optimizer");

    let opt_fut = tokio::spawn({
        let queue = queue.clone();

        async move {
            loop {
                info!("optimizing");

                let res = queue.optimize(&post_pass_optimizer).await.map_err(|e| {
                    e.map_either::<_, _, BoxDynError, BoxDynError>(|x| Box::new(x), |x| Box::new(x))
                        .into_inner()
                });

                sleep(Duration::from_millis(100)).await;

                match res {
                    Ok(()) => {}
                    Err(err) => break Err::<(), _>(err),
                }
            }
        }
    });

    debug!("running");

    let output = Engine::new(store)
        .run(&queue, &pre_pass_optimizer)
        .take(A::AggregatedData::LEN)
        .try_collect()
        .await
        .unwrap();

    opt_fut.abort();

    let data = match HListTryFromIterator::try_from_iter(output) {
        Ok(ok) => ok,
        Err(_) => {
            panic!(
                "could not aggregate data into {}",
                std::any::type_name::<A>()
            )
        }
    };

    // dbg!(queue);

    A::aggregate(a, data)
}

#[derive(DebugNoBound, CloneNoBound)]
pub struct InMemoryQueue<T: QueueMessageTypes> {
    idx: Arc<AtomicU32>,
    ready: Arc<Mutex<BTreeMap<u32, Item<T>>>>,
    done: Arc<Mutex<BTreeMap<u32, Item<T>>>>,
    optimizer_queue: Arc<Mutex<BTreeMap<u32, Item<T>>>>,
}

#[derive(DebugNoBound, CloneNoBound)]
struct Item<T: QueueMessageTypes> {
    #[allow(dead_code)] // used in debug
    parents: Vec<u32>,
    msg: QueueMsg<T>,
}

impl<T: QueueMessageTypes> Queue<T> for InMemoryQueue<T> {
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
        item: QueueMsg<T>,
        pre_enqueue_passes: &'a O,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'a {
        let this = &self;
        let item = item;
        debug!(?item, "enqueueing new item");

        let res = pre_enqueue_passes.run_pass_pure(vec![item]);

        let mut optimizer_queue = this.optimizer_queue.lock().expect("mutex is poisoned");
        let mut ready = this.ready.lock().expect("mutex is poisoned");

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
        F: (FnOnce(QueueMsg<T>) -> Fut) + Send + 'static,
        Fut: Future<Output = (R, Result<Vec<QueueMsg<T>>, String>)> + Send + 'static,
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
                        let res = pre_reenqueue_passes.run_pass_pure(new_msgs);

                        let mut optimizer_queue =
                            self.optimizer_queue.lock().expect("mutex is poisoned");
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

                        Ok(Some(r))
                    }
                    Err(why) => panic!("{why}"),
                }
            }
            None => {
                trace!("queue is empty, sleeping for 1 second");

                sleep(Duration::from_secs(1)).await;

                Ok(None)
            }
        }
    }

    async fn optimize<'a, O: Pass<T>>(
        &'a self,
        optimizer: &'a O,
    ) -> Result<(), Either<Self::Error, O::Error>> {
        let msgs = {
            let lock = self.optimizer_queue.lock().unwrap();
            let msgs = lock.clone();
            drop(lock);
            msgs
        };

        let (ids, msgs): (Vec<_>, Vec<_>) = msgs.into_iter().unzip();

        let res = optimizer
            .run_pass(msgs.into_iter().map(|item| item.msg).collect())
            .await
            .map_err(Either::Right)?;

        // dbg!(&res, std::any::type_name::<O>());

        let mut ready = self.ready.lock().unwrap();
        let mut optimizer_queue = self.optimizer_queue.lock().unwrap();
        let mut done = self.done.lock().unwrap();

        done.append(&mut optimizer_queue);

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

impl<T: QueueMessageTypes> InMemoryQueue<T> {
    fn requeue(
        &self,
        res: OptimizationResult<T>,
        optimizer_queue: &mut BTreeMap<u32, Item<T>>,
        ready: &mut BTreeMap<u32, Item<T>>,
        get_parent_ids: impl Fn(Vec<usize>) -> Vec<u32>,
    ) {
        for (parent_idxs, new_msg) in res.optimize_further {
            optimizer_queue.insert(
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
    use queue_msg_macro::queue_msg;

    use crate::{
        aggregation::{do_aggregate, UseAggregate},
        data, effect, noop, HandleAggregate, HandleData, HandleEffect, HandleEvent, HandleFetch,
        HandleWait, QueueError, QueueMessageTypes, QueueMsg,
    };

    pub enum SimpleMessage {}

    impl QueueMessageTypes for SimpleMessage {
        type Event = SimpleEvent;
        type Data = SimpleData;
        type Fetch = SimpleFetch;
        type Effect = SimpleEffect;
        type Wait = SimpleWait;

        type Aggregate = SimpleAggregate;

        type Store = ();
    }

    impl HandleEffect<SimpleMessage> for SimpleEffect {
        async fn handle(self, _: &()) -> Result<QueueMsg<SimpleMessage>, QueueError> {
            Ok(noop())
        }
    }

    impl HandleEvent<SimpleMessage> for SimpleEvent {
        fn handle(self, _: &()) -> Result<QueueMsg<SimpleMessage>, QueueError> {
            Ok(noop())
        }
    }

    impl HandleData<SimpleMessage> for SimpleData {
        fn handle(self, _: &()) -> Result<QueueMsg<SimpleMessage>, QueueError> {
            Ok(data(self))
        }
    }

    impl HandleFetch<SimpleMessage> for SimpleFetch {
        async fn handle(self, _: &()) -> Result<QueueMsg<SimpleMessage>, QueueError> {
            Ok(match self {
                SimpleFetch::A(FetchA {}) => data(DataA {}),
                SimpleFetch::B(FetchB {}) => data(DataB {}),
                SimpleFetch::C(FetchC {}) => data(DataC {}),
                SimpleFetch::D(FetchD {}) => data(DataD {}),
                SimpleFetch::E(FetchE {}) => data(DataE {}),
            })
        }
    }

    impl HandleWait<SimpleMessage> for SimpleWait {
        async fn handle(self, _: &()) -> Result<QueueMsg<SimpleMessage>, QueueError> {
            Ok(noop())
        }
    }

    impl HandleAggregate<SimpleMessage> for SimpleAggregate {
        fn handle(self, data: VecDeque<SimpleData>) -> Result<QueueMsg<SimpleMessage>, QueueError> {
            Ok(match self {
                Self::AggregatePrintAbc(agg) => do_aggregate(agg, data),
            })
        }
    }

    #[queue_msg]
    pub struct SimpleEvent {}

    #[queue_msg]
    #[derive(Enumorph)]
    pub enum SimpleData {
        A(DataA),
        B(DataB),
        C(DataC),
        D(DataD),
        E(DataE),
    }
    #[queue_msg]
    pub struct DataA {}
    #[queue_msg]
    pub struct DataB {}
    #[queue_msg]
    pub struct DataC {}
    #[queue_msg]
    pub struct DataD {}
    #[queue_msg]
    pub struct DataE {}

    #[queue_msg]
    #[derive(Enumorph)]
    pub enum SimpleFetch {
        A(FetchA),
        B(FetchB),
        C(FetchC),
        D(FetchD),
        E(FetchE),
    }
    #[queue_msg]
    pub struct FetchA {}
    #[queue_msg]
    pub struct FetchB {}
    #[queue_msg]
    pub struct FetchC {}
    #[queue_msg]
    pub struct FetchD {}
    #[queue_msg]
    pub struct FetchE {}

    #[queue_msg]
    #[derive(Enumorph)]
    pub enum SimpleEffect {
        PrintAbc(PrintAbc),
    }

    #[queue_msg]
    pub struct PrintAbc {
        pub a: DataA,
        pub b: DataB,
        pub c: DataC,
    }

    #[queue_msg]
    pub struct SimpleWait {}

    #[queue_msg]
    #[derive(Enumorph)]
    pub enum SimpleAggregate {
        AggregatePrintAbc(AggregatePrintAbc),
    }

    #[queue_msg]
    pub struct AggregatePrintAbc {}

    impl UseAggregate<SimpleMessage> for AggregatePrintAbc {
        type AggregatedData = HList![DataA, DataB, DataC];

        fn aggregate(
            AggregatePrintAbc {}: Self,
            hlist_pat![a, b, c]: Self::AggregatedData,
        ) -> QueueMsg<SimpleMessage> {
            effect(PrintAbc { a, b, c })
        }
    }
}
