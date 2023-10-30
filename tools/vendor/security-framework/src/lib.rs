//! Wrappers around the OSX Security Framework.
#![warn(missing_docs)]
#![allow(non_upper_case_globals)]
#![allow(clippy::manual_non_exhaustive)] // MSRV

#[macro_use]
extern crate core_foundation;

use core_foundation_sys::base::OSStatus;
use security_framework_sys::base::errSecSuccess;

use crate::base::{Error, Result};
#[cfg(target_os = "macos")]
use crate::os::macos::access::SecAccess;
#[cfg(target_os = "macos")]
use crate::os::macos::keychain::SecKeychain;

#[cfg(test)]
macro_rules! p {
    ($e:expr) => {
        match $e {
            Ok(s) => s,
            Err(e) => panic!("{:?}", e),
        }
    };
}

#[cfg(all(not(feature = "OSX_10_13"), any(feature = "alpn", feature = "session-tickets")))]
#[macro_use]
mod dlsym;

#[cfg(target_os = "macos")]
pub mod authorization;
pub mod base;
pub mod certificate;
pub mod cipher_suite;
pub mod identity;
pub mod import_export;
pub mod item;
pub mod key;
pub mod os;
pub mod passwords;
pub mod policy;
pub mod random;
pub mod secure_transport;
pub mod trust;
#[cfg(target_os = "macos")]
pub mod trust_settings;

#[cfg(target_os = "macos")]
trait Pkcs12ImportOptionsInternals {
    fn keychain(&mut self, keychain: SecKeychain) -> &mut Self;
    fn access(&mut self, access: SecAccess) -> &mut Self;
}

#[cfg(target_os = "macos")]
trait ItemSearchOptionsInternals {
    fn keychains(&mut self, keychains: &[SecKeychain]) -> &mut Self;
}

trait AsInner {
    type Inner;
    fn as_inner(&self) -> Self::Inner;
}

#[inline(always)]
fn cvt(err: OSStatus) -> Result<()> {
    match err {
        errSecSuccess => Ok(()),
        err => Err(Error::from_code(err)),
    }
}

#[cfg(test)]
mod test {
    use crate::certificate::SecCertificate;

    pub fn certificate() -> SecCertificate {
        let certificate = include_bytes!("../test/server.der");
        p!(SecCertificate::from_der(certificate))
    }
}
