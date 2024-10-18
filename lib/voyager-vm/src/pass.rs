use std::error::Error;

use derive_where::derive_where;
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
#[derive_where(Debug, PartialEq, Clone)]
#[derive(Serialize, Deserialize)]
#[serde(
    rename_all = "snake_case",
    bound(serialize = "", deserialize = ""),
    deny_unknown_fields
)]
pub struct PassResult<T: QueueMessage> {
    /// [`Op`]s that are considered incomplete by this optimization pass and are to be optimized
    /// further.
    pub optimize_further: Vec<(Vec<usize>, Op<T>, String)>,
    /// [`Op`]s that are considered complete by this optimization pass. No more passes will be run
    /// on these [`Op`]s, and they will be requeued as "ready" in the queue.
    pub ready: Vec<(Vec<usize>, Op<T>)>,
}
