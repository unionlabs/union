//! Contains error type for handling I/O and Errno errors
#[cfg(windows)]
use std::char;
use std::error;
use std::fmt;
use std::io;

/// The error type for Rustyline errors that can arise from
/// I/O related errors or Errno when using the nix-rust library
// #[non_exhaustive]
#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
#[non_exhaustive]
pub enum ReadlineError {
    /// I/O Error
    Io(io::Error),
    /// EOF (VEOF / Ctrl-D)
    Eof,
    /// Interrupt signal (VINTR / VQUIT / Ctrl-C)
    Interrupted,
    /// Chars Error
    #[cfg(unix)]
    Utf8Error,
    /// Unix Error from syscall
    #[cfg(unix)]
    Errno(nix::Error),
    /// Error generated on WINDOW_BUFFER_SIZE_EVENT to mimic unix SIGWINCH
    /// signal
    #[cfg(windows)]
    WindowResize,
    /// Like Utf8Error on unix
    #[cfg(windows)]
    Decode(char::DecodeUtf16Error),
    /// Something went wrong calling a Windows API
    #[cfg(windows)]
    SystemError(clipboard_win::SystemError),
}

impl fmt::Display for ReadlineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ReadlineError::Io(ref err) => err.fmt(f),
            ReadlineError::Eof => write!(f, "EOF"),
            ReadlineError::Interrupted => write!(f, "Interrupted"),
            #[cfg(unix)]
            ReadlineError::Utf8Error => write!(f, "invalid utf-8: corrupt contents"),
            #[cfg(unix)]
            ReadlineError::Errno(ref err) => err.fmt(f),
            #[cfg(windows)]
            ReadlineError::WindowResize => write!(f, "WindowResize"),
            #[cfg(windows)]
            ReadlineError::Decode(ref err) => err.fmt(f),
            #[cfg(windows)]
            ReadlineError::SystemError(ref err) => err.fmt(f),
        }
    }
}

impl error::Error for ReadlineError {}

impl From<io::Error> for ReadlineError {
    fn from(err: io::Error) -> Self {
        ReadlineError::Io(err)
    }
}

impl From<io::ErrorKind> for ReadlineError {
    fn from(kind: io::ErrorKind) -> Self {
        ReadlineError::Io(io::Error::from(kind))
    }
}

#[cfg(unix)]
impl From<nix::Error> for ReadlineError {
    fn from(err: nix::Error) -> Self {
        ReadlineError::Errno(err)
    }
}

#[cfg(windows)]
impl From<char::DecodeUtf16Error> for ReadlineError {
    fn from(err: char::DecodeUtf16Error) -> Self {
        ReadlineError::Decode(err)
    }
}

#[cfg(windows)]
impl From<clipboard_win::SystemError> for ReadlineError {
    fn from(err: clipboard_win::SystemError) -> Self {
        ReadlineError::SystemError(err)
    }
}
