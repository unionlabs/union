// Copyright 2016 Brian Smith.
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHORS DISCLAIM ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY
// SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION
// OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN
// CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

//! Error reporting.

#[cfg(feature = "std")]
extern crate std;

/// An error with absolutely no details.
///
/// *ring* uses this unit type as the error type in most of its results
/// because (a) usually the specific reasons for a failure are obvious or are
/// not useful to know, and/or (b) providing more details about a failure might
/// provide a dangerous side channel, and/or (c) it greatly simplifies the
/// error handling logic.
///
/// `Result<T, ring::error::Unspecified>` is mostly equivalent to
/// `Result<T, ()>`. However, `ring::error::Unspecified` implements
/// [`std::error::Error`] and users of *ring* can implement
/// `From<ring::error::Unspecified>` to map this to their own error types, as
/// described in [“Error Handling” in the Rust Book]:
///
/// ```
/// use ring::rand::{self, SecureRandom};
///
/// enum Error {
///     CryptoError,
///
/// #  #[cfg(feature = "alloc")]
///     IOError(std::io::Error),
///     // [...]
/// }
///
/// impl From<ring::error::Unspecified> for Error {
///     fn from(_: ring::error::Unspecified) -> Self { Error::CryptoError }
/// }
///
/// fn eight_random_bytes() -> Result<[u8; 8], Error> {
///     let rng = rand::SystemRandom::new();
///     let mut bytes = [0; 8];
///
///     // The `From<ring::error::Unspecified>` implementation above makes this
///     // equivalent to
///     // `rng.fill(&mut bytes).map_err(|_| Error::CryptoError)?`.
///     rng.fill(&mut bytes)?;
///
///     Ok(bytes)
/// }
///
/// assert!(eight_random_bytes().is_ok());
/// ```
///
/// Experience with using and implementing other crypto libraries like has
/// shown that sophisticated error reporting facilities often cause significant
/// bugs themselves, both within the crypto library and within users of the
/// crypto library. This approach attempts to minimize complexity in the hopes
/// of avoiding such problems. In some cases, this approach may be too extreme,
/// and it may be important for an operation to provide some details about the
/// cause of a failure. Users of *ring* are encouraged to report such cases so
/// that they can be addressed individually.
///
/// [`std::error::Error`]: https://doc.rust-lang.org/std/error/trait.Error.html
/// [“Error Handling” in the Rust Book]:
///     https://doc.rust-lang.org/book/first-edition/error-handling.html#the-from-trait
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Unspecified;

impl Unspecified {
    fn description_() -> &'static str {
        "ring::error::Unspecified"
    }
}

// This is required for the implementation of `std::error::Error`.
impl core::fmt::Display for Unspecified {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str(Self::description_())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Unspecified {
    #[inline]
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }

    fn description(&self) -> &str {
        Self::description_()
    }
}

impl From<untrusted::EndOfInput> for Unspecified {
    fn from(_: untrusted::EndOfInput) -> Self {
        Unspecified
    }
}

impl From<core::array::TryFromSliceError> for Unspecified {
    fn from(_: core::array::TryFromSliceError) -> Self {
        Unspecified
    }
}

/// An error parsing or validating a key.
///
/// The `Display` implementation and `<KeyRejected as Error>::description()`
/// will return a string that will help you better understand why a key was
/// rejected change which errors are reported in which situations while
/// minimizing the likelihood that any applications will be broken.
///
/// Here is an incomplete list of reasons a key may be unsupported:
///
/// * Invalid or Inconsistent Components: A component of the key has an invalid
///   value, or the mathematical relationship between two (or more) components
///   required for a valid key does not hold.
///
/// * The encoding of the key is invalid. Perhaps the key isn't in the correct
///   format; e.g. it may be Base64 ("PEM") encoded, in which case   the Base64
///   encoding needs to be undone first.
///
/// * The encoding includes a versioning mechanism and that mechanism indicates
///   that the key is encoded in a version of the encoding that isn't supported.
///   This might happen for multi-prime RSA keys (keys with more than two
///   private   prime factors), which aren't supported, for example.
///
/// * Too small or too Large: One of the primary components of the key is too
///   small or two large. Too-small keys are rejected for security reasons. Some
///   unnecessarily large keys are rejected for performance reasons.
///
///  * Wrong algorithm: The key is not valid for the algorithm in which it was
///    being used.
///
///  * Unexpected errors: Report this as a bug.
#[derive(Copy, Clone, Debug)]
pub struct KeyRejected(&'static str);

impl KeyRejected {
    /// The value returned from <Self as std::error::Error>::description()
    pub fn description_(&self) -> &'static str {
        self.0
    }

    pub(crate) fn inconsistent_components() -> Self {
        KeyRejected("InconsistentComponents")
    }

    pub(crate) fn invalid_component() -> Self {
        KeyRejected("InvalidComponent")
    }

    #[inline]
    pub(crate) fn invalid_encoding() -> Self {
        KeyRejected("InvalidEncoding")
    }

    // XXX: See the comment at the call site.
    pub(crate) fn rng_failed() -> Self {
        KeyRejected("RNG failed")
    }

    pub(crate) fn public_key_is_missing() -> Self {
        KeyRejected("PublicKeyIsMissing")
    }

    #[cfg(feature = "alloc")]
    pub(crate) fn too_small() -> Self {
        KeyRejected("TooSmall")
    }

    #[cfg(feature = "alloc")]
    pub(crate) fn too_large() -> Self {
        KeyRejected("TooLarge")
    }

    pub(crate) fn version_not_supported() -> Self {
        KeyRejected("VersionNotSupported")
    }

    pub(crate) fn wrong_algorithm() -> Self {
        KeyRejected("WrongAlgorithm")
    }

    #[cfg(feature = "alloc")]
    pub(crate) fn private_modulus_len_not_multiple_of_512_bits() -> Self {
        KeyRejected("PrivateModulusLenNotMultipleOf512Bits")
    }

    pub(crate) fn unexpected_error() -> Self {
        KeyRejected("UnexpectedError")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for KeyRejected {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }

    fn description(&self) -> &str {
        self.description_()
    }
}

impl core::fmt::Display for KeyRejected {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str(self.description_())
    }
}

impl From<KeyRejected> for Unspecified {
    fn from(_: KeyRejected) -> Self {
        Unspecified
    }
}
