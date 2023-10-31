// Copyright 2016 The rust-url developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This Rust crate implements IDNA
//! [per the WHATWG URL Standard](https://url.spec.whatwg.org/#idna).
//!
//! It also exposes the underlying algorithms from [*Unicode IDNA Compatibility Processing*
//! (Unicode Technical Standard #46)](http://www.unicode.org/reports/tr46/)
//! and [Punycode (RFC 3492)](https://tools.ietf.org/html/rfc3492).
//!
//! Quoting from [UTS #46’s introduction](http://www.unicode.org/reports/tr46/#Introduction):
//!
//! > Initially, domain names were restricted to ASCII characters.
//! > A system was introduced in 2003 for internationalized domain names (IDN).
//! > This system is called Internationalizing Domain Names for Applications,
//! > or IDNA2003 for short.
//! > This mechanism supports IDNs by means of a client software transformation
//! > into a format known as Punycode.
//! > A revision of IDNA was approved in 2010 (IDNA2008).
//! > This revision has a number of incompatibilities with IDNA2003.
//! >
//! > The incompatibilities force implementers of client software,
//! > such as browsers and emailers,
//! > to face difficult choices during the transition period
//! > as registries shift from IDNA2003 to IDNA2008.
//! > This document specifies a mechanism
//! > that minimizes the impact of this transition for client software,
//! > allowing client software to access domains that are valid under either system.

#[macro_use]
extern crate matches;

pub mod punycode;
mod uts46;

pub use crate::uts46::{Config, Errors, Idna};

/// The [domain to ASCII](https://url.spec.whatwg.org/#concept-domain-to-ascii) algorithm.
///
/// Return the ASCII representation a domain name,
/// normalizing characters (upper-case to lower-case and other kinds of equivalence)
/// and using Punycode as necessary.
///
/// This process may fail.
pub fn domain_to_ascii(domain: &str) -> Result<String, uts46::Errors> {
    Config::default().to_ascii(domain)
}

/// The [domain to ASCII](https://url.spec.whatwg.org/#concept-domain-to-ascii) algorithm,
/// with the `beStrict` flag set.
pub fn domain_to_ascii_strict(domain: &str) -> Result<String, uts46::Errors> {
    Config::default()
        .use_std3_ascii_rules(true)
        .verify_dns_length(true)
        .to_ascii(domain)
}

/// The [domain to Unicode](https://url.spec.whatwg.org/#concept-domain-to-unicode) algorithm.
///
/// Return the Unicode representation of a domain name,
/// normalizing characters (upper-case to lower-case and other kinds of equivalence)
/// and decoding Punycode as necessary.
///
/// This may indicate [syntax violations](https://url.spec.whatwg.org/#syntax-violation)
/// but always returns a string for the mapped domain.
pub fn domain_to_unicode(domain: &str) -> (String, Result<(), uts46::Errors>) {
    Config::default().to_unicode(domain)
}
