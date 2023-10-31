use std::error;
use std::fmt;

use instant::Duration;

/// Error is the error value in an operation's
/// result.
///
/// Based on the two possible values, the operation
/// may be retried.
pub enum Error<E> {
    /// Permanent means that it's impossible to execute the operation
    /// successfully. This error is immediately returned from `retry()`.
    Permanent(E),

    /// Transient means that the error is temporary. If the `retry_after` is `None`
    /// the operation should be retried according to the backoff policy, else after
    /// the specified duration. Useful for handling ratelimits like a HTTP 429 response.
    Transient {
        err: E,
        retry_after: Option<Duration>,
    },
}

impl<E> Error<E> {
    // Creates an permanent error.
    pub fn permanent(err: E) -> Self {
        Error::Permanent(err)
    }

    // Creates an transient error which is retried according to the backoff
    // policy.
    pub fn transient(err: E) -> Self {
        Error::Transient {
            err,
            retry_after: None,
        }
    }

    /// Creates a transient error which is retried after the specified duration.
    /// Useful for handling ratelimits like a HTTP 429 response.
    pub fn retry_after(err: E, duration: Duration) -> Self {
        Error::Transient {
            err,
            retry_after: Some(duration),
        }
    }
}

impl<E> fmt::Display for Error<E>
where
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Error::Permanent(ref err)
            | Error::Transient {
                ref err,
                retry_after: _,
            } => err.fmt(f),
        }
    }
}

impl<E> fmt::Debug for Error<E>
where
    E: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let (name, err) = match *self {
            Error::Permanent(ref err) => ("Permanent", err as &dyn fmt::Debug),
            Error::Transient {
                ref err,
                retry_after: _,
            } => ("Transient", err as &dyn fmt::Debug),
        };
        f.debug_tuple(name).field(err).finish()
    }
}

impl<E> error::Error for Error<E>
where
    E: error::Error,
{
    fn description(&self) -> &str {
        match *self {
            Error::Permanent(_) => "permanent error",
            Error::Transient { .. } => "transient error",
        }
    }

    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Error::Permanent(ref err)
            | Error::Transient {
                ref err,
                retry_after: _,
            } => err.source(),
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        self.source()
    }
}

/// By default all errors are transient. Permanent errors can
/// be constructed explicitly. This implementation is for making
/// the question mark operator (?) and the `try!` macro to work.
impl<E> From<E> for Error<E> {
    fn from(err: E) -> Error<E> {
        Error::Transient {
            err,
            retry_after: None,
        }
    }
}

impl<E> PartialEq for Error<E>
where
    E: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Error::Permanent(ref self_err), Error::Permanent(ref other_err)) => {
                self_err == other_err
            }
            (
                Error::Transient {
                    err: self_err,
                    retry_after: self_retry_after,
                },
                Error::Transient {
                    err: other_err,
                    retry_after: other_retry_after,
                },
            ) => self_err == other_err && self_retry_after == other_retry_after,
            _ => false,
        }
    }
}

#[test]
fn create_permanent_error() {
    let e = Error::permanent("err");
    assert_eq!(e, Error::Permanent("err"));
}

#[test]
fn create_transient_error() {
    let e = Error::transient("err");
    assert_eq!(
        e,
        Error::Transient {
            err: "err",
            retry_after: None
        }
    );
}

#[test]
fn create_transient_error_with_retry_after() {
    let retry_after = Duration::from_secs(42);
    let e = Error::retry_after("err", retry_after);
    assert_eq!(
        e,
        Error::Transient {
            err: "err",
            retry_after: Some(retry_after),
        }
    );
}
