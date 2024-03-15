#![feature(trait_alias, extract_if)]

use std::{
    collections::VecDeque,
    error::Error,
    fmt::{Debug, Display},
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use futures::{
    stream::{self, try_unfold},
    Stream, StreamExt, TryStreamExt,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
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

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
#[serde(
    bound(serialize = "", deserialize = ""),
    tag = "@type",
    content = "@value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "T: QueueMsgTypes")
)]
pub enum QueueMsg<T: QueueMsgTypes> {
    /// An external event. This could be something like an IBC event, an external command, or anything else that occurs outside of the state machine. Can also be thought of as an "entry point".
    Event(T::Event),
    /// Inert data that will either be used in an aggregation or bubbled up to the top and sent as an output.
    Data(T::Data),
    /// Fetch some data from the outside world. This can also be thought of as a "read" operation.
    Fetch(T::Fetch),
    /// Send a message to the outside world. This can also be thought of as a "write" operation.
    Msg(T::Msg),
    /// Wait for some external condition.
    Wait(T::Wait),

    DeferUntil {
        point: DeferPoint,
        seconds: u64,
    },
    Repeat {
        times: u64,
        msg: Box<Self>,
    },
    Timeout {
        timeout_timestamp: u64,
        msg: Box<Self>,
    },
    /// A sequence of messages to be executed in order. Messages are handled from the top down, with new messages requeued on top:
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
    /// Note that this is similar to Sequence, but the new messages are queued at the *back* of the list, allowing for uniform progress across all nested messages.
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
    Noop,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum DeferPoint {
    Absolute,
    Relative,
}

#[inline]
pub fn retry<T: QueueMsgTypes>(count: u8, t: impl Into<QueueMsg<T>>) -> QueueMsg<T> {
    QueueMsg::Retry {
        remaining: count,
        msg: Box::new(t.into()),
    }
}

#[inline]
pub fn repeat<T: QueueMsgTypes>(times: u64, t: impl Into<QueueMsg<T>>) -> QueueMsg<T> {
    QueueMsg::Repeat {
        times,
        msg: Box::new(t.into()),
    }
}

#[inline]
pub fn seq<T: QueueMsgTypes>(ts: impl IntoIterator<Item = QueueMsg<T>>) -> QueueMsg<T> {
    QueueMsg::Sequence(ts.into_iter().collect())
}

#[inline]
pub fn conc<T: QueueMsgTypes>(ts: impl IntoIterator<Item = QueueMsg<T>>) -> QueueMsg<T> {
    QueueMsg::Concurrent(ts.into_iter().collect())
}

#[inline]
pub fn defer<T: QueueMsgTypes>(timestamp: u64) -> QueueMsg<T> {
    QueueMsg::DeferUntil {
        point: DeferPoint::Absolute,
        seconds: timestamp,
    }
}

#[inline]
pub fn defer_relative<T: QueueMsgTypes>(seconds: u64) -> QueueMsg<T> {
    QueueMsg::DeferUntil {
        point: DeferPoint::Relative,
        seconds,
    }
}

#[inline]
pub fn fetch<T: QueueMsgTypes>(t: impl Into<T::Fetch>) -> QueueMsg<T> {
    QueueMsg::Fetch(t.into())
}

#[inline]
pub fn msg<T: QueueMsgTypes>(t: impl Into<T::Msg>) -> QueueMsg<T> {
    QueueMsg::Msg(t.into())
}

#[inline]
pub fn data<T: QueueMsgTypes>(t: impl Into<T::Data>) -> QueueMsg<T> {
    QueueMsg::Data(t.into())
}

#[inline]
pub fn wait<T: QueueMsgTypes>(t: impl Into<T::Wait>) -> QueueMsg<T> {
    QueueMsg::Wait(t.into())
}

#[inline]
pub fn event<T: QueueMsgTypes>(t: impl Into<T::Event>) -> QueueMsg<T> {
    QueueMsg::Event(t.into())
}

#[inline]
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

pub trait QueueMsgTypesTraits = Debug
    + Display
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
    type Msg: HandleMsg<Self> + QueueMsgTypesTraits;
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
    ) -> Pin<Box<dyn Future<Output = Result<Option<QueueMsg<T>>, BoxDynError>> + Send + 'a>> {
        tracing::debug!(
            depth,
            %self,
            "handling message",
        );

        let fut = async move {
            match self {
                QueueMsg::Event(event) => Ok(Some(event.handle(store))),
                QueueMsg::Data(data) => Ok(Some(data.handle(store))),
                QueueMsg::Fetch(fetch) => Ok(Some(fetch.handle(store).await)),
                QueueMsg::Msg(msg) => {
                    msg.handle(store).await?;

                    Ok(None)
                }
                QueueMsg::Wait(wait) => Ok(Some(wait.handle(store).await)),

                QueueMsg::DeferUntil {
                    point: DeferPoint::Relative,
                    seconds,
                } => Ok(Some(defer(now() + seconds))),
                QueueMsg::DeferUntil { seconds, .. } => {
                    // if we haven't hit the time yet, requeue the defer msg
                    if now() < seconds {
                        // TODO: Make the time configurable?
                        tokio::time::sleep(Duration::from_secs(1)).await;

                        Ok(Some(defer(seconds)))
                    } else {
                        Ok(None)
                    }
                }
                QueueMsg::Timeout {
                    timeout_timestamp,
                    msg,
                } => {
                    // if we haven't hit the timeout yet, handle the msg
                    if now() > timeout_timestamp {
                        tracing::warn!(json = %serde_json::to_string(&msg).unwrap(), "message expired");

                        Ok(None)
                    } else {
                        msg.handle(store, depth + 1).await
                    }
                }
                QueueMsg::Sequence(mut queue) => match queue.pop_front() {
                    Some(msg) => {
                        let msg = msg.handle(store, depth + 1).await?;

                        if let Some(msg) = msg {
                            queue.push_front(msg)
                        }

                        Ok(Some(flatten_seq(seq(queue))))
                    }
                    None => Ok(None),
                },
                QueueMsg::Concurrent(mut queue) => match queue.pop_front() {
                    Some(msg) => {
                        let msg = msg.handle(store, depth + 1).await?;

                        if let Some(msg) = msg {
                            queue.push_back(msg)
                        }

                        Ok(Some(flatten_conc(conc(queue))))
                    }
                    None => Ok(None),
                },
                QueueMsg::Retry { remaining, msg } => {
                    const RETRY_DELAY_SECONDS: u64 = 3;

                    match msg.clone().handle(store, depth + 1).await {
                        Ok(ok) => Ok(ok),
                        Err(err) => {
                            if remaining > 0 {
                                let retries_left = remaining - 1;

                                tracing::warn!(
                                    %msg,
                                    retries_left,
                                    ?err,
                                    "msg failed, retrying in {RETRY_DELAY_SECONDS} seconds"
                                );

                                Ok(Some(seq([
                                    defer(now() + RETRY_DELAY_SECONDS),
                                    retry(retries_left, *msg),
                                ])))
                            } else {
                                tracing::error!(%msg, "msg failed after all retries");
                                Err(err)
                            }
                        }
                    }
                }
                QueueMsg::Aggregate {
                    mut queue,
                    mut data,
                    receiver,
                } => {
                    if let Some(msg) = queue.pop_front() {
                        let msg = msg.handle(store, depth + 1).await?;

                        if let Some(msg) = msg {
                            match msg {
                                QueueMsg::Data(d) => {
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
                        Ok(Some(receiver.handle(data)))
                    }
                }
                QueueMsg::Repeat { times: 0, .. } => Ok(None),
                QueueMsg::Repeat { times, msg } => Ok(Some(flatten_seq(seq([
                    *msg.clone(),
                    QueueMsg::Repeat {
                        times: times - 1,
                        msg,
                    },
                ])))),
                QueueMsg::Noop => Ok(None),
            }
        };

        Box::pin(fut)
    }
}

impl<T: QueueMsgTypes> std::fmt::Display for QueueMsg<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn display_list<I>(f: &mut std::fmt::Formatter<'_>, iter: I) -> Result<(), std::fmt::Error>
        where
            I: ExactSizeIterator,
            I::Item: Display,
        {
            let len = iter.len();

            write!(f, "[")?;

            for (idx, msg) in iter.enumerate() {
                write!(f, "{msg}")?;
                if idx != len - 1 {
                    write!(f, ", ")?;
                }
            }

            write!(f, "]")?;

            Ok(())
        }

        match self {
            QueueMsg::Event(event) => write!(f, "Event({event})"),
            QueueMsg::Data(data) => write!(f, "Data({data})"),
            QueueMsg::Fetch(fetch) => write!(f, "Fetch({fetch})"),
            QueueMsg::Msg(msg) => write!(f, "Msg({msg})"),
            QueueMsg::Wait(wait) => write!(f, "Wait({wait})"),
            QueueMsg::DeferUntil { point, seconds } => {
                write!(f, "DeferUntil({:?}, {seconds})", point)
            }
            QueueMsg::Repeat { times, msg } => write!(f, "Repeat({times}, {msg})"),
            QueueMsg::Timeout {
                timeout_timestamp,
                msg,
            } => write!(f, "Timeout({timeout_timestamp}, {msg})"),
            QueueMsg::Sequence(queue) => {
                write!(f, "Sequence")?;
                display_list(f, queue.iter())
            }
            QueueMsg::Concurrent(msgs) => {
                write!(f, "Parallel")?;
                display_list(f, msgs.iter())
            }
            QueueMsg::Retry { remaining, msg } => write!(f, "Retry({remaining}, {msg})"),
            QueueMsg::Aggregate {
                queue,
                data,
                receiver,
            } => {
                write!(f, "Aggregate(",)?;
                display_list(f, queue.iter())?;
                write!(f, " -> ",)?;
                display_list(f, data.iter())?;
                write!(f, " -> {receiver})",)
            }
            QueueMsg::Noop => {
                write!(f, "Noop")
            }
        }
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

#[test]
fn flatten() {
    struct EmptyMsgTypes;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
    struct Unit;

    impl Display for Unit {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("Unit")
        }
    }

    impl HandleMsg<EmptyMsgTypes> for Unit {
        async fn handle(self, _: &()) -> Result<(), BoxDynError> {
            Ok(())
        }
    }

    impl HandleEvent<EmptyMsgTypes> for Unit {
        fn handle(self, _: &()) -> QueueMsg<EmptyMsgTypes> {
            QueueMsg::Noop
        }
    }

    impl HandleData<EmptyMsgTypes> for Unit {
        fn handle(self, _: &()) -> QueueMsg<EmptyMsgTypes> {
            QueueMsg::Noop
        }
    }

    impl HandleFetch<EmptyMsgTypes> for Unit {
        async fn handle(self, _: &()) -> QueueMsg<EmptyMsgTypes> {
            QueueMsg::Noop
        }
    }

    impl HandleWait<EmptyMsgTypes> for Unit {
        async fn handle(self, _: &()) -> QueueMsg<EmptyMsgTypes> {
            QueueMsg::Noop
        }
    }

    impl HandleAggregate<EmptyMsgTypes> for Unit {
        fn handle(self, _: VecDeque<Unit>) -> QueueMsg<EmptyMsgTypes> {
            QueueMsg::Noop
        }
    }

    impl QueueMsgTypes for EmptyMsgTypes {
        type Event = Unit;
        type Data = Unit;
        type Fetch = Unit;
        type Msg = Unit;
        type Wait = Unit;

        type Aggregate = Unit;

        type Store = ();
    }

    let msg = seq::<EmptyMsgTypes>([
        defer(1),
        seq([defer(2), seq([defer(3)])]),
        seq([defer(4)]),
        defer(5),
    ]);
    assert_eq!(
        flatten_seq(msg),
        seq([defer(1), defer(2), defer(3), defer(4), defer(5)])
    );

    let msg = seq::<EmptyMsgTypes>([defer(1)]);
    assert_eq!(flatten_seq(msg), defer(1));

    let msg = conc::<EmptyMsgTypes>([defer(1)]);
    assert_eq!(flatten_seq(msg), conc([defer(1)]));

    let msg = conc::<EmptyMsgTypes>([seq([defer(1)])]);
    assert_eq!(flatten_seq(msg), conc([seq([defer(1)])]));

    let msg = seq::<EmptyMsgTypes>([data(Unit)]);
    assert_eq!(flatten_seq(msg), data(Unit));

    let msg = conc::<EmptyMsgTypes>([seq([data(Unit)])]);
    assert_eq!(flatten_seq(msg), conc([seq([data(Unit)])]));

    let msg = conc::<EmptyMsgTypes>([conc([conc([data(Unit)])])]);
    assert_eq!(flatten_conc(msg), data(Unit));
}

pub trait HandleFetch<T: QueueMsgTypes> {
    fn handle(self, store: &T::Store) -> impl Future<Output = QueueMsg<T>> + Send;
}

pub trait HandleData<T: QueueMsgTypes> {
    fn handle(self, store: &T::Store) -> QueueMsg<T>;
}

pub trait HandleWait<T: QueueMsgTypes> {
    fn handle(self, store: &T::Store) -> impl Future<Output = QueueMsg<T>> + Send;
}

pub trait HandleEvent<T: QueueMsgTypes> {
    fn handle(self, store: &T::Store) -> QueueMsg<T>;
}

pub trait HandleMsg<T: QueueMsgTypes> {
    fn handle(self, store: &T::Store) -> impl Future<Output = Result<(), BoxDynError>> + Send;
}

pub trait HandleAggregate<T: QueueMsgTypes> {
    fn handle(self, data: VecDeque<T::Data>) -> QueueMsg<T>;
}

impl<T: QueueMsgTypes> HandleFetch<T> for Never {
    async fn handle(self, _: &<T as QueueMsgTypes>::Store) -> QueueMsg<T> {
        match self {}
    }
}

impl<T: QueueMsgTypes> HandleWait<T> for Never {
    async fn handle(self, _: &<T as QueueMsgTypes>::Store) -> QueueMsg<T> {
        match self {}
    }
}

impl<T: QueueMsgTypes> HandleEvent<T> for Never {
    fn handle(self, _: &<T as QueueMsgTypes>::Store) -> QueueMsg<T> {
        match self {}
    }
}

impl<T: QueueMsgTypes> HandleMsg<T> for Never {
    async fn handle(self, _: &<T as QueueMsgTypes>::Store) -> Result<(), BoxDynError> {
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
pub struct Reactor<T: QueueMsgTypes> {
    store: Arc<T::Store>,
}

pub type BoxDynError = Box<dyn Error + Send + Sync + 'static>;

#[allow(clippy::manual_async_fn)] // please leave me alone
impl<T: QueueMsgTypes> Reactor<T> {
    pub fn new(store: Arc<T::Store>) -> Self {
        Self { store }
    }

    pub fn run<Q>(self, q: Q) -> impl Stream<Item = Result<T::Data, BoxDynError>> + Send + 'static
    where
        Q: Queue<T>,
    {
        fn unfold<T: QueueMsgTypes, Q: Queue<T>>(
            (store, mut q): (Arc<T::Store>, Q),
        ) -> impl Future<Output = Result<Option<(Vec<T::Data>, (Arc<T::Store>, Q))>, BoxDynError>>
               + Send
               + 'static {
            async move {
                // yield back to the runtime
                futures::future::ready(()).await;

                let s = store.clone();

                let data = q
                    .process::<_, _, Vec<T::Data>>(move |msg| async move {
                        match msg.handle(&*s, 0).await.unwrap() {
                            Some(QueueMsg::Data(d)) => (vec![d], Ok(vec![])),
                            Some(QueueMsg::Concurrent(msgs)) => (vec![], Ok(msgs.into())),
                            msg => (vec![], Ok(msg.into_iter().collect())),
                        }
                    })
                    .await;

                match data {
                    Ok(data) => Ok(Some((data.into_iter().flatten().collect(), (store, q)))),
                    Err(err) => Err(err.into()),
                }
            }
        }

        try_unfold::<_, _, _, Vec<T::Data>>((self.store, q), unfold).flat_map(|x| {
            stream::iter(x.map_or_else(|err| vec![Err(err)], |ok| ok.into_iter().map(Ok).collect()))
        })
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

    let output = Reactor::new(store)
        .run(queue)
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

// impl<T: QueueMsgTypes, Q: Queue<T>> Sink<T> for Q {
//     type Error;

//     fn poll_ready(
//         self: Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Result<(), Self::Error>> {
//         todo!()
//     }

//     fn start_send(self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
//         todo!()
//     }

//     fn poll_flush(
//         self: Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Result<(), Self::Error>> {
//         todo!()
//     }

//     fn poll_close(
//         self: Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Result<(), Self::Error>> {
//         todo!()
//     }
// }

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
        tracing::warn!(%item, "enqueueing new item");
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

#[macro_export]
macro_rules! msg_struct {
    (
        $(#[cover($($CoverTy:ty),+)])?
        pub struct $Struct:ident$(<$($generics:ident: $bound:ident$(<$($boundT:ident),+>)?),+>)? {
            $(
                pub $field:ident: $FieldTy:ty,
            )*
        }
    ) => {
        #[derive(Serialize, Deserialize)]
        #[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
        #[cfg_attr(
            feature = "arbitrary",
            derive(arbitrary::Arbitrary),
            arbitrary(bound = "")
        )]
        pub struct $Struct$(<$($generics: $bound$(<$($boundT),+>)?),+>)? {
            $(
                pub $field: $FieldTy,
            )*
            $(
                #[serde(skip)]
                #[cfg_attr(feature = "arbitrary", arbitrary(default))]
                pub __marker: PhantomData<fn() -> ($($CoverTy,)+)>,
            )?
        }
        const _: () = {
            impl$(<$($generics: $bound$(<$($boundT),+>)?),+>)? ::core::fmt::Debug
            for $Struct$(<$($generics),+>)? {
                fn fmt(&self, fmt: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    fmt.debug_struct(stringify!($Struct))
                        $(.field(stringify!($field), &self.$field))*
                        .finish()
                }
            }

            impl$(<$($generics: $bound$(<$($boundT),+>)?),+>)? ::core::clone::Clone
            for $Struct$(<$($generics),+>)? {
                fn clone(&self) -> Self {
                    Self {
                        $($field: ::core::clone::Clone::clone(&self.$field),)*
                        $(
                            __marker: PhantomData::<fn() -> ($($CoverTy,)+)>,
                        )?
                    }
                }
            }

            impl$(<$($generics: $bound$(<$($boundT),+>)?),+>)? ::core::cmp::PartialEq
            for $Struct $(<$($generics),+>)? {
                fn eq(&self, other: &Self) -> bool {
                    let _other = other;
                    true $(&& self.$field == _other.$field)*
                }
            }
        };
    };

    (
        $(#[cover($($CoverTy:ty),+)])?
        pub struct $Struct:ident<$($generics:ident: $bound:ident$(<$($boundT:ident),+>)?),+>(pub $FieldTy:ty$(,)?);
    ) => {
        #[derive(Serialize, Deserialize)]
        #[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields, transparent)]
        #[cfg_attr(
            feature = "arbitrary",
            derive(arbitrary::Arbitrary),
            arbitrary(bound = "")
        )]
        pub struct $Struct<$($generics: $bound$(<$($boundT),+>)?),+> (pub $FieldTy);
        const _: () = {
            impl<$($generics: $bound$(<$($boundT),+>)?),+> ::core::fmt::Debug
            for $Struct<$($generics),+> {
                fn fmt(&self, fmt: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    fmt.debug_tuple(stringify!($Struct))
                        .field(&self.0)
                        .finish()
                }
            }

            impl<$($generics: $bound$(<$($boundT),+>)?),+> ::core::clone::Clone
            for $Struct<$($generics),+> {
                fn clone(&self) -> Self {
                    Self(::core::clone::Clone::clone(&self.0))
                }
            }

            impl<$($generics: $bound$(<$($boundT),+>)?),+> ::core::cmp::PartialEq
            for $Struct<$($generics),+> {
                fn eq(&self, other: &Self) -> bool {
                    self.0 == other.0
                }
            }
        };

    };
}
