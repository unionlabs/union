//! Argon2 algorithms (e.g. Argon2d, Argon2i, Argon2id).

use crate::{Error, Result};
use core::{
    fmt::{self, Display},
    str::FromStr,
};

#[cfg(feature = "password-hash")]
use password_hash::Ident;

/// Argon2d algorithm identifier
#[cfg(feature = "password-hash")]
#[cfg_attr(docsrs, doc(cfg(feature = "password-hash")))]
pub const ARGON2D_IDENT: Ident<'_> = Ident::new_unwrap("argon2d");

/// Argon2i algorithm identifier
#[cfg(feature = "password-hash")]
#[cfg_attr(docsrs, doc(cfg(feature = "password-hash")))]
pub const ARGON2I_IDENT: Ident<'_> = Ident::new_unwrap("argon2i");

/// Argon2id algorithm identifier
#[cfg(feature = "password-hash")]
#[cfg_attr(docsrs, doc(cfg(feature = "password-hash")))]
pub const ARGON2ID_IDENT: Ident<'_> = Ident::new_unwrap("argon2id");

/// Argon2 primitive type: variants of the algorithm.
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Algorithm {
    /// Optimizes against GPU cracking attacks but vulnerable to side-channels.
    ///
    /// Accesses the memory array in a password dependent order, reducing the
    /// possibility of time–memory tradeoff (TMTO) attacks.
    Argon2d = 0,

    /// Optimized to resist side-channel attacks.
    ///
    /// Accesses the memory array in a password independent order, increasing the
    /// possibility of time-memory tradeoff (TMTO) attacks.
    Argon2i = 1,

    /// Hybrid that mixes Argon2i and Argon2d passes (*default*).
    ///
    /// Uses the Argon2i approach for the first half pass over memory and
    /// Argon2d approach for subsequent passes. This effectively places it in
    /// the "middle" between the other two: it doesn't provide as good
    /// TMTO/GPU cracking resistance as Argon2d, nor as good of side-channel
    /// resistance as Argon2i, but overall provides the most well-rounded
    /// approach to both classes of attacks.
    Argon2id = 2,
}

impl Default for Algorithm {
    fn default() -> Algorithm {
        Algorithm::Argon2id
    }
}

impl Algorithm {
    /// Parse an [`Algorithm`] from the provided string.
    pub fn new(id: impl AsRef<str>) -> Result<Self> {
        id.as_ref().parse()
    }

    /// Get the identifier string for this PBKDF2 [`Algorithm`].
    pub fn as_str(&self) -> &str {
        match self {
            Algorithm::Argon2d => "argon2d",
            Algorithm::Argon2i => "argon2i",
            Algorithm::Argon2id => "argon2id",
        }
    }

    /// Get the [`Ident`] that corresponds to this Argon2 [`Algorithm`].
    #[cfg(feature = "password-hash")]
    #[cfg_attr(docsrs, doc(cfg(feature = "password-hash")))]
    pub fn ident(&self) -> Ident<'static> {
        match self {
            Algorithm::Argon2d => ARGON2D_IDENT,
            Algorithm::Argon2i => ARGON2I_IDENT,
            Algorithm::Argon2id => ARGON2ID_IDENT,
        }
    }

    /// Serialize primitive type as little endian bytes
    pub(crate) fn to_le_bytes(self) -> [u8; 4] {
        (self as u32).to_le_bytes()
    }
}

impl AsRef<str> for Algorithm {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Display for Algorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Algorithm {
    type Err = Error;

    fn from_str(s: &str) -> Result<Algorithm> {
        match s {
            "argon2d" => Ok(Algorithm::Argon2d),
            "argon2i" => Ok(Algorithm::Argon2i),
            "argon2id" => Ok(Algorithm::Argon2id),
            _ => Err(Error::AlgorithmInvalid),
        }
    }
}

#[cfg(feature = "password-hash")]
#[cfg_attr(docsrs, doc(cfg(feature = "password-hash")))]
impl From<Algorithm> for Ident<'static> {
    fn from(alg: Algorithm) -> Ident<'static> {
        alg.ident()
    }
}

#[cfg(feature = "password-hash")]
#[cfg_attr(docsrs, doc(cfg(feature = "password-hash")))]
impl<'a> TryFrom<Ident<'a>> for Algorithm {
    type Error = password_hash::Error;

    fn try_from(ident: Ident<'a>) -> password_hash::Result<Algorithm> {
        match ident {
            ARGON2D_IDENT => Ok(Algorithm::Argon2d),
            ARGON2I_IDENT => Ok(Algorithm::Argon2i),
            ARGON2ID_IDENT => Ok(Algorithm::Argon2id),
            _ => Err(password_hash::Error::Algorithm),
        }
    }
}
