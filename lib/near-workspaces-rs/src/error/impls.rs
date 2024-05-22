use std::fmt;
use std::{borrow::Cow, sync::PoisonError};

use crate::result::ExecutionFailure;

use super::{Error, ErrorKind, ErrorRepr, RpcErrorCode, SandboxErrorCode};

impl ErrorKind {
    pub(crate) fn custom<E>(self, error: E) -> Error
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Error::custom(self, error)
    }

    pub(crate) fn message<T>(self, msg: T) -> Error
    where
        T: Into<Cow<'static, str>>,
    {
        Error::message(self, msg)
    }

    pub(crate) fn full<T, E>(self, msg: T, error: E) -> Error
    where
        T: Into<Cow<'static, str>>,
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Error::full(self, msg, error)
    }

    pub(crate) fn detailed(self, error: ExecutionFailure) -> Error {
        Error::detailed(self, error)
    }
}

impl Error {
    pub(crate) fn detailed(kind: ErrorKind, error: ExecutionFailure) -> Self {
        Self {
            repr: ErrorRepr::Detailed {
                kind,
                error: Box::new(error),
            },
        }
    }

    /// Construct a workspaces [`Error`] with the full details of an error which includes
    /// the internal error it references, the custom error message with further context
    /// and the [`ErrorKind`] that represents the category of error.
    pub(crate) fn full<T, E>(kind: ErrorKind, msg: T, error: E) -> Self
    where
        T: Into<Cow<'static, str>>,
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Self {
            repr: ErrorRepr::Full {
                kind,
                message: msg.into(),
                error: error.into(),
            },
        }
    }

    /// Construct a workspaces [`Error`] with the details of an error which includes
    /// the internal error it references and the [`ErrorKind`] that represents the
    /// category of error.
    pub fn custom<E>(kind: ErrorKind, error: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Self {
            repr: ErrorRepr::Custom {
                kind,
                error: error.into(),
            },
        }
    }

    /// Construct a workspaces [`Error`] with the details of an error which includes
    /// the custom error message with further context and the [`ErrorKind`] that
    /// represents the category of error.
    pub fn message<T>(kind: ErrorKind, msg: T) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        Self {
            repr: ErrorRepr::Message {
                kind,
                message: msg.into(),
            },
        }
    }

    /// Construct a workspaces [`Error`] with the details of an error which only
    /// includes the [`ErrorKind`] that represents the category of error.
    pub fn simple(kind: ErrorKind) -> Self {
        Self {
            repr: ErrorRepr::Simple(kind),
        }
    }

    /// Returns the corresponding [`ErrorKind`] for this error.
    pub fn kind(&self) -> &ErrorKind {
        match &self.repr {
            ErrorRepr::Simple(kind) => kind,
            ErrorRepr::Message { kind, .. } => kind,
            ErrorRepr::Custom { kind, .. } => kind,
            ErrorRepr::Full { kind, .. } => kind,
            ErrorRepr::Detailed { kind, .. } => kind,
        }
    }

    /// Consumes the `Error`, returning its inner error (if any).
    ///
    /// If this [`Error`] was constructed via a Custom or Full variant, then
    /// this function will return [`Ok`], otherwise it will return [`Err`].
    pub fn into_inner(self) -> Result<Box<dyn std::error::Error + Send + Sync>, Self> {
        match self.repr {
            ErrorRepr::Custom { error, .. } => Ok(error),
            ErrorRepr::Full { error, .. } => Ok(error),
            ErrorRepr::Detailed { error, .. } => Ok(error),
            _ => Err(self),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.repr)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.repr.source()
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(value: PoisonError<T>) -> Self {
        Self::custom(ErrorKind::Other, value.to_string())
    }
}

impl SandboxErrorCode {
    pub(crate) fn message<T>(self, msg: T) -> Error
    where
        T: Into<Cow<'static, str>>,
    {
        Error::message(ErrorKind::Sandbox(self), msg)
    }

    pub(crate) fn custom<E>(self, error: E) -> Error
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Error::custom(ErrorKind::Sandbox(self), error)
    }

    pub(crate) fn full<T, E>(self, msg: T, error: E) -> Error
    where
        T: Into<Cow<'static, str>>,
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Error::full(ErrorKind::Sandbox(self), msg, error)
    }
}

impl From<SandboxErrorCode> for Error {
    fn from(code: SandboxErrorCode) -> Self {
        Self::simple(ErrorKind::Sandbox(code))
    }
}

impl RpcErrorCode {
    pub(crate) fn message<T>(self, msg: T) -> Error
    where
        T: Into<Cow<'static, str>>,
    {
        Error::message(ErrorKind::Rpc(self), msg)
    }

    pub(crate) fn custom<E>(self, error: E) -> Error
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Error::custom(ErrorKind::Rpc(self), error)
    }
}

impl From<RpcErrorCode> for Error {
    fn from(code: RpcErrorCode) -> Self {
        Self::simple(ErrorKind::Rpc(code))
    }
}
