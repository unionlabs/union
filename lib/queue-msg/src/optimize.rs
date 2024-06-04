use std::{convert::Infallible, error::Error, fmt::Debug};

use either::Either;
use frame_support_procedural::{CloneNoBound, DebugNoBound};
use futures::Future;

use crate::{QueueMessageTypes, QueueMsg};

pub mod passes;

/// An optimization pass over the queue. This is an "impure" pass, in that it can access the
/// external environment and it is fallible.
pub trait Pass<T: QueueMessageTypes>: Debug + Clone + Send + Sync + Sized + 'static {
    type Error: Error + Send + Sync + 'static;

    fn run_pass(
        &self,
        msgs: Vec<QueueMsg<T>>,
    ) -> impl Future<Output = Result<OptimizationResult<T>, Self::Error>> + Send;
}

/// An optimization pass over the queue. This is a "pure" pass, in that it should not access the
/// environment and is infallible.
///
/// Note that while it is technically possible to do blocking network requests and access the
/// external environment, please do not do that.
pub trait PurePass<T: QueueMessageTypes>: Debug + Clone + Send + Sync + Sized + 'static {
    fn run_pass_pure(&self, msgs: Vec<QueueMsg<T>>) -> OptimizationResult<T>;
}

/// The result of running an optimization pass. Both `optimize_further` and `ready` are lists of
/// `(parents, msg)`, allowing for correlating new messages with multiple parents (i.e. combining
/// messages).
#[derive(DebugNoBound, CloneNoBound)]
pub struct OptimizationResult<T: QueueMessageTypes> {
    /// Messages that are considered incomplete by this optimization pass and are to be optimized
    /// further.
    ///
    /// For pure passes, it is recommended to return all messages here after being processed so
    /// that subsequent passes can run on them as well. In fact, all of the passes defined in
    /// [`passes`] work this way, with [`FinalPass`] requeueing everything under `ready`.
    pub optimize_further: Vec<(Vec<usize>, QueueMsg<T>)>,
    /// Messages that are considered complete by this optimization pass. No more passes will be run
    /// on these messages, and they will be requeued as "ready" in the queue.
    pub ready: Vec<(Vec<usize>, QueueMsg<T>)>,
}

/// Wrap a [`PurePass`] in this to make it into a [`Pass`], composable with other [`Pass`]es.
///
/// `(Pure<T>, Pure<U>)` is equivalent to `Pure<(T, U)>`.
#[derive(Debug, Clone)]
pub struct Pure<T>(pub T);

impl<O: PurePass<T>, T: QueueMessageTypes> Pass<T> for Pure<O> {
    type Error = Infallible;

    async fn run_pass(&self, msgs: Vec<QueueMsg<T>>) -> Result<OptimizationResult<T>, Self::Error> {
        Ok(self.0.run_pass_pure(msgs))
    }
}

impl<T: QueueMessageTypes, A: Pass<T>, B: Pass<T>> Pass<T> for (A, B) {
    type Error = Either<A::Error, B::Error>;

    async fn run_pass(&self, msgs: Vec<QueueMsg<T>>) -> Result<OptimizationResult<T>, Self::Error> {
        let res_a = self.0.run_pass(msgs).await.map_err(Either::Left)?;
        let (pass_a_optimize_further_parent_ids, pass_1_optimize_further): (Vec<_>, Vec<_>) =
            res_a.optimize_further.into_iter().unzip();

        let res_b = self
            .1
            .run_pass(pass_1_optimize_further)
            .await
            .map_err(Either::Right)?;

        Ok(combine_optimization_results(
            res_b,
            pass_a_optimize_further_parent_ids,
            res_a.ready,
        ))
    }
}

impl<T: QueueMessageTypes, A: PurePass<T>, B: PurePass<T>> PurePass<T> for (A, B) {
    fn run_pass_pure(&self, msgs: Vec<QueueMsg<T>>) -> OptimizationResult<T> {
        let res_a = self.0.run_pass_pure(msgs);
        let (pass_a_optimize_further_parent_ids, pass_1_optimize_further): (Vec<_>, Vec<_>) =
            res_a.optimize_further.into_iter().unzip();

        let res_b = self.1.run_pass_pure(pass_1_optimize_further);

        combine_optimization_results(res_b, pass_a_optimize_further_parent_ids, res_a.ready)
    }
}

fn combine_optimization_results<T: QueueMessageTypes>(
    mut res_b: OptimizationResult<T>,
    pass_a_optimize_further_parent_ids: Vec<Vec<usize>>,
    res_a_ready: Vec<(Vec<usize>, QueueMsg<T>)>,
) -> OptimizationResult<T> {
    for (parents, _) in &mut res_b.ready {
        *parents = parents
            .iter()
            .flat_map(|p| &pass_a_optimize_further_parent_ids[*p])
            .copied()
            .collect();
    }

    for (parents, _) in &mut res_b.optimize_further {
        *parents = parents
            .iter()
            .flat_map(|p| &pass_a_optimize_further_parent_ids[*p])
            .copied()
            .collect();
    }

    OptimizationResult {
        ready: res_a_ready.into_iter().chain(res_b.ready).collect(),
        optimize_further: res_b.optimize_further,
    }
}
