// Copyright 2015 Brian Smith.
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHORS DISCLAIM ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR
// ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
// ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
// OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

//! webpki: Web PKI X.509 Certificate Validation.
//!
//! See `EndEntityCert`'s documentation for a description of the certificate
//! processing steps necessary for a TLS connection.
//!
//! # Features
//!
//! | Feature | Description |
//! | ------- | ----------- |
//! | `alloc` | Enable features that require use of the heap. Currently all RSA signature algorithms require this feature. |
//! | `std` | Enable features that require libstd. Implies `alloc`. |

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(unreachable_pub)]
#![deny(warnings, missing_docs, clippy::as_conversions)]
#![allow(
    clippy::len_without_is_empty,
    clippy::new_without_default,
    clippy::single_match,
    clippy::single_match_else,
    clippy::type_complexity,
    clippy::upper_case_acronyms
)]

#[cfg(any(test, feature = "alloc"))]
#[cfg_attr(test, macro_use)]
extern crate alloc;

#[macro_use]
mod der;

mod calendar;
mod cert;
mod end_entity;
mod error;
mod signed_data;
mod subject_name;
mod time;
mod trust_anchor;

mod crl;
mod verify_cert;
mod x509;

pub use {
    cert::{Cert, EndEntityOrCa},
    crl::{BorrowedCertRevocationList, BorrowedRevokedCert, CertRevocationList, RevocationReason},
    end_entity::EndEntityCert,
    error::Error,
    signed_data::{
        SignatureAlgorithm, ECDSA_P256_SHA256, ECDSA_P256_SHA384, ECDSA_P384_SHA256,
        ECDSA_P384_SHA384, ED25519,
    },
    time::Time,
    trust_anchor::{TlsClientTrustAnchors, TlsServerTrustAnchors, TrustAnchor},
};

// TODO(XXX): An interaction between Rust 1.57 and clippy requires working around 'unreachable_pub'
//            false positives by breaking out these 'pub use' statements individually. Once MSRV
//            increases they can be collapsed into one as part of the larger 'pub use' above.
pub use subject_name::AddrParseError;
pub use subject_name::DnsNameRef;
pub use subject_name::InvalidDnsNameError;
pub use subject_name::InvalidSubjectNameError;
pub use subject_name::IpAddrRef;
pub use subject_name::SubjectNameRef;

#[cfg(feature = "alloc")]
pub use {
    crl::{OwnedCertRevocationList, OwnedRevokedCert},
    signed_data::{
        RSA_PKCS1_2048_8192_SHA256, RSA_PKCS1_2048_8192_SHA384, RSA_PKCS1_2048_8192_SHA512,
        RSA_PKCS1_3072_8192_SHA384, RSA_PSS_2048_8192_SHA256_LEGACY_KEY,
        RSA_PSS_2048_8192_SHA384_LEGACY_KEY, RSA_PSS_2048_8192_SHA512_LEGACY_KEY,
    },
    subject_name::IpAddr,
};

// TODO(XXX): Similar to above, we break this individual 'pub use' for Rust 1.57.
#[cfg(feature = "alloc")]
pub use subject_name::DnsName;
