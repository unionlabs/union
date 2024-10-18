#![feature(trait_alias)]
#![warn(clippy::pedantic, clippy::large_futures, clippy::large_stack_frames)]
#![allow(clippy::missing_panics_doc, clippy::module_name_repetitions)]
#![cfg_attr(not(test), warn(clippy::unwrap_used,))]

use std::{
    self,
    collections::VecDeque,
    error::Error,
    fmt::Debug,
    future::Future,
    pin::Pin,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use derive_where::derive_where;
use either::Either::{self, Left, Right};
use itertools::Itertools;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::time::sleep;
use tracing::{debug, error, info, trace, warn};
use unionlabs::never::Never;

use crate::{filter::InterestFilter, pass::Pass};

pub mod engine;
pub mod filter;
pub mod in_memory;
pub mod pass;

#[cfg(test)]
mod tests;

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
    fn enqueue<'a>(
        &'a self,
        item: Op<T>,
        filter: &'a T::Filter,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'a;

    /// Process the item at the front of the queue, if there is one. New items will be pre-processed by `O` before being reenqueued.
    ///
    /// All items will be enqueued to be optimized, unless marked as ready by `O`.
    fn process<'a, F, Fut, R>(
        &'a self,
        filter: &'a T::Filter,
        f: F,
    ) -> impl Future<Output = Result<Option<R>, Self::Error>> + Send + Captures<'a>
    where
        F: (FnOnce(Op<T>) -> Fut) + Send + Captures<'a>,
        Fut: Future<Output = (R, Result<Vec<Op<T>>, String>)> + Send + Captures<'a>,
        R: Send + Sync + 'static;

    fn optimize<'a, O: Pass<T>>(
        &'a self,
        tag: &'a str,
        optimizer: &'a O,
    ) -> impl Future<Output = Result<(), Either<Self::Error, O::Error>>> + Send + 'a;
}

#[derive_where(Debug, PartialEq, Clone)]
#[derive(Serialize, Deserialize)]
#[serde(
    tag = "@type",
    content = "@value",
    rename_all = "snake_case",
    bound(serialize = "", deserialize = ""),
    deny_unknown_fields
)]
pub enum Op<T: QueueMessage> {
    /// Inert data that will either be used in an [`Op::Promise`] or bubbled up to the top and sent as
    /// an output.
    Data(T::Data),
    /// Execute an action.
    Call(T::Call),
    Defer {
        until: u64,
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
    /// Note that this is similar to `Seq`, except that the new messages are queued at the
    /// *back* of the list, allowing for uniform progress across all nested messages.
    Conc(VecDeque<Self>),
    Promise(Promise<T>),
    /// Handle the contained message, voiding any returned `Data` messages that it returns.
    Void(Box<Self>),
    Noop,
}

#[derive_where(Debug, PartialEq, Clone)]
#[derive(Serialize, Deserialize)]
#[serde(
    rename_all = "snake_case",
    bound(serialize = "", deserialize = ""),
    deny_unknown_fields
)]
pub struct Promise<T: QueueMessage> {
    /// Messages that are expected to resolve to [`Op::Data`].
    pub queue: VecDeque<Op<T>>,
    /// The resolved data messages.
    pub data: VecDeque<T::Data>,
    /// The message that will utilize the aggregated data.
    pub receiver: T::Callback,
}

pub trait Visit<T: QueueMessage> {
    fn visit_op(&mut self, op: &mut Op<T>) {
        match op {
            Op::Data(data) => self.visit_data(data),
            Op::Call(call) => self.visit_call(call),
            Op::Defer { until: _ } | Op::Noop => {}
            Op::Seq(seq) => seq.iter_mut().for_each(|op| self.visit_op(op)),
            Op::Conc(conc) => conc.iter_mut().for_each(|op| self.visit_op(op)),
            Op::Promise(Promise {
                queue,
                data,
                receiver: _,
            }) => {
                queue.iter_mut().for_each(|op| self.visit_op(op));
                data.iter_mut().for_each(|data| self.visit_data(data));
            }
            Op::Void(op) => self.visit_op(op),
        }
    }

    fn visit_data(&mut self, data: &mut T::Data) {
        let _ = data;
    }

    fn visit_call(&mut self, call: &mut T::Call) {
        let _ = call;
    }
}

pub trait OpT =
    Debug + Clone + PartialEq + Serialize + for<'a> Deserialize<'a> + Send + Sync + Unpin;

pub trait QueueMessage: Sized + 'static {
    type Data: OpT;
    type Call: CallT<Self> + OpT;
    type Callback: CallbackT<Self> + OpT;

    type Filter: InterestFilter<Self>;

    type Context: Context;
}

pub trait Context: Send + Sync {}

impl Context for () {}

pub type BoxDynError = Box<dyn Error + Send + Sync + 'static>;

impl<T: QueueMessage> Op<T> {
    // NOTE: Box is required bc recursion
    #[allow(clippy::type_complexity)]
    pub fn process<'a>(
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
                        data = %serde_json::to_string(&data).expect("serialization is infallible; qed;"),
                        "received data outside of an aggregation"
                    );
                    Ok(None)
                }

                Op::Call(call) => call.process(store).await.map(Some),
                Op::Defer { until: seconds } => {
                    // if we haven't hit the time yet, requeue the defer op
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
                Op::Seq(mut queue) => match queue.pop_front() {
                    Some(op) => {
                        let op = op.process(store, depth + 1).await?;

                        if let Some(op) = op {
                            queue.push_front(op);
                        }

                        Ok(Some(seq(queue)))
                    }
                    None => Ok(None),
                },
                Op::Conc(mut queue) => match queue.pop_front() {
                    Some(op) => {
                        let op = op.process(store, depth + 1).await?;

                        if let Some(op) = op {
                            queue.push_back(op);
                        }

                        Ok(Some(conc(queue)))
                    }
                    None => Ok(None),
                },
                Op::Promise(Promise {
                    mut queue,
                    mut data,
                    receiver,
                }) => {
                    if let Some(op) = queue.pop_front() {
                        match op {
                            Op::Data(d) => {
                                data.push_back(d);
                            }
                            op => {
                                let op = op.process(store, depth + 1).await?;

                                if let Some(op) = op {
                                    match op {
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
                        // queue is empty, handle op
                        receiver.process(store, data).await.map(Some)
                    }
                }
                Op::Void(op) => {
                    // TODO: distribute across seq/conc
                    Ok(op.process(store, depth + 1).await?.map(|op| match op {
                        Op::Data(data) => {
                            debug!(
                                data = %serde_json::to_string(&data).expect("serialization is infallible; qed;"),
                                "voiding data"
                            );
                            noop()
                        }
                        op => void(op),
                    }))
                }
                Op::Noop => Ok(None),
            }
        };

        Box::pin(fut)
    }

    pub fn normalize(self) -> Vec<Op<T>> {
        pub fn go<T: QueueMessage>(op: Op<T>) -> Vec<Op<T>> {
            match op {
                Op::Data(data) => vec![Op::Data(data)],
                Op::Call(call) => vec![Op::Call(call)],
                Op::Defer { until } => vec![Op::Defer { until }],
                Op::Seq(seq) => {
                    let mut ops = seq.into_iter().flat_map(go).collect::<Vec<_>>();

                    let first_non_data_op_idx = ops
                        .iter()
                        .enumerate()
                        .find_map(|(idx, op)| (!matches!(op, Op::Data(_))).then_some(idx))
                        .unwrap_or(ops.len());

                    match ops.len() {
                        0 => vec![],
                        1 => vec![ops.pop().expect("length is 1; qed;")],
                        2.. => {
                            let non_data_ops = ops.split_off(first_non_data_op_idx);
                            let data_ops = ops;

                            if non_data_ops.is_empty() {
                                data_ops
                            } else {
                                data_ops
                                    .into_iter()
                                    .chain([Op::Seq(
                                        non_data_ops
                                            .into_iter()
                                            .flat_map(|op| match op {
                                                Op::Seq(seq) => seq.into(),
                                                op => vec![op],
                                            })
                                            .collect(),
                                    )])
                                    .collect()
                            }
                        }
                    }
                }
                Op::Conc(conc) => {
                    let (datas, mut ops): (Vec<_>, Vec<_>) = conc
                        .into_iter()
                        .flat_map(go)
                        .flat_map(|op| match op {
                            Op::Conc(seq) => seq.into(),
                            op => vec![op],
                        })
                        .partition_map(|op| match op {
                            Op::Data(data) => Left(data),
                            op => Right(op),
                        });

                    let ops = match ops.len() {
                        0 => vec![],
                        1 => vec![ops.pop().expect("length is 1; qed;")],
                        2.. => {
                            vec![Op::Conc(ops.into())]
                        }
                    };

                    datas.into_iter().map(Op::Data).chain(ops).collect()
                }
                Op::Promise(Promise {
                    queue,
                    data,
                    receiver,
                }) => vec![Op::Promise(Promise {
                    queue: queue.into_iter().flat_map(go).collect(),
                    data,
                    receiver,
                })],
                Op::Void(op) => vec![Op::Void(op)],
                Op::Noop => vec![],
            }
        }

        go(self)
            .into_iter()
            .flat_map(|op| {
                // flatten conc to multiple messages
                match op {
                    Op::Conc(ops) => ops.into_iter().collect(),
                    op => vec![op],
                }
            })
            .collect()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum QueueError {
    #[error("fatal error while handling message")]
    Fatal(#[source] BoxDynError),
    #[error("error while handling message")]
    Retry(#[source] BoxDynError),
}

pub trait CallT<T: QueueMessage> {
    fn process(self, store: &T::Context) -> impl Future<Output = Result<Op<T>, QueueError>> + Send;
}

pub trait CallbackT<T: QueueMessage> {
    fn process(
        self,
        ctx: &T::Context,
        data: VecDeque<T::Data>,
    ) -> impl Future<Output = Result<Op<T>, QueueError>> + Send;
}

impl<T: QueueMessage> CallT<T> for Never {
    async fn process(self, _: &T::Context) -> Result<Op<T>, QueueError> {
        match self {}
    }
}

/// Returns the current unix timestamp in seconds.
#[must_use = "retrieving the current timestamp has no effect"]
#[allow(clippy::missing_panics_doc)]
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the current timestamp must be greater than the unix epoch")
        .as_secs()
}

// hax
pub trait Captures<'a> {}
impl<T: ?Sized> Captures<'_> for T {}

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
    Op::Promise(Promise {
        queue: queue.into_iter().collect(),
        data: data.into_iter().collect(),
        receiver: callback.into(),
    })
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
