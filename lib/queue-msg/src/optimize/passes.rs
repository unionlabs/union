use tracing::debug;

use crate::{
    conc,
    optimize::{OptimizationResult, Pass, Pure, PurePass},
    repeat, retry, seq, QueueMessageTypes, QueueMsg,
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

/// Runs the normalization passes, and then finalizes the optimizations with [`FinalPass`]. Use this
/// if you don't have any custom optimizations to run.
pub type NormalizeFinal = (Normalize, FinalPass);

const _: fn() = || {
    fn impls_pure_pass<T: QueueMessageTypes, P: PurePass<T>>() {}

    fn f<T: QueueMessageTypes>() {
        impls_pure_pass::<T, Normalize>();
        impls_pure_pass::<T, NormalizeFinal>();
    }

    fn impls_pass<T: QueueMessageTypes, P: Pass<T>>() {}

    fn g<T: QueueMessageTypes>() {
        impls_pass::<T, Pure<Normalize>>();
        impls_pass::<T, Pure<NormalizeFinal>>();
    }
};

#[derive(Debug, Clone, Default)]
pub struct FinalPass;

impl<T: QueueMessageTypes> PurePass<T> for FinalPass {
    fn run_pass_pure(&self, msgs: Vec<QueueMsg<T>>) -> OptimizationResult<T> {
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

impl<T: QueueMessageTypes> PurePass<T> for ExtractData {
    fn run_pass_pure(&self, msgs: Vec<QueueMsg<T>>) -> OptimizationResult<T> {
        fn extract_data<T: QueueMessageTypes>(msg: QueueMsg<T>) -> Vec<QueueMsg<T>> {
            fn go<T: QueueMessageTypes>(msg: QueueMsg<T>) -> Vec<QueueMsg<T>> {
                match msg {
                    QueueMsg::Sequence(msgs) => {
                        let (data, msgs): (Vec<_>, Vec<_>) = msgs
                            .into_iter()
                            .flat_map(go)
                            .partition(|msg| matches!(msg, QueueMsg::Data(_)));

                        if data.is_empty() {
                            vec![seq(msgs)]
                        } else {
                            data.into_iter().chain([seq(msgs)]).collect()
                        }
                    }
                    QueueMsg::Concurrent(msgs) => {
                        let (data, msgs): (Vec<_>, Vec<_>) = msgs
                            .into_iter()
                            .flat_map(go)
                            .partition(|msg| matches!(msg, QueueMsg::Data(_)));

                        if data.is_empty() {
                            vec![conc(msgs)]
                        } else {
                            data.into_iter().chain([conc(msgs)]).collect()
                        }
                    }
                    QueueMsg::Aggregate {
                        queue,
                        data,
                        receiver,
                    } => vec![QueueMsg::Aggregate {
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
                .collect(),
            ready: vec![],
        }
    }
}

/// Remove any `Noop` messages.
#[derive(Debug, Clone, Default)]
pub struct RemoveNoop;

impl<T: QueueMessageTypes> PurePass<T> for RemoveNoop {
    fn run_pass_pure(&self, msgs: Vec<QueueMsg<T>>) -> OptimizationResult<T> {
        fn remove_noop<T: QueueMessageTypes>(msg: QueueMsg<T>) -> Option<QueueMsg<T>> {
            fn go<T: QueueMessageTypes>(msg: QueueMsg<T>) -> Option<QueueMsg<T>> {
                match msg {
                    QueueMsg::Repeat { times, msg } => go(*msg).map(|msg| repeat(times, msg)),
                    QueueMsg::Timeout {
                        timeout_timestamp,
                        msg,
                    } => go(*msg).map(|msg| QueueMsg::Timeout {
                        timeout_timestamp,
                        msg: Box::new(msg),
                    }),
                    QueueMsg::Sequence(msgs) => Some(seq(msgs.into_iter().flat_map(go))),
                    QueueMsg::Concurrent(msgs) => Some(conc(msgs.into_iter().flat_map(go))),
                    QueueMsg::Retry { remaining, msg } => go(*msg).map(|msg| retry(remaining, msg)),
                    QueueMsg::Aggregate {
                        queue,
                        data,
                        receiver,
                    } => Some(QueueMsg::Aggregate {
                        queue: queue.into_iter().flat_map(remove_noop).collect(),
                        data,
                        receiver,
                    }),
                    QueueMsg::Void(msg) => go(*msg),
                    QueueMsg::Noop => None,
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
                .collect(),
            ready: vec![],
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct FlattenSeq;

impl<T: QueueMessageTypes> PurePass<T> for FlattenSeq {
    fn run_pass_pure(&self, msgs: Vec<QueueMsg<T>>) -> OptimizationResult<T> {
        fn flatten_seq<T: QueueMessageTypes>(msg: QueueMsg<T>) -> QueueMsg<T> {
            fn go<T: QueueMessageTypes>(msg: QueueMsg<T>) -> Vec<QueueMsg<T>> {
                match msg {
                    QueueMsg::Sequence(new_seq) => new_seq.into_iter().flat_map(go).collect(),
                    QueueMsg::Concurrent(c) => vec![conc(c.into_iter().flat_map(|msg| {
                        let mut msgs = go(msg);

                        match msgs.len() {
                            0 => None,
                            1 => Some(msgs.pop().unwrap()),
                            _ => Some(seq(msgs)),
                        }
                    }))],
                    QueueMsg::Aggregate {
                        queue,
                        data,
                        receiver,
                    } => vec![QueueMsg::Aggregate {
                        queue: queue.into_iter().map(flatten_seq).collect(),
                        data,
                        receiver,
                    }],
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
                .collect(),
            ready: vec![],
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct FlattenConc;

impl<T: QueueMessageTypes> PurePass<T> for FlattenConc {
    fn run_pass_pure(&self, msgs: Vec<QueueMsg<T>>) -> OptimizationResult<T> {
        fn flatten_conc<T: QueueMessageTypes>(msg: QueueMsg<T>) -> Vec<QueueMsg<T>> {
            fn go<T: QueueMessageTypes>(msg: QueueMsg<T>) -> Vec<QueueMsg<T>> {
                match msg {
                    QueueMsg::Concurrent(new_conc) => new_conc.into_iter().flat_map(go).collect(),
                    QueueMsg::Sequence(s) => vec![seq(s.into_iter().flat_map(|msg| {
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
                    QueueMsg::Aggregate {
                        queue,
                        data,
                        receiver,
                    } => vec![QueueMsg::Aggregate {
                        queue: queue.into_iter().flat_map(flatten_conc).collect(),
                        data,
                        receiver,
                    }],
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
                .collect(),
            ready: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        aggregate, data, defer_relative, effect, event, fetch, noop,
        test_utils::{
            AggregatePrintAbc, DataA, DataB, DataC, FetchA, PrintAbc, SimpleEvent, SimpleMessage,
        },
    };

    #[test]
    fn normalize() {
        let msgs = vec![seq::<SimpleMessage>([
            fetch(FetchA {}),
            seq([
                data(DataA {}),
                noop(),
                fetch(FetchA {}),
                conc([
                    effect(PrintAbc {
                        a: DataA {},
                        b: DataB {},
                        c: DataC {},
                    }),
                    data(DataC {}),
                    repeat(None, noop()),
                ]),
                fetch(FetchA {}),
            ]),
        ])];

        let expected_output = vec![
            (vec![0], data(DataA {})),
            (vec![0], data(DataC {})),
            (
                vec![0],
                seq([
                    fetch(FetchA {}),
                    fetch(FetchA {}),
                    effect(PrintAbc {
                        a: DataA {},
                        b: DataB {},
                        c: DataC {},
                    }),
                    fetch(FetchA {}),
                ]),
            ),
        ];

        let optimized = Normalize::default().run_pass_pure(msgs.clone());
        assert_eq!(optimized.optimize_further, expected_output);
        assert_eq!(optimized.ready, []);

        let optimized = NormalizeFinal::default().run_pass_pure(msgs);
        assert_eq!(optimized.ready, expected_output);
        assert_eq!(optimized.optimize_further, []);
    }

    #[test]
    fn seq_conc_conc() {
        let msgs = vec![seq::<SimpleMessage>([
            conc([
                aggregate([], [], AggregatePrintAbc {}),
                aggregate([], [], AggregatePrintAbc {}),
            ]),
            conc([
                aggregate([], [], AggregatePrintAbc {}),
                aggregate([], [], AggregatePrintAbc {}),
            ]),
            conc([
                repeat(None, seq([event(SimpleEvent {}), defer_relative(10)])),
                repeat(None, seq([event(SimpleEvent {}), defer_relative(10)])),
                // this seq is the only message that should be flattened
                seq([
                    effect(PrintAbc {
                        a: DataA {},
                        b: DataB {},
                        c: DataC {},
                    }),
                    seq([
                        aggregate([], [], AggregatePrintAbc {}),
                        aggregate([], [], AggregatePrintAbc {}),
                        aggregate([], [], AggregatePrintAbc {}),
                    ]),
                ]),
            ]),
        ])];

        let expected_output = vec![(
            vec![0],
            seq::<SimpleMessage>([
                conc([
                    aggregate([], [], AggregatePrintAbc {}),
                    aggregate([], [], AggregatePrintAbc {}),
                ]),
                conc([
                    aggregate([], [], AggregatePrintAbc {}),
                    aggregate([], [], AggregatePrintAbc {}),
                ]),
                conc([
                    repeat(None, seq([event(SimpleEvent {}), defer_relative(10)])),
                    repeat(None, seq([event(SimpleEvent {}), defer_relative(10)])),
                    seq([
                        effect(PrintAbc {
                            a: DataA {},
                            b: DataB {},
                            c: DataC {},
                        }),
                        aggregate([], [], AggregatePrintAbc {}),
                        aggregate([], [], AggregatePrintAbc {}),
                        aggregate([], [], AggregatePrintAbc {}),
                    ]),
                ]),
            ]),
        )];

        let optimized = Normalize::default().run_pass_pure(msgs.clone());

        assert_eq!(optimized.optimize_further, expected_output);
        assert_eq!(optimized.ready, []);

        let optimized = NormalizeFinal::default().run_pass_pure(msgs);
        assert_eq!(optimized.ready, expected_output);
        assert_eq!(optimized.optimize_further, []);
    }
}
