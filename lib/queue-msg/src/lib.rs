#![feature(trait_alias, extract_if)]

use std::{
    collections::VecDeque,
    error::Error,
    fmt::Debug,
    future::Future,
    num::NonZeroU64,
    pin::Pin,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use frame_support_procedural::{CloneNoBound, DebugNoBound};
use futures::{
    stream::{self, try_unfold},
    Stream, StreamExt, TryStreamExt,
};
pub use queue_msg_macro::queue_msg;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::time::sleep;
use unionlabs::{never::Never, MaybeArbitrary};

use crate::aggregation::{HListTryFromIterator, UseAggregate};

pub mod aggregation;

pub trait Queue<T: QueueMsgTypes>: Clone + Send + Sync + Sized + 'static {
    /// Error type returned by this queue, representing errors that are out of control of the consumer (i.e. unable to connect to database, can't insert into row, can't deserialize row, etc)
    type Error: Error + Send + Sync + 'static;
    type Config: Debug + Clone + Serialize + DeserializeOwned;

    fn new(cfg: Self::Config) -> impl Future<Output = Result<Self, Self::Error>>;

    fn enqueue(
        &mut self,
        item: QueueMsg<T>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + '_;

    fn process<F, Fut, R>(
        &mut self,
        f: F,
    ) -> impl Future<Output = Result<Option<R>, Self::Error>> + Send + '_
    where
        F: (FnOnce(QueueMsg<T>) -> Fut) + Send + 'static,
        Fut: Future<Output = (R, Result<Vec<QueueMsg<T>>, String>)> + Send + 'static,
        R: Send + Sync + 'static;
}

#[queue_msg]
#[debug(bound())]
pub enum QueueMsg<T: QueueMsgTypes> {
    /// An external event. This could be something like an IBC event, an external command, or anything else that occurs outside of the state machine. Can also be thought of as an "entry point".
    Event(T::Event),
    /// Inert data that will either be used in an aggregation or bubbled up to the top and sent as an output.
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
    /// A sequence of messages to be executed in order. Messages are handled from the front, with new messages requeued at the front:
    //
    /// ```txt
    /// [A B C]
    /// D = handle(A)
    /// [D B C]
    /// ```
    Sequence(VecDeque<Self>),
    /// A list of messages to be executed concurrently. If this is queued as a top-level message, each contained message will be requeued individually as a top-level message, however if it is nested within another message, it's semantics are as follows:
    ///
    /// ```txt
    /// [A B C]
    /// D = handle(A)
    /// [B C D]
    /// ```
    ///
    /// Note that this is similar to `Sequence`, but the new messages are queued at the *back* of the list, allowing for uniform progress across all nested messages.
    Concurrent(VecDeque<Self>),
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
    Void(Box<Self>),
    Noop,
}

#[queue_msg]
pub enum Defer {
    Absolute(u64),
    Relative(u64),
}

#[inline]
#[must_use = "constructing an instruction has no effect"]
pub fn retry<T: QueueMsgTypes>(count: u8, t: impl Into<QueueMsg<T>>) -> QueueMsg<T> {
    QueueMsg::Retry {
        remaining: count,
        msg: Box::new(t.into()),
    }
}

#[inline]
#[must_use = "constructing an instruction has no effect"]
pub fn repeat<T: QueueMsgTypes>(
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
pub fn seq<T: QueueMsgTypes>(ts: impl IntoIterator<Item = QueueMsg<T>>) -> QueueMsg<T> {
    QueueMsg::Sequence(ts.into_iter().collect())
}

#[inline]
#[must_use = "constructing an instruction has no effect"]
pub fn conc<T: QueueMsgTypes>(ts: impl IntoIterator<Item = QueueMsg<T>>) -> QueueMsg<T> {
    QueueMsg::Concurrent(ts.into_iter().collect())
}

#[inline]
#[must_use = "constructing an instruction has no effect"]
pub fn defer_absolute<T: QueueMsgTypes>(timestamp: u64) -> QueueMsg<T> {
    QueueMsg::Defer(Defer::Absolute(timestamp))
}

#[inline]
#[must_use = "constructing an instruction has no effect"]
pub fn defer_relative<T: QueueMsgTypes>(seconds: u64) -> QueueMsg<T> {
    QueueMsg::Defer(Defer::Relative(seconds))
}

#[inline]
#[must_use = "constructing an instruction has no effect"]
pub fn fetch<T: QueueMsgTypes>(t: impl Into<T::Fetch>) -> QueueMsg<T> {
    QueueMsg::Fetch(t.into())
}

#[inline]
#[must_use = "constructing an instruction has no effect"]
pub fn effect<T: QueueMsgTypes>(t: impl Into<T::Effect>) -> QueueMsg<T> {
    QueueMsg::Effect(t.into())
}

#[inline]
#[must_use = "constructing an instruction has no effect"]
pub fn data<T: QueueMsgTypes>(t: impl Into<T::Data>) -> QueueMsg<T> {
    QueueMsg::Data(t.into())
}

#[inline]
#[must_use = "constructing an instruction has no effect"]
pub fn wait<T: QueueMsgTypes>(t: impl Into<T::Wait>) -> QueueMsg<T> {
    QueueMsg::Wait(t.into())
}

#[inline]
#[must_use = "constructing an instruction has no effect"]
pub fn event<T: QueueMsgTypes>(t: impl Into<T::Event>) -> QueueMsg<T> {
    QueueMsg::Event(t.into())
}

#[inline]
#[must_use = "constructing an instruction has no effect"]
pub fn aggregate<T: QueueMsgTypes>(
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
pub fn void<T: QueueMsgTypes>(t: impl Into<QueueMsg<T>>) -> QueueMsg<T> {
    QueueMsg::Void(Box::new(t.into()))
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

pub trait QueueMsgTypes: Sized + 'static {
    type Event: HandleEvent<Self> + QueueMsgTypesTraits;
    type Data: HandleData<Self> + QueueMsgTypesTraits;
    type Fetch: HandleFetch<Self> + QueueMsgTypesTraits;
    type Effect: HandleEffect<Self> + QueueMsgTypesTraits;
    type Wait: HandleWait<Self> + QueueMsgTypesTraits;
    type Aggregate: HandleAggregate<Self> + QueueMsgTypesTraits;

    type Store: Debug + Send + Sync;
}

impl<T: QueueMsgTypes> QueueMsg<T> {
    // NOTE: Box is required bc recursion
    #[allow(clippy::type_complexity)]
    pub fn handle<'a>(
        self,
        store: &'a T::Store,
        depth: usize,
    ) -> Pin<Box<dyn Future<Output = Result<Option<QueueMsg<T>>, QueueError>> + Send + 'a>> {
        tracing::debug!(depth, "handling message");

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
                        tracing::warn!("message expired");

                        Ok(None)
                    } else {
                        // REVIEW: Should we handle the message here or return it to be handled on the next time it's popped from the queue?
                        msg.handle(store, depth + 1).await
                    }
                }
                Self::Sequence(mut queue) => match queue.pop_front() {
                    Some(msg) => {
                        let msg = msg.handle(store, depth + 1).await?;

                        if let Some(msg) = msg {
                            queue.push_front(msg)
                        }

                        Ok(Some(flatten_seq(seq(queue))))
                    }
                    None => Ok(None),
                },
                Self::Concurrent(mut queue) => match queue.pop_front() {
                    Some(msg) => {
                        let msg = msg.handle(store, depth + 1).await?;

                        if let Some(msg) = msg {
                            queue.push_back(msg)
                        }

                        Ok(Some(flatten_conc(conc(queue))))
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

                                tracing::warn!(
                                    retries_left,
                                    ?err,
                                    "msg failed, retrying in {RETRY_DELAY_SECONDS} seconds"
                                );

                                Ok(Some(seq([
                                    defer_absolute(now() + RETRY_DELAY_SECONDS),
                                    retry(retries_left, *msg),
                                ])))
                            } else {
                                tracing::error!("msg failed after all retries");
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
                    Ok(Some(flatten_seq(seq([*msg.clone(), repeat(None, *msg)]))))
                }
                Self::Repeat {
                    times: Some(times),
                    msg,
                } => Ok(Some(flatten_seq(seq([*msg.clone()].into_iter().chain(
                    // if times - 1 > 0, queue repeat with times - 1
                    NonZeroU64::new(times.get() - 1_u64).map(|times| repeat(Some(times), *msg)),
                ))))),
                QueueMsg::Void(msg) => {
                    // TODO: distribute across seq/conc
                    Ok(msg.handle(store, depth + 1).await?.map(|msg| match msg {
                        QueueMsg::Data(data) => {
                            tracing::debug!(data = %serde_json::to_string(&data).unwrap(), "voiding data");
                            QueueMsg::Noop
                        }
                        msg => void(msg),
                    }))
                }
                QueueMsg::Noop => Ok(None),
            }
        };

        Box::pin(fut)
    }
}

fn flatten_seq<T: QueueMsgTypes>(msg: QueueMsg<T>) -> QueueMsg<T> {
    fn flatten<T: QueueMsgTypes>(msg: QueueMsg<T>) -> Vec<QueueMsg<T>> {
        match msg {
            QueueMsg::Sequence(new_seq) => new_seq
                .into_iter()
                .flat_map(flatten)
                .filter(|x| !matches!(x, QueueMsg::Noop))
                .collect(),
            _ => [msg].into(),
        }
    }

    let mut msgs = flatten(msg);

    if msgs.len() == 1 {
        msgs.pop().unwrap()
    } else {
        let mut data = vec![];
        for d in msgs.extract_if(|msg| matches!(msg, QueueMsg::Data(_))) {
            data.push(d);
        }

        if data.is_empty() {
            seq(msgs)
        } else {
            conc(data.into_iter().chain([seq(msgs)]))
        }
    }
}

fn flatten_conc<T: QueueMsgTypes>(msg: QueueMsg<T>) -> QueueMsg<T> {
    fn flatten<T: QueueMsgTypes>(msg: QueueMsg<T>) -> Vec<QueueMsg<T>> {
        match msg {
            QueueMsg::Concurrent(new_conc) => new_conc
                .into_iter()
                .flat_map(flatten)
                .filter(|x| !matches!(x, QueueMsg::Noop))
                .collect(),
            _ => [msg].into(),
        }
    }

    let mut msgs = flatten(msg);

    if msgs.len() == 1 {
        msgs.pop().unwrap()
    } else {
        conc(msgs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flatten() {
        struct EmptyMsgTypes;

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        #[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
        struct Unit;

        impl HandleEffect<EmptyMsgTypes> for Unit {
            async fn handle(self, _: &()) -> Result<QueueMsg<EmptyMsgTypes>, QueueError> {
                Ok(QueueMsg::Noop)
            }
        }

        impl HandleEvent<EmptyMsgTypes> for Unit {
            fn handle(self, _: &()) -> Result<QueueMsg<EmptyMsgTypes>, QueueError> {
                Ok(QueueMsg::Noop)
            }
        }

        impl HandleData<EmptyMsgTypes> for Unit {
            fn handle(self, _: &()) -> Result<QueueMsg<EmptyMsgTypes>, QueueError> {
                Ok(QueueMsg::Noop)
            }
        }

        impl HandleFetch<EmptyMsgTypes> for Unit {
            async fn handle(self, _: &()) -> Result<QueueMsg<EmptyMsgTypes>, QueueError> {
                Ok(QueueMsg::Noop)
            }
        }

        impl HandleWait<EmptyMsgTypes> for Unit {
            async fn handle(self, _: &()) -> Result<QueueMsg<EmptyMsgTypes>, QueueError> {
                Ok(QueueMsg::Noop)
            }
        }

        impl HandleAggregate<EmptyMsgTypes> for Unit {
            fn handle(self, _: VecDeque<Unit>) -> Result<QueueMsg<EmptyMsgTypes>, QueueError> {
                Ok(QueueMsg::Noop)
            }
        }

        impl QueueMsgTypes for EmptyMsgTypes {
            type Event = Unit;
            type Data = Unit;
            type Fetch = Unit;
            type Effect = Unit;
            type Wait = Unit;

            type Aggregate = Unit;

            type Store = ();
        }

        let msg = seq::<EmptyMsgTypes>([
            defer_absolute(1),
            seq([defer_absolute(2), seq([defer_absolute(3)])]),
            seq([defer_absolute(4)]),
            defer_absolute(5),
        ]);
        assert_eq!(
            flatten_seq(msg),
            seq([
                defer_absolute(1),
                defer_absolute(2),
                defer_absolute(3),
                defer_absolute(4),
                defer_absolute(5)
            ])
        );

        let msg = seq::<EmptyMsgTypes>([defer_absolute(1)]);
        assert_eq!(flatten_seq(msg), defer_absolute(1));

        let msg = conc::<EmptyMsgTypes>([defer_absolute(1)]);
        assert_eq!(flatten_seq(msg), conc([defer_absolute(1)]));

        let msg = conc::<EmptyMsgTypes>([seq([defer_absolute(1)])]);
        assert_eq!(flatten_seq(msg), conc([seq([defer_absolute(1)])]));

        let msg = seq::<EmptyMsgTypes>([data(Unit)]);
        assert_eq!(flatten_seq(msg), data(Unit));

        let msg = conc::<EmptyMsgTypes>([seq([data(Unit)])]);
        assert_eq!(flatten_seq(msg), conc([seq([data(Unit)])]));

        let msg = conc::<EmptyMsgTypes>([conc([conc([data(Unit)])])]);
        assert_eq!(flatten_conc(msg), data(Unit));
    }
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

pub trait HandleFetch<T: QueueMsgTypes> {
    fn handle(
        self,
        store: &T::Store,
    ) -> impl Future<Output = Result<QueueMsg<T>, QueueError>> + Send;
}

pub trait HandleData<T: QueueMsgTypes> {
    fn handle(self, store: &T::Store) -> Result<QueueMsg<T>, QueueError>;
}

pub trait HandleWait<T: QueueMsgTypes> {
    fn handle(
        self,
        store: &T::Store,
    ) -> impl Future<Output = Result<QueueMsg<T>, QueueError>> + Send;
}

pub trait HandleEvent<T: QueueMsgTypes> {
    fn handle(self, store: &T::Store) -> Result<QueueMsg<T>, QueueError>;
}

pub trait HandleEffect<T: QueueMsgTypes> {
    fn handle(
        self,
        store: &T::Store,
    ) -> impl Future<Output = Result<QueueMsg<T>, QueueError>> + Send;
}

pub trait HandleAggregate<T: QueueMsgTypes> {
    fn handle(self, data: VecDeque<T::Data>) -> Result<QueueMsg<T>, QueueError>;
}

impl<T: QueueMsgTypes> HandleFetch<T> for Never {
    async fn handle(self, _: &<T as QueueMsgTypes>::Store) -> Result<QueueMsg<T>, QueueError> {
        match self {}
    }
}

impl<T: QueueMsgTypes> HandleWait<T> for Never {
    async fn handle(self, _: &<T as QueueMsgTypes>::Store) -> Result<QueueMsg<T>, QueueError> {
        match self {}
    }
}

impl<T: QueueMsgTypes> HandleEvent<T> for Never {
    fn handle(self, _: &<T as QueueMsgTypes>::Store) -> Result<QueueMsg<T>, QueueError> {
        match self {}
    }
}

impl<T: QueueMsgTypes> HandleEffect<T> for Never {
    async fn handle(self, _: &<T as QueueMsgTypes>::Store) -> Result<QueueMsg<T>, QueueError> {
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
pub struct Engine<T: QueueMsgTypes> {
    store: Arc<T::Store>,
}

pub type BoxDynError = Box<dyn Error + Send + Sync + 'static>;

impl<T: QueueMsgTypes> Engine<T> {
    pub fn new(store: Arc<T::Store>) -> Self {
        Self { store }
    }

    pub fn run<'a, Q>(
        &'a self,
        q: &'a mut Q,
    ) -> impl Stream<Item = Result<T::Data, BoxDynError>> + Send + 'a
    where
        Q: Queue<T>,
    {
        try_unfold::<_, _, _, Option<T::Data>>(q, move |q| async move {
            self.step(q).await.map(|x| x.map(|x| (x, q)))
        })
        .flat_map(|x| stream::iter(x.transpose()))
    }

    pub(crate) async fn step<'a, Q: Queue<T>>(
        &'a self,
        q: &'a mut Q,
    ) -> Result<Option<Option<T::Data>>, BoxDynError> {
        // yield back to the runtime
        futures::future::ready(()).await;

        let s = self.store.clone();

        let data = q
            .process::<_, _, Option<T::Data>>(move |msg| async move {
                match msg.handle(&*s, 0).await {
                    Ok(Some(QueueMsg::Data(d))) => {
                        let data_output = d.clone().handle(&s).unwrap();

                        // run to a fixed point
                        if data_output != data(d.clone()) {
                            (None, Ok(vec![data_output]))
                        } else {
                            (Some(d), Ok(vec![]))
                        }
                    }
                    Ok(Some(QueueMsg::Concurrent(msgs))) => (None, Ok(msgs.into())),
                    Ok(msg) => (None, Ok(msg.into_iter().collect())),
                    Err(QueueError::Fatal(fatal)) => {
                        tracing::error!(error = %fatal.to_string(), "unrecoverable error");
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

pub async fn run_to_completion<A: UseAggregate<T, R>, T: QueueMsgTypes, R, Q: Queue<T>>(
    a: A,
    store: Arc<T::Store>,
    queue_config: Q::Config,
    msgs: impl IntoIterator<Item = QueueMsg<T>>,
) -> R {
    let mut queue = Q::new(queue_config).await.unwrap();

    for msg in msgs {
        queue.enqueue(msg).await.unwrap();
    }

    let output = Engine::new(store)
        .run(&mut queue)
        .take(A::AggregatedData::LEN)
        .try_collect()
        .await
        .unwrap();

    let data = match HListTryFromIterator::try_from_iter(output) {
        Ok(ok) => ok,
        Err(_) => {
            panic!(
                "could not aggregate data into {}",
                std::any::type_name::<A>()
            )
        }
    };

    A::aggregate(a, data)
}

#[derive(DebugNoBound, CloneNoBound)]
pub struct InMemoryQueue<T: QueueMsgTypes>(Arc<Mutex<VecDeque<QueueMsg<T>>>>);

impl<T: QueueMsgTypes> Queue<T> for InMemoryQueue<T> {
    type Error = std::convert::Infallible;
    type Config = ();

    fn new(_cfg: Self::Config) -> impl Future<Output = Result<Self, Self::Error>> {
        futures::future::ok(Self(Arc::new(Mutex::new(VecDeque::default()))))
    }

    fn enqueue(
        &mut self,
        item: QueueMsg<T>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + '_ {
        tracing::info!(?item, "enqueueing new item");
        self.0.lock().expect("mutex is poisoned").push_back(item);
        futures::future::ok(())
    }

    async fn process<F, Fut, R>(&mut self, f: F) -> Result<Option<R>, Self::Error>
    where
        F: (FnOnce(QueueMsg<T>) -> Fut) + Send + 'static,
        Fut: Future<Output = (R, Result<Vec<QueueMsg<T>>, String>)> + Send + 'static,
        R: Send + Sync + 'static,
    {
        let msg = {
            let mut queue = self.0.lock().expect("mutex is poisoned");
            let msg = queue.pop_front();

            drop(queue);

            msg
        };

        match msg {
            Some(msg) => {
                tracing::info!(
                    json = %serde_json::to_string(&msg).unwrap(),
                );

                let (r, res) = f(msg.clone()).await;
                match res {
                    Ok(new_msgs) => {
                        for new_msg in new_msgs {
                            let mut queue = self.0.lock().expect("mutex is poisoned");
                            queue.push_back(new_msg);
                        }

                        Ok(Some(r))
                    }
                    Err(why) => panic!("{why}"),
                }
            }
            None => Ok(None),
        }
    }
}
