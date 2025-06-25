use std::error::Error;

use futures::Future;
use serde::{Deserialize, Serialize};

use crate::{Op, QueueMessage};

/// An optimization pass over the queue.
pub trait Pass<T: QueueMessage>: Send + Sync + Sized {
    type Error: Error + Send + Sync + 'static;

    fn run_pass(
        &self,
        ops: Vec<Op<T>>,
    ) -> impl Future<Output = Result<PassResult<T>, Self::Error>> + Send;
}

/// The result of running an optimization pass. Both `optimize_further` and `ready` are lists of
/// `(parents, op)`, allowing for correlating new messages with multiple parents (i.e. combining
/// messages).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct PassResult<T: QueueMessage> {
    /// [`Op`]s that are considered incomplete by this optimization pass and are to be optimized
    /// further.
    pub optimize_further: Vec<OptimizeFurther<T>>,
    /// [`Op`]s that are considered complete by this optimization pass. No more passes will be run
    /// on these [`Op`]s, and they will be requeued as "ready" in the queue.
    pub ready: Vec<Ready<T>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct OptimizeFurther<T: QueueMessage> {
    pub parents: Vec<usize>,
    pub op: Op<T>,
    pub tag: String,
}

impl<T: QueueMessage> OptimizeFurther<T> {
    pub fn new(parents: Vec<usize>, op: Op<T>, tag: String) -> Self {
        Self { parents, op, tag }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct Ready<T: QueueMessage> {
    pub parents: Vec<usize>,
    pub op: Op<T>,
}

impl<T: QueueMessage> Ready<T> {
    pub fn new(parents: Vec<usize>, op: Op<T>) -> Self {
        Self { parents, op }
    }
}

impl<T: QueueMessage> Default for PassResult<T> {
    fn default() -> Self {
        Self {
            optimize_further: vec![],
            ready: vec![],
        }
    }
}
