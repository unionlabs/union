//! Verifiable Oblivious Pseudorandom Function (VOPRF) using prime order groups
//!
//! <https://datatracker.ietf.org/doc/draft-irtf-cfrg-voprf/>

use crate::PrimeCurve;

/// Elliptic curve parameters used by VOPRF.
pub trait VoprfParameters: PrimeCurve {
    /// The `ID` parameter which identifies a particular elliptic curve
    /// as defined in [section 4 of `draft-irtf-cfrg-voprf-19`][voprf].
    ///
    /// [voprf]: https://www.ietf.org/archive/id/draft-irtf-cfrg-voprf-19.html#name-ciphersuites-2
    const ID: &'static str;

    /// The `Hash` parameter which assigns a particular hash function to this
    /// ciphersuite as defined in [section 4 of `draft-irtf-cfrg-voprf-19`][voprf].
    ///
    /// [voprf]: https://www.ietf.org/archive/id/draft-irtf-cfrg-voprf-19.html#name-ciphersuites-2
    type Hash: digest::Digest;
}
