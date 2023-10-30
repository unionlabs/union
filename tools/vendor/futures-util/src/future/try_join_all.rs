//! Definition of the `TryJoinAll` combinator, waiting for all of a list of
//! futures to finish with either success or error.

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::fmt;
use core::future::Future;
use core::iter::FromIterator;
use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};

use super::{assert_future, TryFuture, TryMaybeDone};

fn iter_pin_mut<T>(slice: Pin<&mut [T]>) -> impl Iterator<Item = Pin<&mut T>> {
    // Safety: `std` _could_ make this unsound if it were to decide Pin's
    // invariants aren't required to transmit through slices. Otherwise this has
    // the same safety as a normal field pin projection.
    unsafe { slice.get_unchecked_mut() }.iter_mut().map(|t| unsafe { Pin::new_unchecked(t) })
}

enum FinalState<E = ()> {
    Pending,
    AllDone,
    Error(E),
}

/// Future for the [`try_join_all`] function.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct TryJoinAll<F>
where
    F: TryFuture,
{
    elems: Pin<Box<[TryMaybeDone<F>]>>,
}

impl<F> fmt::Debug for TryJoinAll<F>
where
    F: TryFuture + fmt::Debug,
    F::Ok: fmt::Debug,
    F::Error: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TryJoinAll").field("elems", &self.elems).finish()
    }
}

/// Creates a future which represents either a collection of the results of the
/// futures given or an error.
///
/// The returned future will drive execution for all of its underlying futures,
/// collecting the results into a destination `Vec<T>` in the same order as they
/// were provided.
///
/// If any future returns an error then all other futures will be canceled and
/// an error will be returned immediately. If all futures complete successfully,
/// however, then the returned future will succeed with a `Vec` of all the
/// successful results.
///
/// This function is only available when the `std` or `alloc` feature of this
/// library is activated, and it is activated by default.
///
/// # Examples
///
/// ```
/// # futures::executor::block_on(async {
/// use futures::future::{self, try_join_all};
///
/// let futures = vec![
///     future::ok::<u32, u32>(1),
///     future::ok::<u32, u32>(2),
///     future::ok::<u32, u32>(3),
/// ];
///
/// assert_eq!(try_join_all(futures).await, Ok(vec![1, 2, 3]));
///
/// let futures = vec![
///     future::ok::<u32, u32>(1),
///     future::err::<u32, u32>(2),
///     future::ok::<u32, u32>(3),
/// ];
///
/// assert_eq!(try_join_all(futures).await, Err(2));
/// # });
/// ```
pub fn try_join_all<I>(i: I) -> TryJoinAll<I::Item>
where
    I: IntoIterator,
    I::Item: TryFuture,
{
    let elems: Box<[_]> = i.into_iter().map(TryMaybeDone::Future).collect();
    assert_future::<Result<Vec<<I::Item as TryFuture>::Ok>, <I::Item as TryFuture>::Error>, _>(
        TryJoinAll { elems: elems.into() },
    )
}

impl<F> Future for TryJoinAll<F>
where
    F: TryFuture,
{
    type Output = Result<Vec<F::Ok>, F::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = FinalState::AllDone;

        for elem in iter_pin_mut(self.elems.as_mut()) {
            match elem.try_poll(cx) {
                Poll::Pending => state = FinalState::Pending,
                Poll::Ready(Ok(())) => {}
                Poll::Ready(Err(e)) => {
                    state = FinalState::Error(e);
                    break;
                }
            }
        }

        match state {
            FinalState::Pending => Poll::Pending,
            FinalState::AllDone => {
                let mut elems = mem::replace(&mut self.elems, Box::pin([]));
                let results =
                    iter_pin_mut(elems.as_mut()).map(|e| e.take_output().unwrap()).collect();
                Poll::Ready(Ok(results))
            }
            FinalState::Error(e) => {
                let _ = mem::replace(&mut self.elems, Box::pin([]));
                Poll::Ready(Err(e))
            }
        }
    }
}

impl<F: TryFuture> FromIterator<F> for TryJoinAll<F> {
    fn from_iter<T: IntoIterator<Item = F>>(iter: T) -> Self {
        try_join_all(iter)
    }
}
