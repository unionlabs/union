use queue_msg::{
    optimize::{OptimizationResult, PurePass},
    Op,
};
use voyager_message::VoyagerMessage;

/// Marks all block_message messages as ready, such that they can be processed sooner.
///
/// Intended to be run as a pre-push pass to skip the roundtrip between the optimizer and the queue.
#[derive(Clone, Debug)]
pub struct BlockPassthrough;

impl PurePass<VoyagerMessage> for BlockPassthrough {
    fn run_pass_pure(&self, msgs: Vec<Op<VoyagerMessage>>) -> OptimizationResult<VoyagerMessage> {
        msgs.into_iter().enumerate().map(|(idx, msg)| match msg {
            Op::Event(_) => todo!(),
            Op::Data(_) => todo!(),
            Op::Fetch(_) => todo!(),
            Op::Effect(_) => todo!(),
            Op::Wait(_) => todo!(),
            Op::Defer(_) => todo!(),
            Op::Repeat { times, msg } => todo!(),
            Op::Timeout {
                timeout_timestamp,
                msg,
            } => todo!(),
            Op::Seq(_) => todo!(),
            Op::Conc(_) => todo!(),
            Op::Race(_) => todo!(),
            Op::Retry { remaining, msg } => todo!(),
            Op::Aggregate {
                queue,
                data,
                receiver,
            } => todo!(),
            Op::Void(_) => todo!(),
            Op::Noop => todo!(),
        })
    }
}
