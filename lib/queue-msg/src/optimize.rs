use std::{convert::Infallible, error::Error, fmt::Debug};

use either::Either;
use frame_support_procedural::{CloneNoBound, DebugNoBound, DefaultNoBound};
use futures::Future;
use serde::{Deserialize, Serialize};

use crate::{Op, QueueMessage};

/// An optimization pass over the queue. This is an "impure" pass, in that it can access the
/// external environment and it is fallible.
pub trait Pass<T: QueueMessage>: Send + Sync + Sized {
    type Error: Error + Send + Sync + 'static;

    fn run_pass(
        &self,
        msgs: Vec<Op<T>>,
    ) -> impl Future<Output = Result<OptimizationResult<T>, Self::Error>> + Send;
}

/// An optimization pass over the queue. This is a "pure" pass, in that it should not access the
/// environment and is infallible.
///
/// Note that while it is technically possible to do blocking network requests and access the
/// external environment, please do not do that.
pub trait PurePass<T: QueueMessage>: Debug + Clone + Send + Sync + Sized + 'static {
    fn run_pass_pure(&self, msgs: Vec<Op<T>>) -> OptimizationResult<T>;
}

/// The result of running an optimization pass. Both `optimize_further` and `ready` are lists of
/// `(parents, msg)`, allowing for correlating new messages with multiple parents (i.e. combining
/// messages).
#[derive(DebugNoBound, CloneNoBound, DefaultNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct OptimizationResult<T: QueueMessage> {
    /// Messages that are considered incomplete by this optimization pass and are to be optimized
    /// further.
    ///
    /// For pure passes, it is recommended to return all messages here after being processed so
    /// that subsequent passes can run on them as well. In fact, all of the passes defined in
    /// [`passes`] work this way, with [`FinalPass`] requeueing everything under `ready`.
    pub optimize_further: Vec<(Vec<usize>, Op<T>, String)>,
    /// Messages that are considered complete by this optimization pass. No more passes will be run
    /// on these messages, and they will be requeued as "ready" in the queue.
    pub ready: Vec<(Vec<usize>, Op<T>)>,
}

/// Wrap a [`PurePass`] in this to make it into a [`Pass`], composable with other [`Pass`]es.
///
/// `(Pure<T>, Pure<U>)` is equivalent to `Pure<(T, U)>`.
#[derive(Debug, Clone)]
pub struct Pure<T>(pub T);

impl<O: PurePass<T>, T: QueueMessage> Pass<T> for Pure<O> {
    type Error = Infallible;

    async fn run_pass(&self, msgs: Vec<Op<T>>) -> Result<OptimizationResult<T>, Self::Error> {
        Ok(self.0.run_pass_pure(msgs))
    }
}

impl<T: QueueMessage, A: Pass<T>, B: Pass<T>> Pass<T> for (A, B) {
    type Error = Either<A::Error, B::Error>;

    async fn run_pass(&self, msgs: Vec<Op<T>>) -> Result<OptimizationResult<T>, Self::Error> {
        let res_a = self.0.run_pass(msgs).await.map_err(Either::Left)?;
        let (pass_a_optimize_further_parent_ids, (pass_1_optimize_further, _)): (
            Vec<_>,
            (Vec<_>, Vec<_>),
        ) = res_a
            .optimize_further
            .into_iter()
            .map(|(a, b, c)| (a, (b, c)))
            .unzip();

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

impl<T: QueueMessage, A: PurePass<T>, B: PurePass<T>> PurePass<T> for (A, B) {
    fn run_pass_pure(&self, msgs: Vec<Op<T>>) -> OptimizationResult<T> {
        let res_a = self.0.run_pass_pure(msgs);
        let (pass_a_optimize_further_parent_ids, (pass_1_optimize_further, _)): (
            Vec<_>,
            (Vec<_>, Vec<_>),
        ) = res_a
            .optimize_further
            .into_iter()
            .map(|(a, b, c)| (a, (b, c)))
            .unzip();

        let res_b = self.1.run_pass_pure(pass_1_optimize_further);

        combine_optimization_results(res_b, pass_a_optimize_further_parent_ids, res_a.ready)
    }
}

fn combine_optimization_results<T: QueueMessage>(
    mut res_b: OptimizationResult<T>,
    pass_a_optimize_further_parent_ids: Vec<Vec<usize>>,
    res_a_ready: Vec<(Vec<usize>, Op<T>)>,
) -> OptimizationResult<T> {
    for (parents, _) in &mut res_b.ready {
        *parents = parents
            .iter()
            .flat_map(|p| &pass_a_optimize_further_parent_ids[*p])
            .copied()
            .collect();
    }

    for (parents, _, _) in &mut res_b.optimize_further {
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

#[derive(Debug, Clone, PartialEq)]
pub struct NoopPass;

impl<T: QueueMessage> PurePass<T> for NoopPass {
    fn run_pass_pure(&self, msgs: Vec<Op<T>>) -> OptimizationResult<T> {
        OptimizationResult {
            optimize_further: vec![],
            ready: msgs
                .into_iter()
                .enumerate()
                .map(|(idx, msg)| (vec![idx], msg))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        call, conc, data, defer, noop,
        normalize::normalize,
        now, promise, race, repeat, seq,
        test_utils::{BuildPrintAbc, DataA, DataB, DataC, FetchA, FetchB, PrintAbc, SimpleMessage},
    };

    #[test]
    fn normalize_works_in_single_pass() {
        let msgs = vec![seq::<SimpleMessage>([
            call(FetchA {}),
            seq([
                data(DataA {}),
                noop(),
                call(FetchA {}),
                conc([
                    call(PrintAbc {
                        a: DataA {},
                        b: DataB {},
                        c: DataC {},
                    }),
                    data(DataC {}),
                    repeat(None, noop()),
                ]),
                call(FetchA {}),
            ]),
        ])];

        let expected_output = vec![
            (vec![0], data(DataA {})),
            (vec![0], data(DataC {})),
            (
                vec![0],
                seq([
                    call(FetchA {}),
                    call(FetchA {}),
                    call(PrintAbc {
                        a: DataA {},
                        b: DataB {},
                        c: DataC {},
                    }),
                    call(FetchA {}),
                ]),
            ),
        ];

        let optimized = normalize(msgs.clone());
        assert_eq!(optimized, expected_output);

        let optimized = normalize(msgs);
        assert_eq!(optimized, expected_output);
    }

    #[test]
    fn seq_conc_conc() {
        let msgs = vec![seq::<SimpleMessage>([
            conc([
                promise([], [], BuildPrintAbc {}),
                promise([], [], BuildPrintAbc {}),
            ]),
            conc([
                promise([], [], BuildPrintAbc {}),
                promise([], [], BuildPrintAbc {}),
            ]),
            conc([
                repeat(None, seq([call(FetchA {}), defer(now() + 10)])),
                repeat(None, seq([call(FetchB {}), defer(now() + 10)])),
                // this seq is the only message that should be flattened
                seq([
                    call(PrintAbc {
                        a: DataA {},
                        b: DataB {},
                        c: DataC {},
                    }),
                    seq([
                        promise([], [], BuildPrintAbc {}),
                        promise([], [], BuildPrintAbc {}),
                        promise([], [], BuildPrintAbc {}),
                    ]),
                ]),
            ]),
        ])];

        let expected_output = vec![(
            vec![0],
            seq::<SimpleMessage>([
                conc([
                    promise([], [], BuildPrintAbc {}),
                    promise([], [], BuildPrintAbc {}),
                ]),
                conc([
                    promise([], [], BuildPrintAbc {}),
                    promise([], [], BuildPrintAbc {}),
                ]),
                conc([
                    repeat(None, seq([call(FetchA {}), defer(now() + 10)])),
                    repeat(None, seq([call(FetchB {}), defer(now() + 10)])),
                    seq([
                        call(PrintAbc {
                            a: DataA {},
                            b: DataB {},
                            c: DataC {},
                        }),
                        promise([], [], BuildPrintAbc {}),
                        promise([], [], BuildPrintAbc {}),
                        promise([], [], BuildPrintAbc {}),
                    ]),
                ]),
            ]),
        )];

        let optimized = normalize(msgs.clone());
        assert_eq!(optimized, expected_output);

        let optimized = normalize(msgs);
        assert_eq!(optimized, expected_output);
    }

    #[test]
    fn race_opt() {
        let msgs = vec![race::<SimpleMessage>([
            seq([call(FetchA {}), call(FetchB {})]),
            conc([call(FetchA {}), conc([call(FetchA {}), call(FetchA {})])]),
        ])];

        let expected_output = vec![(
            vec![0],
            race::<SimpleMessage>([
                seq([call(FetchA {}), call(FetchB {})]),
                conc([call(FetchA {}), call(FetchA {}), call(FetchA {})]),
            ]),
        )];

        let optimized = normalize(msgs);
        assert_eq!(optimized, expected_output);
    }

    #[test]
    fn race_opt_noop() {
        let msgs = vec![race::<SimpleMessage>([seq([]), conc([])])];

        // an empty seq optimizes to an empty seq, but an empty conc optimizes to noop
        // REVIEW: Why?
        let expected_output = vec![(vec![0], race::<SimpleMessage>([seq([]), noop()]))];

        let optimized = normalize(msgs);
        assert_eq!(optimized, expected_output);
    }
}
