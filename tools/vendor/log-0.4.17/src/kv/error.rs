use std::fmt;

/// An error encountered while working with structured data.
#[derive(Debug)]
pub struct Error {
    inner: Inner,
}

#[derive(Debug)]
enum Inner {
    #[cfg(feature = "std")]
    Boxed(std_support::BoxedError),
    Msg(&'static str),
    Value(value_bag::Error),
    Fmt,
}

impl Error {
    /// Create an error from a message.
    pub fn msg(msg: &'static str) -> Self {
        Error {
            inner: Inner::Msg(msg),
        }
    }

    // Not public so we don't leak the `value_bag` API
    pub(super) fn from_value(err: value_bag::Error) -> Self {
        Error {
            inner: Inner::Value(err),
        }
    }

    // Not public so we don't leak the `value_bag` API
    pub(super) fn into_value(self) -> value_bag::Error {
        match self.inner {
            Inner::Value(err) => err,
            #[cfg(feature = "kv_unstable_std")]
            _ => value_bag::Error::boxed(self),
            #[cfg(not(feature = "kv_unstable_std"))]
            _ => value_bag::Error::msg("error inspecting a value"),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Inner::*;
        match &self.inner {
            #[cfg(feature = "std")]
            &Boxed(ref err) => err.fmt(f),
            &Value(ref err) => err.fmt(f),
            &Msg(ref msg) => msg.fmt(f),
            &Fmt => fmt::Error.fmt(f),
        }
    }
}

impl From<fmt::Error> for Error {
    fn from(_: fmt::Error) -> Self {
        Error { inner: Inner::Fmt }
    }
}

#[cfg(feature = "std")]
mod std_support {
    use super::*;
    use std::{error, io};

    pub(super) type BoxedError = Box<dyn error::Error + Send + Sync>;

    impl Error {
        /// Create an error from a standard error type.
        pub fn boxed<E>(err: E) -> Self
        where
            E: Into<BoxedError>,
        {
            Error {
                inner: Inner::Boxed(err.into()),
            }
        }
    }

    impl error::Error for Error {}

    impl From<io::Error> for Error {
        fn from(err: io::Error) -> Self {
            Error::boxed(err)
        }
    }
}
