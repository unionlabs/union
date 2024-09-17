use crate::{conc, noop, race, repeat, retry, seq, void, Op, QueueMessage};

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
#[allow(clippy::let_and_return)]
pub fn normalize<T: QueueMessage>(msgs: Vec<Op<T>>) -> Vec<(Vec<usize>, Op<T>)> {
    // dbg!(&msgs);

    // let ids = msgs.iter().enumerate().map(|(id, _)| id).copied().collect::<Vec<_>>();
    let (parent_idxs, msgs): (Vec<_>, Vec<_>) = extract_data(msgs).into_iter().unzip();

    // dbg!(&msgs);

    let (parent_idxs, msgs): (Vec<_>, Vec<_>) =
        combine_normalization_pass_output(parent_idxs, remove_noop(msgs))
            .into_iter()
            .unzip();

    // dbg!(&msgs);

    let (parent_idxs, msgs): (Vec<_>, Vec<_>) =
        combine_normalization_pass_output(parent_idxs, flatten_seq(msgs))
            .into_iter()
            .unzip();

    // dbg!(&msgs);

    let output = combine_normalization_pass_output(parent_idxs, flatten_conc(msgs));

    // dbg!(&output);

    output
}

fn combine_normalization_pass_output<T: QueueMessage>(
    previous_parents: Vec<Vec<usize>>,
    mut output: Vec<(Vec<usize>, Op<T>)>,
) -> Vec<(Vec<usize>, Op<T>)> {
    for (parents, _) in &mut output {
        *parents = parents
            .iter()
            .flat_map(|p| &previous_parents[*p])
            .copied()
            .collect();
    }

    output
}

/// Extract all data out of the contained messages, pulling out into top-level messages.
///
/// Both `Sequence` and `Concurrent` are descended into, as well as `Aggregate` - for `Aggregate`,
/// `Data` messages are pulled out to the top level of the internal aggregation queue. For `seq`,
// `data` messages are only pulled out if they are in the front of the queue.
// REVIEW: Should data messages be queued as ready?
pub fn extract_data<T: QueueMessage>(msgs: Vec<Op<T>>) -> Vec<(Vec<usize>, Op<T>)> {
    // fn extract_data_internal<T: QueueMessage>(msg: Op<T>) -> Vec<Op<T>> {
    fn go<T: QueueMessage>(msg: Op<T>) -> Vec<Op<T>> {
        match msg {
            Op::Seq(msgs) => {
                let mut msgs = msgs.into_iter().flat_map(go).collect::<Vec<_>>();

                let first_non_data_msg_idx = msgs
                    .iter()
                    .enumerate()
                    .find_map(|(idx, msg)| (!matches!(msg, Op::Data(_))).then_some(idx))
                    .unwrap_or(msgs.len());

                // dbg!(&msgs);
                // dbg!(first_non_data_msg_idx);

                let non_data_msgs = msgs.split_off(first_non_data_msg_idx);
                let data_msgs = msgs;

                if non_data_msgs.is_empty() {
                    data_msgs
                } else {
                    data_msgs
                        .into_iter()
                        .chain([seq(non_data_msgs.into_iter().flat_map(go))])
                        .collect()
                }

                // if data.is_empty() {
                //     vec![seq(msgs)]
                // } else {
                //     data.into_iter().chain([seq(msgs)]).collect()
                // }
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
            Op::Promise {
                queue,
                data,
                receiver,
            } => vec![Op::Promise {
                queue: queue.into_iter().flat_map(go).collect(),
                data,
                receiver,
            }],
            _ => vec![msg],
        }
    }

    //     go(msg)
    // }

    msgs.into_iter()
        .enumerate()
        .flat_map(|(i, msg)| go(msg).into_iter().map(move |msg| (vec![i], msg)))
        .collect()
}

/// Remove any `Noop` messages that don't hold any semantic weight (noop in a race cannot be optimized away, for example)
pub fn remove_noop<T: QueueMessage>(msgs: Vec<Op<T>>) -> Vec<(Vec<usize>, Op<T>)> {
    fn remove_noop_internal<T: QueueMessage>(msg: Op<T>) -> Option<Op<T>> {
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
                Op::Promise {
                    queue,
                    data,
                    receiver,
                } => Some(Op::Promise {
                    queue: queue.into_iter().flat_map(remove_noop_internal).collect(),
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

    msgs.into_iter()
        .enumerate()
        .flat_map(|(i, msg)| remove_noop_internal(msg).map(|msg| (vec![i], msg)))
        .collect()
}

pub fn flatten_seq<T: QueueMessage>(msgs: Vec<Op<T>>) -> Vec<(Vec<usize>, Op<T>)> {
    fn flatten_seq_internal<T: QueueMessage>(msg: Op<T>) -> Option<Op<T>> {
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
                Op::Promise {
                    queue,
                    data,
                    receiver,
                } => vec![Op::Promise {
                    queue: queue.into_iter().filter_map(flatten_seq_internal).collect(),
                    data,
                    receiver,
                }],
                Op::Race(msgs) => {
                    vec![Op::Race(
                        msgs.into_iter()
                            .filter_map(|msg| {
                                // `noop`s hold semantic weight in the context of a race
                                if msg == noop() {
                                    Some(noop())
                                // empty seq holds semantic weight in the context of a race
                                } else if msg == seq([]) {
                                    Some(seq([]))
                                } else {
                                    flatten_seq_internal(msg)
                                }
                            })
                            .collect(),
                    )]
                }
                _ => [msg].into(),
            }
        }

        let mut msgs = go(msg);

        match msgs.len() {
            0 => None,
            1 => Some(msgs.pop().unwrap()),
            _ => Some(seq(msgs)),
        }
    }

    msgs.into_iter()
        .enumerate()
        .filter_map(|(i, msg)| flatten_seq_internal(msg).map(|msg| (vec![i], msg)))
        .collect()
}

pub fn flatten_conc<T: QueueMessage>(msgs: Vec<Op<T>>) -> Vec<(Vec<usize>, Op<T>)> {
    fn flatten_conc_internal<T: QueueMessage>(msg: Op<T>) -> Vec<Op<T>> {
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
                Op::Promise {
                    queue,
                    data,
                    receiver,
                } => vec![Op::Promise {
                    queue: queue.into_iter().flat_map(flatten_conc_internal).collect(),
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

    msgs.into_iter()
        .enumerate()
        .flat_map(|(i, msg)| {
            flatten_conc_internal(msg)
                .into_iter()
                .map(move |msg| (vec![i], msg))
        })
        .collect()
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::{
//         aggregate, data, defer_relative, effect, fetch, noop, race,
//         test_utils::{
//             AggregatePrintAbc, DataA, DataB, DataC, FetchA, PrintAbc, SimpleEvent, SimpleMessage,
//         },
//     };

//     #[test]
//     fn normalize() {
//         let msgs = vec![seq::<SimpleMessage>([
//             fetch(FetchA {}),
//             seq([
//                 data(DataA {}),
//                 noop(),
//                 fetch(FetchA {}),
//                 conc([
//                     effect(PrintAbc {
//                         a: DataA {},
//                         b: DataB {},
//                         c: DataC {},
//                     }),
//                     data(DataC {}),
//                     repeat(None, noop()),
//                 ]),
//                 fetch(FetchA {}),
//             ]),
//         ])];

//         let expected_output = vec![
//             (vec![0], data(DataA {})),
//             (vec![0], data(DataC {})),
//             (
//                 vec![0],
//                 seq([
//                     fetch(FetchA {}),
//                     fetch(FetchA {}),
//                     effect(PrintAbc {
//                         a: DataA {},
//                         b: DataB {},
//                         c: DataC {},
//                     }),
//                     fetch(FetchA {}),
//                 ]),
//             ),
//         ];

//         let optimized = Normalize::default().run_pass_pure(msgs.clone());
//         assert_eq!(optimized.optimize_further, expected_output);
//         assert_eq!(optimized.ready, []);

//         let optimized = NormalizeFinal::default().run_pass_pure(msgs);
//         assert_eq!(optimized.ready, expected_output);
//         assert_eq!(optimized.optimize_further, []);
//     }

//     #[test]
//     fn seq_conc_conc() {
//         let msgs = vec![seq::<SimpleMessage>([
//             conc([
//                 aggregate([], [], AggregatePrintAbc {}),
//                 aggregate([], [], AggregatePrintAbc {}),
//             ]),
//             conc([
//                 aggregate([], [], AggregatePrintAbc {}),
//                 aggregate([], [], AggregatePrintAbc {}),
//             ]),
//             conc([
//                 // repeat(None, seq([event(SimpleEvent {}), defer_relative(10)])),
//                 // repeat(None, seq([event(SimpleEvent {}), defer_relative(10)])),
//                 // this seq is the only message that should be flattened
//                 seq([
//                     effect(PrintAbc {
//                         a: DataA {},
//                         b: DataB {},
//                         c: DataC {},
//                     }),
//                     seq([
//                         aggregate([], [], AggregatePrintAbc {}),
//                         aggregate([], [], AggregatePrintAbc {}),
//                         aggregate([], [], AggregatePrintAbc {}),
//                     ]),
//                 ]),
//             ]),
//         ])];

//         let expected_output = vec![(
//             vec![0],
//             seq::<SimpleMessage>([
//                 conc([
//                     aggregate([], [], AggregatePrintAbc {}),
//                     aggregate([], [], AggregatePrintAbc {}),
//                 ]),
//                 conc([
//                     aggregate([], [], AggregatePrintAbc {}),
//                     aggregate([], [], AggregatePrintAbc {}),
//                 ]),
//                 conc([
//                     repeat(None, seq([event(SimpleEvent {}), defer_relative(10)])),
//                     repeat(None, seq([event(SimpleEvent {}), defer_relative(10)])),
//                     seq([
//                         effect(PrintAbc {
//                             a: DataA {},
//                             b: DataB {},
//                             c: DataC {},
//                         }),
//                         aggregate([], [], AggregatePrintAbc {}),
//                         aggregate([], [], AggregatePrintAbc {}),
//                         aggregate([], [], AggregatePrintAbc {}),
//                     ]),
//                 ]),
//             ]),
//         )];

//         let optimized = Normalize::default().run_pass_pure(msgs.clone());

//         assert_eq!(optimized.optimize_further, expected_output);
//         assert_eq!(optimized.ready, []);

//         let optimized = NormalizeFinal::default().run_pass_pure(msgs);
//         assert_eq!(optimized.ready, expected_output);
//         assert_eq!(optimized.optimize_further, []);
//     }

//     #[test]
//     fn race_opt() {
//         let msgs = vec![race::<SimpleMessage>([
//             seq([event(SimpleEvent {}), event(SimpleEvent {})]),
//             conc([
//                 event(SimpleEvent {}),
//                 conc([event(SimpleEvent {}), event(SimpleEvent {})]),
//             ]),
//         ])];

//         let expected_output = vec![(
//             vec![0],
//             race::<SimpleMessage>([
//                 seq([event(SimpleEvent {}), event(SimpleEvent {})]),
//                 conc([
//                     event(SimpleEvent {}),
//                     event(SimpleEvent {}),
//                     event(SimpleEvent {}),
//                 ]),
//             ]),
//         )];

//         let optimized = Normalize::default().run_pass_pure(msgs.clone());

//         assert_eq!(optimized.optimize_further, expected_output);
//         assert_eq!(optimized.ready, []);
//     }

//     #[test]
//     fn race_opt_noop() {
//         let msgs = vec![race::<SimpleMessage>([seq([]), conc([])])];

//         // an empty seq optimizes to an empty seq, but an empty conc optimizes to noop
//         let expected_output = vec![(vec![0], race::<SimpleMessage>([seq([]), noop()]))];

//         let optimized = Normalize::default().run_pass_pure(msgs.clone());

//         assert_eq!(optimized.optimize_further, expected_output);
//         assert_eq!(optimized.ready, []);
//     }
// }
