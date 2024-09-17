use tracing::debug;

use crate::{
    conc, noop,
    optimize::{OptimizationResult, Pass, Pure, PurePass},
    race, repeat, retry, seq, void, Op, QueueMessage,
};

/// Combination of passes over the queue to normalize the internal structure. See the documentation
/// on each pass for more information on how they work individually.
///
/// This order is important, and it is recommended to reuse this type instead of the individual
/// passes when using them in combination with your own passes.
///
/// # Passes
///
/// First, any `Data` messages are extracted out of their contained structures, pulling them out
/// into individual top-level messages:
///
/// ```patch
/// - seq(.., seq(data, noop, .., conc(effect, data, repeat(noop)), ..))
/// + data, data, seq(.., seq(noop, .., conc(effect, repeat(noop)), ..))
/// ```
///
/// Then, any `Noop` messages are removed:
///
/// ```patch
/// - data, data, seq(.., seq(noop, .., conc(effect, repeat(noop)), ..))
/// + data, data, seq(.., seq(.., conc(effect), ..))
/// ```
///
/// Next, `Sequence`s are flattened:
///
/// ```patch
/// - data, data, seq(.., seq(noop, .., conc(effect, repeat(noop)), ..))
/// + data, data, seq(.., .., conc(effect), ..)
/// ```
///
/// Finally, `Concurrent`s are flattened:
///
/// ```patch
/// - data, data, seq(.., .., conc(effect), ..)
/// + data, data, seq(.., .., effect, ..)
/// ```
///
/// `FlattenSeq` and `FlattenConc` are associative, as are `ExtractData` and `RemoveNoop`.
///
/// Note how if the flattening occurred first, then the `conc(effect) -> effect` transformation could
/// never have occurred since the data and noop messages would still be there, resulting in an
/// incomplete normalization.
pub type Normalize = (ExtractData, (RemoveNoop, (FlattenSeq, FlattenConc)));
pub const NORMALIZE: Normalize = (ExtractData, (RemoveNoop, (FlattenSeq, FlattenConc)));

/// Runs the normalization passes, and then finalizes the optimizations with [`FinalPass`]. Use this
/// if you don't have any custom optimizations to run.
pub type NormalizeFinal = (Normalize, FinalPass);
pub const NORMALIZE_FINAL: NormalizeFinal = (NORMALIZE, FinalPass);

const _: fn() = || {
    fn impls_pure_pass<T: QueueMessage, P: PurePass<T>>() {}

    fn f<T: QueueMessage>() {
        impls_pure_pass::<T, Normalize>();
        impls_pure_pass::<T, NormalizeFinal>();
    }

    fn impls_pass<T: QueueMessage, P: Pass<T>>() {}

    fn g<T: QueueMessage>() {
        impls_pass::<T, Pure<Normalize>>();
        impls_pass::<T, Pure<NormalizeFinal>>();
    }
};

#[derive(Debug, Clone, Default)]
pub struct FinalPass;

impl<T: QueueMessage> PurePass<T> for FinalPass {
    fn run_pass_pure(&self, msgs: Vec<Op<T>>) -> OptimizationResult<T> {
        debug!("running final pass");

        OptimizationResult {
            optimize_further: vec![],
            ready: msgs
                .into_iter()
                .enumerate()
                .map(|(i, msg)| (vec![i], msg))
                .collect(),
        }
    }
}

/// Extract all data out of the contained messages, pulling out into top-level messages.
///
/// Both `Sequence` and `Concurrent` are descended into, as well as `Aggregate` - for `Aggregate`,
/// `Data` messages are pulled out to the top level of the internal aggregation queue.
// REVIEW: Should data messages be queued as ready?
#[derive(Debug, Clone, Default)]
pub struct ExtractData;

impl<T: QueueMessage> PurePass<T> for ExtractData {
    fn run_pass_pure(&self, msgs: Vec<Op<T>>) -> OptimizationResult<T> {
        fn extract_data<T: QueueMessage>(msg: Op<T>) -> Vec<Op<T>> {
            fn go<T: QueueMessage>(msg: Op<T>) -> Vec<Op<T>> {
                match msg {
                    Op::Seq(msgs) => {
                        let (data, msgs): (Vec<_>, Vec<_>) = msgs
                            .into_iter()
                            .flat_map(go)
                            .partition(|msg| matches!(msg, Op::Data(_)));

                        if data.is_empty() {
                            vec![seq(msgs)]
                        } else {
                            data.into_iter().chain([seq(msgs)]).collect()
                        }
                    }
                    Op::Conc(msgs) => {
                        let (data, msgs): (Vec<_>, Vec<_>) = msgs
                            .into_iter()
                            .flat_map(go)
                            .partition(|msg| matches!(msg, Op::Data(_)));

                        if data.is_empty() {
                            vec![conc(msgs)]
                        } else {
                            data.into_iter().chain([conc(msgs)]).collect()
                        }
                    }
                    Op::Aggregate {
                        queue,
                        data,
                        receiver,
                    } => vec![Op::Aggregate {
                        queue: queue.into_iter().flat_map(extract_data).collect(),
                        data,
                        receiver,
                    }],
                    _ => vec![msg],
                }
            }

            go(msg)
        }

        OptimizationResult {
            optimize_further: msgs
                .into_iter()
                .enumerate()
                .flat_map(|(i, msg)| extract_data(msg).into_iter().map(move |msg| (vec![i], msg)))
                .map(|(a, b)| (a, b, "NO TAG".to_owned()))
                .collect(),
            ready: vec![],
        }
    }
}

/// Remove any `Noop` messages.
#[derive(Debug, Clone, Default)]
pub struct RemoveNoop;

impl<T: QueueMessage> PurePass<T> for RemoveNoop {
    fn run_pass_pure(&self, msgs: Vec<Op<T>>) -> OptimizationResult<T> {
        fn remove_noop<T: QueueMessage>(msg: Op<T>) -> Option<Op<T>> {
            fn go<T: QueueMessage>(msg: Op<T>) -> Option<Op<T>> {
                match msg {
                    Op::Repeat { times, msg } => go(*msg).map(|msg| repeat(times, msg)),
                    Op::Timeout {
                        timeout_timestamp,
                        msg,
                    } => go(*msg).map(|msg| Op::Timeout {
                        timeout_timestamp,
                        msg: Box::new(msg),
                    }),
                    Op::Seq(msgs) => Some(seq(msgs.into_iter().flat_map(go))),
                    Op::Conc(msgs) => Some(conc(msgs.into_iter().flat_map(go))),
                    Op::Retry { remaining, msg } => go(*msg).map(|msg| retry(remaining, msg)),
                    Op::Aggregate {
                        queue,
                        data,
                        receiver,
                    } => Some(Op::Aggregate {
                        queue: queue.into_iter().flat_map(remove_noop).collect(),
                        data,
                        receiver,
                    }),
                    Op::Void(msg) => go(*msg).map(void),
                    Op::Noop => None,
                    _ => Some(msg),
                }
            }

            go(msg)
        }

        OptimizationResult {
            optimize_further: msgs
                .into_iter()
                .enumerate()
                .flat_map(|(i, msg)| remove_noop(msg).map(|msg| (vec![i], msg)))
                .map(|(a, b)| (a, b, "NO TAG".to_owned()))
                .collect(),
            ready: vec![],
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct FlattenSeq;

impl<T: QueueMessage> PurePass<T> for FlattenSeq {
    fn run_pass_pure(&self, msgs: Vec<Op<T>>) -> OptimizationResult<T> {
        fn flatten_seq<T: QueueMessage>(msg: Op<T>) -> Op<T> {
            fn go<T: QueueMessage>(msg: Op<T>) -> Vec<Op<T>> {
                match msg {
                    Op::Seq(new_seq) => new_seq.into_iter().flat_map(go).collect(),
                    Op::Conc(c) => vec![conc(c.into_iter().flat_map(|msg| {
                        let mut msgs = go(msg);

                        match msgs.len() {
                            0 => None,
                            1 => Some(msgs.pop().unwrap()),
                            _ => Some(seq(msgs)),
                        }
                    }))],
                    Op::Aggregate {
                        queue,
                        data,
                        receiver,
                    } => vec![Op::Aggregate {
                        queue: queue.into_iter().map(flatten_seq).collect(),
                        data,
                        receiver,
                    }],
                    Op::Race(msgs) => {
                        vec![Op::Race(msgs.into_iter().map(flatten_seq).collect())]
                    }
                    _ => [msg].into(),
                }
            }

            let mut msgs = go(msg);

            if msgs.len() == 1 {
                msgs.pop().unwrap()
            } else {
                seq(msgs)
            }
        }

        OptimizationResult {
            optimize_further: msgs
                .into_iter()
                .enumerate()
                .map(|(i, msg)| (vec![i], flatten_seq(msg)))
                .map(|(a, b)| (a, b, "NO TAG".to_owned()))
                .collect(),
            ready: vec![],
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct FlattenConc;

impl<T: QueueMessage> PurePass<T> for FlattenConc {
    fn run_pass_pure(&self, msgs: Vec<Op<T>>) -> OptimizationResult<T> {
        fn flatten_conc<T: QueueMessage>(msg: Op<T>) -> Vec<Op<T>> {
            fn go<T: QueueMessage>(msg: Op<T>) -> Vec<Op<T>> {
                match msg {
                    Op::Conc(new_conc) => new_conc.into_iter().flat_map(go).collect(),
                    Op::Seq(s) => vec![seq(s.into_iter().flat_map(|msg| {
                        let mut msgs = go(msg);

                        match msgs.len() {
                            0 => None,
                            1 => Some(msgs.pop().unwrap()),
                            // wrap in conc again
                            // seq(conc(a.., conc(b..)), c..) == seq(conc(a.., b..), c..)
                            // seq(conc(a.., conc(b..)), c..) != seq(a.., b.., c..)
                            _ => Some(conc(msgs)),
                        }
                    }))],
                    Op::Aggregate {
                        queue,
                        data,
                        receiver,
                    } => vec![Op::Aggregate {
                        queue: queue.into_iter().flat_map(flatten_conc).collect(),
                        data,
                        receiver,
                    }],
                    Op::Race(msgs) => {
                        vec![race(msgs.into_iter().map(|msg| {
                            let mut msgs = go(msg);

                            match msgs.len() {
                                0 => noop(),
                                1 => msgs.pop().unwrap(),
                                _ => conc(msgs),
                            }
                        }))]
                    }
                    _ => [msg].into(),
                }
            }

            go(msg)
        }

        OptimizationResult {
            optimize_further: msgs
                .into_iter()
                .enumerate()
                .flat_map(|(i, msg)| flatten_conc(msg).into_iter().map(move |msg| (vec![i], msg)))
                .map(|(a, b)| (a, b, "NO TAG".to_owned()))
                .collect(),
            ready: vec![],
        }
    }
}
