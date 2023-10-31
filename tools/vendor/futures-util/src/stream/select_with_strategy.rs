use super::assert_stream;
use crate::stream::{Fuse, StreamExt};
use core::{fmt, pin::Pin};
use futures_core::stream::{FusedStream, Stream};
use futures_core::task::{Context, Poll};
use pin_project_lite::pin_project;

/// Type to tell [`SelectWithStrategy`] which stream to poll next.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum PollNext {
    /// Poll the first stream.
    Left,
    /// Poll the second stream.
    Right,
}

impl PollNext {
    /// Toggle the value and return the old one.
    pub fn toggle(&mut self) -> Self {
        let old = *self;

        match self {
            PollNext::Left => *self = PollNext::Right,
            PollNext::Right => *self = PollNext::Left,
        }

        old
    }
}

impl Default for PollNext {
    fn default() -> Self {
        PollNext::Left
    }
}

pin_project! {
    /// Stream for the [`select_with_strategy()`] function. See function docs for details.
    #[must_use = "streams do nothing unless polled"]
    pub struct SelectWithStrategy<St1, St2, Clos, State> {
        #[pin]
        stream1: Fuse<St1>,
        #[pin]
        stream2: Fuse<St2>,
        state: State,
        clos: Clos,
    }
}

/// This function will attempt to pull items from both streams. You provide a
/// closure to tell [`SelectWithStrategy`] which stream to poll. The closure can
/// store state on `SelectWithStrategy` to which it will receive a `&mut` on every
/// invocation. This allows basing the strategy on prior choices.
///
/// After one of the two input streams completes, the remaining one will be
/// polled exclusively. The returned stream completes when both input
/// streams have completed.
///
/// Note that this function consumes both streams and returns a wrapped
/// version of them.
///
/// ## Examples
///
/// ### Priority
/// This example shows how to always prioritize the left stream.
///
/// ```rust
/// # futures::executor::block_on(async {
/// use futures::stream::{ repeat, select_with_strategy, PollNext, StreamExt };
///
/// let left = repeat(1);
/// let right = repeat(2);
///
/// // We don't need any state, so let's make it an empty tuple.
/// // We must provide some type here, as there is no way for the compiler
/// // to infer it. As we don't need to capture variables, we can just
/// // use a function pointer instead of a closure.
/// fn prio_left(_: &mut ()) -> PollNext { PollNext::Left }
///
/// let mut out = select_with_strategy(left, right, prio_left);
///
/// for _ in 0..100 {
///     // Whenever we poll out, we will alwas get `1`.
///     assert_eq!(1, out.select_next_some().await);
/// }
/// # });
/// ```
///
/// ### Round Robin
/// This example shows how to select from both streams round robin.
/// Note: this special case is provided by [`futures-util::stream::select`].
///
/// ```rust
/// # futures::executor::block_on(async {
/// use futures::stream::{ repeat, select_with_strategy, PollNext, StreamExt };
///
/// let left = repeat(1);
/// let right = repeat(2);
///
/// let rrobin = |last: &mut PollNext| last.toggle();
///
/// let mut out = select_with_strategy(left, right, rrobin);
///
/// for _ in 0..100 {
///     // We should be alternating now.
///     assert_eq!(1, out.select_next_some().await);
///     assert_eq!(2, out.select_next_some().await);
/// }
/// # });
/// ```
pub fn select_with_strategy<St1, St2, Clos, State>(
    stream1: St1,
    stream2: St2,
    which: Clos,
) -> SelectWithStrategy<St1, St2, Clos, State>
where
    St1: Stream,
    St2: Stream<Item = St1::Item>,
    Clos: FnMut(&mut State) -> PollNext,
    State: Default,
{
    assert_stream::<St1::Item, _>(SelectWithStrategy {
        stream1: stream1.fuse(),
        stream2: stream2.fuse(),
        state: Default::default(),
        clos: which,
    })
}

impl<St1, St2, Clos, State> SelectWithStrategy<St1, St2, Clos, State> {
    /// Acquires a reference to the underlying streams that this combinator is
    /// pulling from.
    pub fn get_ref(&self) -> (&St1, &St2) {
        (self.stream1.get_ref(), self.stream2.get_ref())
    }

    /// Acquires a mutable reference to the underlying streams that this
    /// combinator is pulling from.
    ///
    /// Note that care must be taken to avoid tampering with the state of the
    /// stream which may otherwise confuse this combinator.
    pub fn get_mut(&mut self) -> (&mut St1, &mut St2) {
        (self.stream1.get_mut(), self.stream2.get_mut())
    }

    /// Acquires a pinned mutable reference to the underlying streams that this
    /// combinator is pulling from.
    ///
    /// Note that care must be taken to avoid tampering with the state of the
    /// stream which may otherwise confuse this combinator.
    pub fn get_pin_mut(self: Pin<&mut Self>) -> (Pin<&mut St1>, Pin<&mut St2>) {
        let this = self.project();
        (this.stream1.get_pin_mut(), this.stream2.get_pin_mut())
    }

    /// Consumes this combinator, returning the underlying streams.
    ///
    /// Note that this may discard intermediate state of this combinator, so
    /// care should be taken to avoid losing resources when this is called.
    pub fn into_inner(self) -> (St1, St2) {
        (self.stream1.into_inner(), self.stream2.into_inner())
    }
}

impl<St1, St2, Clos, State> FusedStream for SelectWithStrategy<St1, St2, Clos, State>
where
    St1: Stream,
    St2: Stream<Item = St1::Item>,
    Clos: FnMut(&mut State) -> PollNext,
{
    fn is_terminated(&self) -> bool {
        self.stream1.is_terminated() && self.stream2.is_terminated()
    }
}

impl<St1, St2, Clos, State> Stream for SelectWithStrategy<St1, St2, Clos, State>
where
    St1: Stream,
    St2: Stream<Item = St1::Item>,
    Clos: FnMut(&mut State) -> PollNext,
{
    type Item = St1::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<St1::Item>> {
        let this = self.project();

        match (this.clos)(this.state) {
            PollNext::Left => poll_inner(this.stream1, this.stream2, cx),
            PollNext::Right => poll_inner(this.stream2, this.stream1, cx),
        }
    }
}

fn poll_inner<St1, St2>(
    a: Pin<&mut St1>,
    b: Pin<&mut St2>,
    cx: &mut Context<'_>,
) -> Poll<Option<St1::Item>>
where
    St1: Stream,
    St2: Stream<Item = St1::Item>,
{
    let a_done = match a.poll_next(cx) {
        Poll::Ready(Some(item)) => return Poll::Ready(Some(item)),
        Poll::Ready(None) => true,
        Poll::Pending => false,
    };

    match b.poll_next(cx) {
        Poll::Ready(Some(item)) => Poll::Ready(Some(item)),
        Poll::Ready(None) if a_done => Poll::Ready(None),
        Poll::Ready(None) | Poll::Pending => Poll::Pending,
    }
}

impl<St1, St2, Clos, State> fmt::Debug for SelectWithStrategy<St1, St2, Clos, State>
where
    St1: fmt::Debug,
    St2: fmt::Debug,
    State: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SelectWithStrategy")
            .field("stream1", &self.stream1)
            .field("stream2", &self.stream2)
            .field("state", &self.state)
            .finish()
    }
}
