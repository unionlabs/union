use std::fmt;
use std::io;
use std::str::Utf8Error;
use std::string::FromUtf8Error;

/// Custom result type for `cargo_metadata::Error`
pub type Result<T> = ::std::result::Result<T, Error>;

/// Error returned when executing/parsing `cargo metadata` fails.
///
/// # Note about Backtraces
///
/// This error type does not contain backtraces, but each error variant
/// comes from _one_ specific place, so it's not really needed for the
/// inside of this crate. If you need a backtrace down to, but not inside
/// of, a failed call of `cargo_metadata` you can do one of multiple thinks:
///
/// 1. Convert it to a `failure::Error` (possible using the `?` operator),
///    which is similar to a `Box<::std::error::Error + 'static + Send  + Sync>`.
/// 2. Have appropriate variants in your own error type. E.g. you could wrap
///    a `failure::Context<Error>` or add a `failure::Backtrace` field (which
///    is empty if `RUST_BACKTRACE` is not set, so it's simple to use).
/// 3. You still can place a failure based error into a `error_chain` if you
///    really want to. (Either through foreign_links or by making it a field
///    value of a `ErrorKind` variant).
///
#[derive(Debug)]
pub enum Error {
    /// Error during execution of `cargo metadata`
    CargoMetadata {
        /// stderr returned by the `cargo metadata` command
        stderr: String,
    },

    /// IO Error during execution of `cargo metadata`
    Io(io::Error),

    /// Output of `cargo metadata` was not valid utf8
    Utf8(Utf8Error),

    /// Error output of `cargo metadata` was not valid utf8
    ErrUtf8(FromUtf8Error),

    /// Deserialization error (structure of json did not match expected structure)
    Json(::serde_json::Error),

    /// The output did not contain any json
    NoJson,
}

impl From<io::Error> for Error {
    fn from(v: io::Error) -> Self {
        Error::Io(v)
    }
}

impl From<Utf8Error> for Error {
    fn from(v: Utf8Error) -> Self {
        Error::Utf8(v)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(v: FromUtf8Error) -> Self {
        Error::ErrUtf8(v)
    }
}

impl From<::serde_json::Error> for Error {
    fn from(v: ::serde_json::Error) -> Self {
        Error::Json(v)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::CargoMetadata { stderr } => {
                write!(
                    f,
                    "`cargo metadata` exited with an error: {}",
                    stderr.trim_end()
                )
            }
            Error::Io(err) => write!(f, "failed to start `cargo metadata`: {}", err),
            Error::Utf8(err) => write!(f, "cannot convert the stdout of `cargo metadata`: {}", err),
            Error::ErrUtf8(err) => {
                write!(f, "cannot convert the stderr of `cargo metadata`: {}", err)
            }
            Error::Json(err) => write!(f, "failed to interpret `cargo metadata`'s json: {}", err),
            Error::NoJson => write!(
                f,
                "could not find any json in the output of `cargo metadata`"
            ),
        }
    }
}

impl ::std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::CargoMetadata { .. } => None,
            Error::Io(err) => Some(err),
            Error::Utf8(err) => Some(err),
            Error::ErrUtf8(err) => Some(err),
            Error::Json(err) => Some(err),
            Error::NoJson => None,
        }
    }
}
