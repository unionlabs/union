use core::fmt;

/// An error related to Ed25519 signatures.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Error {
    /// The encoding of a secret key was malformed.
    MalformedSecretKey,
    /// The encoding of a public key was malformed.
    MalformedPublicKey,
    /// Signature verification failed.
    InvalidSignature,
    /// A byte slice of the wrong length was supplied during parsing.
    InvalidSliceLength,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            Self::MalformedSecretKey => "Malformed secret key encoding.",
            Self::MalformedPublicKey => "Malformed public key encoding.",
            Self::InvalidSignature => "Invalid signature.",
            Self::InvalidSliceLength => "Invalid length when parsing byte slice.",
        };

        msg.fmt(f)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
