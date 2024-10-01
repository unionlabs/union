use either::Either::{Left, Right};
use itertools::Itertools;

use crate::{Op, QueueMessage};

pub fn normalize<T: QueueMessage>(ops: Vec<Op<T>>) -> Vec<(Vec<usize>, Op<T>)> {
    pub fn go<T: QueueMessage>(op: Op<T>) -> Vec<Op<T>> {
        match op {
            Op::Data(data) => vec![Op::Data(data)],
            Op::Call(call) => vec![Op::Call(call)],
            Op::Defer { until } => vec![Op::Defer { until }],
            Op::Timeout {
                timeout_timestamp,
                msg,
            } => vec![Op::Timeout {
                timeout_timestamp,
                msg,
            }],
            Op::Seq(seq) => {
                let mut ops = seq.into_iter().flat_map(go).collect::<Vec<_>>();

                let first_non_data_msg_idx = ops
                    .iter()
                    .enumerate()
                    .find_map(|(idx, msg)| (!matches!(msg, Op::Data(_))).then_some(idx))
                    .unwrap_or(ops.len());

                match ops.len() {
                    0 => vec![],
                    1 => vec![ops.pop().expect("length is 1; qed;")],
                    2.. => {
                        let non_data_msgs = ops.split_off(first_non_data_msg_idx);
                        let data_msgs = ops;

                        if non_data_msgs.is_empty() {
                            data_msgs
                        } else {
                            data_msgs
                                .into_iter()
                                .chain([Op::Seq(
                                    non_data_msgs
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
            Op::Retry { remaining, msg } => vec![Op::Retry { remaining, msg }],
            Op::Promise {
                queue,
                data,
                receiver,
            } => vec![Op::Promise {
                queue: queue.into_iter().flat_map(go).collect(),
                data,
                receiver,
            }],
            Op::Void(op) => vec![Op::Void(op)],
            Op::Noop => vec![],
        }
    }

    ops.into_iter()
        .enumerate()
        .flat_map(|(i, op)| {
            // flatten conc to multiple messages
            go(op).into_iter().flat_map(move |op| match op {
                Op::Conc(ops) => ops.into_iter().map(move |op| (vec![i], op)).collect(),
                op => vec![(vec![i], op)],
            })
        })
        .collect()
}
