//! Error type

use core::fmt;

#[cfg(feature = "password-hash")]
use password_hash::errors::InvalidValue;

/// Result with argon2's [`Error`] type.
pub type Result<T> = core::result::Result<T, Error>;

/// Error type.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// Associated data is too long.
    AdTooLong,

    /// Algorithm identifier invalid.
    AlgorithmInvalid,

    /// "B64" encoding is invalid.
    B64Encoding(base64ct::Error),

    /// Key ID is too long.
    KeyIdTooLong,

    /// Memory cost is too small.
    MemoryTooLittle,

    /// Memory cost is too large.
    MemoryTooMuch,

    /// Output is too short.
    OutputTooShort,

    /// Output is too long.
    OutputTooLong,

    /// Password is too long.
    PwdTooLong,

    /// Salt is too short.
    SaltTooShort,

    /// Salt is too long.
    SaltTooLong,

    /// Secret is too long.
    SecretTooLong,

    /// Not enough threads.
    ThreadsTooFew,

    /// Too many threads.
    ThreadsTooMany,

    /// Time cost is too small.
    TimeTooSmall,

    /// Invalid version
    VersionInvalid,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Error::AdTooLong => "associated data is too long",
            Error::AlgorithmInvalid => "algorithm identifier invalid",
            Error::B64Encoding(inner) => return write!(f, "B64 encoding invalid: {}", inner),
            Error::KeyIdTooLong => "key ID is too long",
            Error::MemoryTooLittle => "memory cost is too small",
            Error::MemoryTooMuch => "memory cost is too large",
            Error::OutputTooShort => "output is too short",
            Error::OutputTooLong => "output is too long",
            Error::PwdTooLong => "password is too long",
            Error::SaltTooShort => "salt is too short",
            Error::SaltTooLong => "salt is too long",
            Error::SecretTooLong => "secret is too long",
            Error::ThreadsTooFew => "not enough threads",
            Error::ThreadsTooMany => "too many threads",
            Error::TimeTooSmall => "time cost is too small",
            Error::VersionInvalid => "invalid version",
        })
    }
}

impl From<base64ct::Error> for Error {
    fn from(err: base64ct::Error) -> Error {
        Error::B64Encoding(err)
    }
}

#[cfg(feature = "password-hash")]
#[cfg_attr(docsrs, doc(cfg(feature = "password-hash")))]
impl From<Error> for password_hash::Error {
    fn from(err: Error) -> password_hash::Error {
        match err {
            Error::AdTooLong => InvalidValue::TooLong.param_error(),
            Error::AlgorithmInvalid => password_hash::Error::Algorithm,
            Error::B64Encoding(inner) => password_hash::Error::B64Encoding(inner),
            Error::KeyIdTooLong => InvalidValue::TooLong.param_error(),
            Error::MemoryTooLittle => InvalidValue::TooShort.param_error(),
            Error::MemoryTooMuch => InvalidValue::TooLong.param_error(),
            Error::PwdTooLong => password_hash::Error::Password,
            Error::OutputTooShort => password_hash::Error::OutputTooShort,
            Error::OutputTooLong => password_hash::Error::OutputTooLong,
            Error::SaltTooShort => InvalidValue::TooShort.salt_error(),
            Error::SaltTooLong => InvalidValue::TooLong.salt_error(),
            Error::SecretTooLong => InvalidValue::TooLong.param_error(),
            Error::ThreadsTooFew => InvalidValue::TooShort.param_error(),
            Error::ThreadsTooMany => InvalidValue::TooLong.param_error(),
            Error::TimeTooSmall => InvalidValue::TooShort.param_error(),
            Error::VersionInvalid => password_hash::Error::Version,
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
