use std::thread;
use std::time::Duration;

use crate::backoff::Backoff;
use crate::error::Error;

/// Retries this operation according to the backoff policy.
/// backoff is reset before it is used.
///
/// # Examples
///
/// ```rust
/// # use backoff::{ExponentialBackoff, Error, retry};
/// let f = || -> Result<(), Error<&str>> {
///     // Business logic...
///     // Give up.
///     Err(Error::Permanent("error"))
/// };
///
/// let backoff = ExponentialBackoff::default();
/// let _ = retry(backoff, f).err().unwrap();
/// ```
pub fn retry<F, B, T, E>(backoff: B, op: F) -> Result<T, Error<E>>
where
    F: FnMut() -> Result<T, Error<E>>,
    B: Backoff,
{
    let mut retry = Retry {
        backoff,
        notify: NoopNotify,
        sleep: ThreadSleep,
    };

    retry.retry_notify(op)
}

/// Retries this operation according to the backoff policy.
/// Calls notify on failed attempts (in case of transient errors).
/// backoff is reset before it is used.
///
/// # Examples
///
/// ```rust
/// # use backoff::{Error, retry_notify};
/// # use backoff::backoff::Stop;
/// # use std::time::Duration;
/// let notify = |err, dur| { println!("Error happened at {:?}: {}", dur, err); };
/// let f = || -> Result<(), Error<&str>> {
///     // Business logic...
///     Err(Error::transient("error"))
/// };
///
/// let backoff = Stop{};
/// let _ = retry_notify(backoff, f, notify).err().unwrap();
/// ```
pub fn retry_notify<F, B, N, T, E>(backoff: B, op: F, notify: N) -> Result<T, Error<E>>
where
    F: FnMut() -> Result<T, Error<E>>,
    B: Backoff,
    N: Notify<E>,
{
    let mut retry = Retry {
        backoff,
        notify,
        sleep: ThreadSleep,
    };

    retry.retry_notify(op)
}

struct Retry<B, N, S> {
    backoff: B,
    notify: N,
    sleep: S,
}

impl<B, N, S> Retry<B, N, S> {
    pub fn retry_notify<F, T, E>(&mut self, mut op: F) -> Result<T, Error<E>>
    where
        F: FnMut() -> Result<T, Error<E>>,
        B: Backoff,
        N: Notify<E>,
        S: Sleep,
    {
        self.backoff.reset();

        loop {
            let err = match op() {
                Ok(v) => return Ok(v),
                Err(err) => err,
            };

            let (err, next) = match err {
                Error::Permanent(err) => return Err(Error::Permanent(err)),
                Error::Transient { err, retry_after } => {
                    match retry_after.or_else(|| self.backoff.next_backoff()) {
                        Some(next) => (err, next),
                        None => return Err(Error::transient(err)),
                    }
                }
            };

            self.notify.notify(err, next);

            self.sleep.sleep(next);
        }
    }
}

trait Sleep {
    fn sleep(&mut self, dur: Duration);
}

struct ThreadSleep;

impl Sleep for ThreadSleep {
    fn sleep(&mut self, dur: Duration) {
        thread::sleep(dur);
    }
}

/// Notify is called in [`retry_notify`](trait.Operation.html#method.retry_notify) in case of errors.
pub trait Notify<E> {
    fn notify(&mut self, err: E, duration: Duration);
}

impl<E, F> Notify<E> for F
where
    F: FnMut(E, Duration),
{
    fn notify(&mut self, err: E, duration: Duration) {
        self(err, duration)
    }
}

/// No-op implementation of [`Notify`]. Literally does nothing.
#[derive(Debug, Clone, Copy)]
pub struct NoopNotify;

impl<E> Notify<E> for NoopNotify {
    fn notify(&mut self, _: E, _: Duration) {}
}
