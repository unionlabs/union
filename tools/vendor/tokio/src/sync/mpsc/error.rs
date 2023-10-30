//! Channel error types.

use std::error::Error;
use std::fmt;

/// Error returned by the `Sender`.
#[derive(Debug)]
pub struct SendError<T>(pub T);

impl<T> fmt::Display for SendError<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "channel closed")
    }
}

impl<T: fmt::Debug> std::error::Error for SendError<T> {}

// ===== TrySendError =====

/// This enumeration is the list of the possible error outcomes for the
/// [try_send](super::Sender::try_send) method.
#[derive(Debug, Eq, PartialEq)]
pub enum TrySendError<T> {
    /// The data could not be sent on the channel because the channel is
    /// currently full and sending would require blocking.
    Full(T),

    /// The receive half of the channel was explicitly closed or has been
    /// dropped.
    Closed(T),
}

impl<T: fmt::Debug> Error for TrySendError<T> {}

impl<T> fmt::Display for TrySendError<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            "{}",
            match self {
                TrySendError::Full(..) => "no available capacity",
                TrySendError::Closed(..) => "channel closed",
            }
        )
    }
}

impl<T> From<SendError<T>> for TrySendError<T> {
    fn from(src: SendError<T>) -> TrySendError<T> {
        TrySendError::Closed(src.0)
    }
}

// ===== TryRecvError =====

/// Error returned by `try_recv`.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum TryRecvError {
    /// This **channel** is currently empty, but the **Sender**(s) have not yet
    /// disconnected, so data may yet become available.
    Empty,
    /// The **channel**'s sending half has become disconnected, and there will
    /// never be any more data received on it.
    Disconnected,
}

impl fmt::Display for TryRecvError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            TryRecvError::Empty => "receiving on an empty channel".fmt(fmt),
            TryRecvError::Disconnected => "receiving on a closed channel".fmt(fmt),
        }
    }
}

impl Error for TryRecvError {}

// ===== RecvError =====

/// Error returned by `Receiver`.
#[derive(Debug, Clone)]
#[doc(hidden)]
#[deprecated(note = "This type is unused because recv returns an Option.")]
pub struct RecvError(());

#[allow(deprecated)]
impl fmt::Display for RecvError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "channel closed")
    }
}

#[allow(deprecated)]
impl Error for RecvError {}

cfg_time! {
    // ===== SendTimeoutError =====

    #[derive(Debug, Eq, PartialEq)]
    /// Error returned by [`Sender::send_timeout`](super::Sender::send_timeout)].
    pub enum SendTimeoutError<T> {
        /// The data could not be sent on the channel because the channel is
        /// full, and the timeout to send has elapsed.
        Timeout(T),

        /// The receive half of the channel was explicitly closed or has been
        /// dropped.
        Closed(T),
    }

    impl<T: fmt::Debug> Error for SendTimeoutError<T> {}

    impl<T> fmt::Display for SendTimeoutError<T> {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                fmt,
                "{}",
                match self {
                    SendTimeoutError::Timeout(..) => "timed out waiting on send operation",
                    SendTimeoutError::Closed(..) => "channel closed",
                }
            )
        }
    }
}
