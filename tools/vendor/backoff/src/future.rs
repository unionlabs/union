use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use futures_core::ready;
use pin_project_lite::pin_project;

use crate::{backoff::Backoff, error::Error};

use crate::retry::{NoopNotify, Notify};

pub trait Sleeper {
    type Sleep: Future<Output = ()> + Send + 'static;
    fn sleep(&self, dur: Duration) -> Self::Sleep;
}

/// Retries given `operation` according to the [`Backoff`] policy
/// [`Backoff`] is reset before it is used.
/// The returned future can be spawned onto a compatible runtime.
///
/// Only available through the `tokio` and `async-std` feature flags.
///
/// # Example
///
/// ```rust
/// use backoff::ExponentialBackoff;
///
/// async fn f() -> Result<(), backoff::Error<&'static str>> {
///     // Business logic...
///     Err(backoff::Error::Permanent("error"))
/// }
///
/// # async fn go() {
/// backoff::future::retry(ExponentialBackoff::default(), f).await.err().unwrap();
/// # }
/// # fn main() { futures_executor::block_on(go()); }
/// ```
#[cfg(any(feature = "tokio", feature = "async-std"))]
pub fn retry<I, E, Fn, Fut, B>(
    backoff: B,
    operation: Fn,
) -> Retry<impl Sleeper, B, NoopNotify, Fn, Fut>
where
    B: Backoff,
    Fn: FnMut() -> Fut,
    Fut: Future<Output = Result<I, Error<E>>>,
{
    retry_notify(backoff, operation, NoopNotify)
}

/// Retries given `operation` according to the [`Backoff`] policy.
/// Calls `notify` on failed attempts (in case of [`Error::Transient`]).
/// [`Backoff`] is reset before it is used.
/// The returned future can be spawned onto a compatible runtime.
///
/// Only available through the `tokio` and `async-std` feature flags.
///
/// # Async `notify`
///
/// `notify` can be neither `async fn` or [`Future`]. If you need to perform some async
/// operations inside `notify`, consider using your runtimes task-spawning functionality.
///
/// The reason behind this is that [`Retry`] future cannot be responsible for polling
/// `notify` future, because can easily be dropped _before_ `notify` is completed.
/// So, considering the fact that most of the time no async operations are required in
/// `notify`, it's up to the caller to decide how async `notify` should be performed.
///
/// # Example
///
/// ```rust
/// use backoff::backoff::Stop;
///
/// async fn f() -> Result<(), backoff::Error<&'static str>> {
///     // Business logic...
///     Err(backoff::Error::transient("error"))
/// }
///
/// # async fn go() {
/// let err = backoff::future::retry_notify(Stop {}, f, |e, dur| {
///     println!("Error happened at {:?}: {}", dur, e)
/// })
/// .await
/// .err()
/// .unwrap();
/// assert_eq!(err, "error");
/// # }
/// # fn main() { futures_executor::block_on(go()); }
/// ```
#[cfg(any(feature = "tokio", feature = "async-std"))]
pub fn retry_notify<I, E, Fn, Fut, B, N>(
    mut backoff: B,
    operation: Fn,
    notify: N,
) -> Retry<impl Sleeper, B, N, Fn, Fut>
where
    B: Backoff,
    Fn: FnMut() -> Fut,
    Fut: Future<Output = Result<I, Error<E>>>,
    N: Notify<E>,
{
    backoff.reset();
    Retry::new(rt_sleeper(), backoff, notify, operation)
}

pin_project! {
    /// Retry implementation.
    pub struct Retry<S: Sleeper, B, N, Fn, Fut> {
        // The [`Sleeper`] that we generate the `delay` futures from.
        sleeper: S,

        // [`Backoff`] implementation to count next [`Retry::delay`] with.
        backoff: B,

        // [`Future`] which delays execution before next [`Retry::operation`] invocation.
        #[pin]
        delay: OptionPinned<S::Sleep>,

        // Operation to be retried. It must return [`Future`].
        operation: Fn,

        // [`Future`] being resolved once [`Retry::operation`] is completed.
        #[pin]
        fut: Fut,

        // [`Notify`] implementation to track [`Retry`] ticks.
        notify: N,
    }
}

impl<S, B, N, Fn, Fut, I, E> Retry<S, B, N, Fn, Fut>
where
    S: Sleeper,
    Fn: FnMut() -> Fut,
    Fut: Future<Output = Result<I, Error<E>>>,
{
    pub fn new(sleeper: S, backoff: B, notify: N, mut operation: Fn) -> Self {
        let fut = operation();
        Retry {
            sleeper,
            backoff,
            delay: OptionPinned::None,
            operation,
            fut,
            notify,
        }
    }
}

pin_project! {
    #[project = OptionProj]
    enum OptionPinned<T> {
        Some {
            #[pin]
            inner: T,
        },
        None,
    }
}

impl<S, B, N, Fn, Fut, I, E> Future for Retry<S, B, N, Fn, Fut>
where
    S: Sleeper,
    B: Backoff,
    N: Notify<E>,
    Fn: FnMut() -> Fut,
    Fut: Future<Output = Result<I, Error<E>>>,
{
    type Output = Result<I, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        loop {
            if let OptionProj::Some { inner: delay } = this.delay.as_mut().project() {
                ready!(delay.poll(cx));
                this.delay.set(OptionPinned::None);
            }

            match ready!(this.fut.as_mut().poll(cx)) {
                Ok(v) => return Poll::Ready(Ok(v)),
                Err(Error::Permanent(e)) => return Poll::Ready(Err(e)),
                Err(Error::Transient { err, retry_after }) => {
                    match retry_after.or_else(|| this.backoff.next_backoff()) {
                        Some(duration) => {
                            this.notify.notify(err, duration);
                            this.delay.set(OptionPinned::Some {
                                inner: this.sleeper.sleep(duration),
                            });
                            this.fut.set((this.operation)());
                        }
                        None => return Poll::Ready(Err(err)),
                    }
                }
            }
        }
    }
}

#[cfg(all(feature = "tokio", feature = "async-std"))]
compile_error!("Feature \"tokio\" and \"async-std\" cannot be enabled at the same time");

#[cfg(feature = "async-std")]
fn rt_sleeper() -> impl Sleeper {
    AsyncStdSleeper
}

#[cfg(feature = "tokio")]
fn rt_sleeper() -> impl Sleeper {
    TokioSleeper
}

#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]

struct TokioSleeper;
#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
impl Sleeper for TokioSleeper {
    type Sleep = ::tokio_1::time::Sleep;
    fn sleep(&self, dur: Duration) -> Self::Sleep {
        ::tokio_1::time::sleep(dur)
    }
}

#[cfg(feature = "async-std")]
#[cfg_attr(docsrs, doc(cfg(feature = "async-std")))]
struct AsyncStdSleeper;

#[cfg(feature = "async-std")]
#[cfg_attr(docsrs, doc(cfg(feature = "async-std")))]
impl Sleeper for AsyncStdSleeper {
    type Sleep = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;
    fn sleep(&self, dur: Duration) -> Self::Sleep {
        Box::pin(::async_std_1::task::sleep(dur))
    }
}
