use crate::{Op, QueueMessage};

/// A filter to run on [`Op`]s before they're pushed into the queue.
pub trait InterestFilter<T: QueueMessage>: Send + Sync + Sized + 'static {
    fn check_interest<'a>(&'a self, op: &Op<T>) -> FilterResult<'a>;
}

/// The result of running an [`InterestFilter`] on an [`Op`].
pub enum FilterResult<'a> {
    Interest(Interest<'a>),
    /// No interest.
    NoInterest,
}

/// Interest has been expressed in this Op, with the contained tag(s). It will be inserted into the optimization queue under these tag(s).
pub struct Interest<'a> {
    /// Tags that expressed interest in a copy of an Op.
    pub tags: Vec<&'a str>,
    /// Whether or not to remove the Op from the queue.
    pub remove: bool,
}

/// A noop implementation of an interest filter that never expresses interest in any messages.
impl<T: QueueMessage> InterestFilter<T> for () {
    fn check_interest<'a>(&'a self, op: &Op<T>) -> FilterResult<'a> {
        let _ = op;

        FilterResult::NoInterest
    }
}
