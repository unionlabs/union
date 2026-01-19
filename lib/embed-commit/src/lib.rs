#![no_std]

use core::str::FromStr;

use bytemuck::CheckedBitPattern;

/// The git rev of the code, as supplied at build time. On `wasm32` targets, this is available via the `commit_hash` export.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, CheckedBitPattern)]
#[repr(u64)]
#[rustfmt::skip]
pub enum Rev {
    /// The state of the build is unknown (i.e. `GIT_REV` was not set).
    //                  U  N  K  N  O  W  N
    #[default]
    Unknown        = 0x_75_6E_6B_6E_6F_77_6E,
    /// The build is dirty.
    ///
    /// `GIT_REV=dirty`
    //                  D  I  R  T  Y
    Dirty          = 0x_64_69_72_74_79,
    /// The build was done on the specified commit hash.
    ///
    /// `GIT_REV=6e65766572676f6e6e6167697665796f75757020`
    //                  H  A  S  H
    Hash([u8; 20]) = 0x_68_61_73_68,
}

impl Rev {
    /// Returns `true` if the rev is [`Unknown`].
    ///
    /// [`Unknown`]: Rev::Unknown
    #[must_use]
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }

    /// Returns `true` if the rev is [`Dirty`].
    ///
    /// [`Dirty`]: Rev::Dirty
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        matches!(self, Self::Dirty)
    }

    /// Returns `true` if the rev is [`Hash`].
    ///
    /// [`Hash`]: Rev::Hash
    #[must_use]
    pub fn is_hash(&self) -> bool {
        matches!(self, Self::Hash(..))
    }
}

impl core::fmt::Debug for Rev {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Unknown => write!(f, "Unknown"),
            Self::Dirty => write!(f, "Dirty"),
            Self::Hash(_) => write!(f, "Hash({self})"),
        }
    }
}

impl core::fmt::Display for Rev {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Rev::Unknown => f.write_str("unknown"),
            Rev::Dirty => f.write_str("dirty"),
            Rev::Hash(hash) => const_hex::Buffer::<20, false>::new().format(hash).fmt(f),
        }
    }
}

impl FromStr for Rev {
    type Err = const_hex::FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "unknown" => Ok(Self::Unknown),
            "dirty" => Ok(Self::Dirty),
            hash => const_hex::decode_to_array(hash).map(Self::Hash),
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Rev {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Rev {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        <&str>::deserialize(deserializer).and_then(|s| s.parse().map_err(serde::de::Error::custom))
    }
}

#[cfg_attr(
    not(target_arch = "wasm32"),
    unsafe(no_mangle),
    used,
    unsafe(link_section = ".note.embed_commit.GIT_REV")
)]
pub static GIT_REV: Rev = match option_env!("GIT_REV") {
    None => Rev::Unknown,
    Some(hash) => match hash.as_bytes() {
        b"dirty" => Rev::Dirty,
        hash => Rev::Hash(match const_hex::const_decode_to_array(hash) {
            Ok(ok) => ok,
            Err(_) => panic!(
                "invalid GIT_REV env var, value must be either \
                unset, \"dirty\" or a 20-byte hex string (commit)"
            ),
        }),
    },
};

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn commit_hash() -> Rev {
    unsafe { core::ptr::read_volatile(&GIT_REV as *const _) }
}

#[test]
fn size() {
    assert_eq!(size_of::<Rev>(), 32);
}

#[test]
fn bytes() {
    assert_eq!(
        bytemuck::checked::from_bytes::<Rev>(
            &const_hex::decode_to_array::<_, 32>(
                "6E776F6E6B6E7500000000000000000000000000000000000000000000000000"
            )
            .unwrap()
        ),
        &Rev::Unknown
    );

    assert_eq!(
        bytemuck::checked::from_bytes::<Rev>(
            &const_hex::decode_to_array::<_, 32>(
                "7974726964000000000000000000000000000000000000000000000000000000"
            )
            .unwrap()
        ),
        &Rev::Dirty
    );

    assert_eq!(
        bytemuck::checked::from_bytes::<Rev>(
            &const_hex::decode_to_array::<_, 32>(
                "6873616800000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF00000000"
            )
            .unwrap()
        ),
        &Rev::Hash([0xff; 20])
    );
}
