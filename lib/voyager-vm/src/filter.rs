use crate::{Op, QueueMessage};

/// A filter to run on [`Op`]s before they're pushed into the queue.
pub trait InterestFilter<T: QueueMessage>: Send + Sync + Sized + 'static {
    fn check_interest<'a>(&'a self, op: &Op<T>) -> FilterResult<'a>;
}

/// The result of running an [`InterestFilter`] on an [`Op`].
pub enum FilterResult<'a> {
    /// Interest has been expressed in this Op, with the contained tag. It will be inserted into the optimization queue under this tag.
    Interest(&'a str),
    /// No interest.
    NoInterest,
}

/// A noop implementation of an interest filter that never expresses interest in any messages.
impl<T: QueueMessage> InterestFilter<T> for () {
    fn check_interest<'a>(&'a self, op: &Op<T>) -> FilterResult<'a> {
        let _ = op;

        FilterResult::NoInterest
    }
}
