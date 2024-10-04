use crate::{Op, QueueMessage};

/// A filter to run on [`Op`]s before they're pushed into the queue.
// TODO: Review if all of these trait bounds are necessary
pub trait InterestFilter<T: QueueMessage>: Send + Sync + Sized + 'static {
    fn check_interest<'a>(&'a self, op: &Op<T>) -> FilterResult<'a>;
}

pub enum FilterResult<'a> {
    Interest(&'a str),
    NoInterest,
}

impl<T: QueueMessage> InterestFilter<T> for () {
    fn check_interest<'a>(&'a self, op: &Op<T>) -> FilterResult<'a> {
        let _ = op;

        FilterResult::NoInterest
    }
}
