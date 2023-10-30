//! The message digest context.
//!
//! # Examples
//!
//! Compute the SHA256 checksum of data
//!
//! ```
//! use openssl::md::Md;
//! use openssl::md_ctx::MdCtx;
//!
//! let mut ctx = MdCtx::new().unwrap();
//! ctx.digest_init(Md::sha256()).unwrap();
//! ctx.digest_update(b"Some Crypto Text").unwrap();
//! let mut digest = [0; 32];
//! ctx.digest_final(&mut digest).unwrap();
//!
//! assert_eq!(
//!     digest,
//!     *b"\x60\x78\x56\x38\x8a\xca\x5c\x51\x83\xc4\xd1\x4d\xc8\xf9\xcc\xf2\
//!        \xa5\x21\xb3\x10\x93\x72\xfa\xd6\x7c\x55\xf5\xc9\xe3\xd1\x83\x19",
//! );
//! ```
//!
//! Sign and verify data with RSA and SHA256
//!
//! ```
//! use openssl::md::Md;
//! use openssl::md_ctx::MdCtx;
//! use openssl::pkey::PKey;
//! use openssl::rsa::Rsa;
//!
//! // Generate a random RSA key.
//! let key = Rsa::generate(4096).unwrap();
//! let key = PKey::from_rsa(key).unwrap();
//!
//! let text = b"Some Crypto Text";
//!
//! // Create the signature.
//! let mut ctx = MdCtx::new().unwrap();
//! ctx.digest_sign_init(Some(Md::sha256()), &key).unwrap();
//! ctx.digest_sign_update(text).unwrap();
//! let mut signature = vec![];
//! ctx.digest_sign_final_to_vec(&mut signature).unwrap();
//!
//! // Verify the signature.
//! let mut ctx = MdCtx::new().unwrap();
//! ctx.digest_verify_init(Some(Md::sha256()), &key).unwrap();
//! ctx.digest_verify_update(text).unwrap();
//! let valid = ctx.digest_verify_final(&signature).unwrap();
//! assert!(valid);
//! ```
//!
//! Compute and verify an HMAC-SHA256
//!
//! ```
//! use openssl::md::Md;
//! use openssl::md_ctx::MdCtx;
//! use openssl::memcmp;
//! use openssl::pkey::PKey;
//!
//! // Create a key with the HMAC secret.
//! let key = PKey::hmac(b"my secret").unwrap();
//!
//! let text = b"Some Crypto Text";
//!
//! // Compute the HMAC.
//! let mut ctx = MdCtx::new().unwrap();
//! ctx.digest_sign_init(Some(Md::sha256()), &key).unwrap();
//! ctx.digest_sign_update(text).unwrap();
//! let mut hmac = vec![];
//! ctx.digest_sign_final_to_vec(&mut hmac).unwrap();
//!
//! // Verify the HMAC. You can't use MdCtx to do this; instead use a constant time equality check.
//! # let target = hmac.clone();
//! let valid = memcmp::eq(&hmac, &target);
//! assert!(valid);
//! ```
use crate::error::ErrorStack;
use crate::md::MdRef;
use crate::pkey::{HasPrivate, PKeyRef};
use crate::pkey_ctx::PkeyCtxRef;
use crate::{cvt, cvt_n, cvt_p};
use cfg_if::cfg_if;
use foreign_types::{ForeignType, ForeignTypeRef};
use openssl_macros::corresponds;
use std::convert::TryFrom;
use std::ptr;

cfg_if! {
    if #[cfg(ossl110)] {
        use ffi::{EVP_MD_CTX_free, EVP_MD_CTX_new};
    } else {
        use ffi::{EVP_MD_CTX_create as EVP_MD_CTX_new, EVP_MD_CTX_destroy as EVP_MD_CTX_free};
    }
}

foreign_type_and_impl_send_sync! {
    type CType = ffi::EVP_MD_CTX;
    fn drop = EVP_MD_CTX_free;

    pub struct MdCtx;
    /// A reference to an [`MdCtx`].
    pub struct MdCtxRef;
}

impl MdCtx {
    /// Creates a new context.
    #[corresponds(EVP_MD_CTX_new)]
    #[inline]
    pub fn new() -> Result<Self, ErrorStack> {
        ffi::init();

        unsafe {
            let ptr = cvt_p(EVP_MD_CTX_new())?;
            Ok(MdCtx::from_ptr(ptr))
        }
    }
}

impl MdCtxRef {
    /// Initializes the context to compute the digest of data.
    #[corresponds(EVP_DigestInit_ex)]
    #[inline]
    pub fn digest_init(&mut self, digest: &MdRef) -> Result<(), ErrorStack> {
        unsafe {
            cvt(ffi::EVP_DigestInit_ex(
                self.as_ptr(),
                digest.as_ptr(),
                ptr::null_mut(),
            ))?;
        }

        Ok(())
    }

    /// Initializes the context to compute the signature of data.
    ///
    /// A reference to the context's inner `PkeyCtx` is returned, allowing signature settings to be configured.
    #[corresponds(EVP_DigestSignInit)]
    #[inline]
    pub fn digest_sign_init<'a, T>(
        &'a mut self,
        digest: Option<&MdRef>,
        pkey: &PKeyRef<T>,
    ) -> Result<&'a mut PkeyCtxRef<T>, ErrorStack>
    where
        T: HasPrivate,
    {
        unsafe {
            let mut p = ptr::null_mut();
            cvt(ffi::EVP_DigestSignInit(
                self.as_ptr(),
                &mut p,
                digest.map_or(ptr::null(), |p| p.as_ptr()),
                ptr::null_mut(),
                pkey.as_ptr(),
            ))?;
            Ok(PkeyCtxRef::from_ptr_mut(p))
        }
    }

    /// Initializes the context to verify the signature of data.
    ///
    /// A reference to the context's inner `PkeyCtx` is returned, allowing signature settings to be configured.
    #[corresponds(EVP_DigestVerifyInit)]
    #[inline]
    pub fn digest_verify_init<'a, T>(
        &'a mut self,
        digest: Option<&MdRef>,
        pkey: &PKeyRef<T>,
    ) -> Result<&'a mut PkeyCtxRef<T>, ErrorStack>
    where
        T: HasPrivate,
    {
        unsafe {
            let mut p = ptr::null_mut();
            cvt(ffi::EVP_DigestVerifyInit(
                self.as_ptr(),
                &mut p,
                digest.map_or(ptr::null(), |p| p.as_ptr()),
                ptr::null_mut(),
                pkey.as_ptr(),
            ))?;
            Ok(PkeyCtxRef::from_ptr_mut(p))
        }
    }

    /// Updates the context with more data.
    #[corresponds(EVP_DigestUpdate)]
    #[inline]
    pub fn digest_update(&mut self, data: &[u8]) -> Result<(), ErrorStack> {
        unsafe {
            cvt(ffi::EVP_DigestUpdate(
                self.as_ptr(),
                data.as_ptr() as *const _,
                data.len(),
            ))?;
        }

        Ok(())
    }

    /// Updates the context with more data.
    #[corresponds(EVP_DigestSignUpdate)]
    #[inline]
    pub fn digest_sign_update(&mut self, data: &[u8]) -> Result<(), ErrorStack> {
        unsafe {
            cvt(ffi::EVP_DigestSignUpdate(
                self.as_ptr(),
                data.as_ptr() as *const _,
                data.len(),
            ))?;
        }

        Ok(())
    }

    /// Updates the context with more data.
    #[corresponds(EVP_DigestVerifyUpdate)]
    #[inline]
    pub fn digest_verify_update(&mut self, data: &[u8]) -> Result<(), ErrorStack> {
        unsafe {
            cvt(ffi::EVP_DigestVerifyUpdate(
                self.as_ptr(),
                data.as_ptr() as *const _,
                data.len(),
            ))?;
        }

        Ok(())
    }

    /// Copies the computed digest into the buffer, returning the number of bytes written.
    #[corresponds(EVP_DigestFinal)]
    #[inline]
    pub fn digest_final(&mut self, out: &mut [u8]) -> Result<usize, ErrorStack> {
        let mut len = u32::try_from(out.len()).unwrap_or(u32::MAX);

        unsafe {
            cvt(ffi::EVP_DigestFinal(
                self.as_ptr(),
                out.as_mut_ptr(),
                &mut len,
            ))?;
        }

        Ok(len as usize)
    }

    /// Copies the computed digest into the buffer.
    ///
    /// Requires OpenSSL 1.1.1 or newer.
    #[corresponds(EVP_DigestFinalXOF)]
    #[inline]
    #[cfg(ossl111)]
    pub fn digest_final_xof(&mut self, out: &mut [u8]) -> Result<(), ErrorStack> {
        unsafe {
            cvt(ffi::EVP_DigestFinalXOF(
                self.as_ptr(),
                out.as_mut_ptr(),
                out.len(),
            ))?;
        }

        Ok(())
    }

    /// Signs the computed digest.
    ///
    /// If `out` is set to `None`, an upper bound on the number of bytes required for the output buffer will be
    /// returned.
    #[corresponds(EVP_DigestSignFinal)]
    #[inline]
    pub fn digest_sign_final(&mut self, out: Option<&mut [u8]>) -> Result<usize, ErrorStack> {
        let mut len = out.as_ref().map_or(0, |b| b.len());

        unsafe {
            cvt(ffi::EVP_DigestSignFinal(
                self.as_ptr(),
                out.map_or(ptr::null_mut(), |b| b.as_mut_ptr()),
                &mut len,
            ))?;
        }

        Ok(len)
    }

    /// Like [`Self::digest_sign_final`] but appends the signature to a [`Vec`].
    pub fn digest_sign_final_to_vec(&mut self, out: &mut Vec<u8>) -> Result<usize, ErrorStack> {
        let base = out.len();
        let len = self.digest_sign_final(None)?;
        out.resize(base + len, 0);
        let len = self.digest_sign_final(Some(&mut out[base..]))?;
        out.truncate(base + len);
        Ok(len)
    }

    /// Verifies the provided signature.
    ///
    /// Returns `Ok(true)` if the signature is valid, `Ok(false)` if the signature is invalid, and `Err` if an error
    /// occurred.
    #[corresponds(EVP_DigestVerifyFinal)]
    #[inline]
    pub fn digest_verify_final(&mut self, signature: &[u8]) -> Result<bool, ErrorStack> {
        unsafe {
            let r = cvt_n(ffi::EVP_DigestVerifyFinal(
                self.as_ptr(),
                signature.as_ptr() as *mut _,
                signature.len(),
            ))?;
            Ok(r == 1)
        }
    }

    /// Computes the signature of the data in `from`.
    ///
    /// If `to` is set to `None`, an upper bound on the number of bytes required for the output buffer will be
    /// returned.
    ///
    /// Requires OpenSSL 1.1.1 or newer.
    #[corresponds(EVP_DigestSign)]
    #[cfg(ossl111)]
    #[inline]
    pub fn digest_sign(&mut self, from: &[u8], to: Option<&mut [u8]>) -> Result<usize, ErrorStack> {
        let mut len = to.as_ref().map_or(0, |b| b.len());

        unsafe {
            cvt(ffi::EVP_DigestSign(
                self.as_ptr(),
                to.map_or(ptr::null_mut(), |b| b.as_mut_ptr()),
                &mut len,
                from.as_ptr(),
                from.len(),
            ))?;
        }

        Ok(len)
    }

    /// Like [`Self::digest_sign`] but appends the signature to a [`Vec`].
    #[cfg(ossl111)]
    pub fn digest_sign_to_vec(
        &mut self,
        from: &[u8],
        to: &mut Vec<u8>,
    ) -> Result<usize, ErrorStack> {
        let base = to.len();
        let len = self.digest_sign(from, None)?;
        to.resize(base + len, 0);
        let len = self.digest_sign(from, Some(&mut to[base..]))?;
        to.truncate(base + len);
        Ok(len)
    }

    /// Verifies the signature of the data in `data`.
    ///
    /// Returns `Ok(true)` if the signature is valid, `Ok(false)` if the signature is invalid, and `Err` if an error
    /// occurred.
    ///
    /// Requires OpenSSL 1.1.1 or newer.
    #[corresponds(EVP_DigestVerify)]
    #[cfg(ossl111)]
    #[inline]
    pub fn digest_verify(&mut self, data: &[u8], signature: &[u8]) -> Result<bool, ErrorStack> {
        unsafe {
            let r = cvt(ffi::EVP_DigestVerify(
                self.as_ptr(),
                signature.as_ptr(),
                signature.len(),
                data.as_ptr(),
                data.len(),
            ))?;
            Ok(r == 1)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::md::Md;
    use crate::pkey::PKey;
    use crate::rsa::Rsa;

    #[test]
    fn verify_fail() {
        let key1 = Rsa::generate(4096).unwrap();
        let key1 = PKey::from_rsa(key1).unwrap();

        let md = Md::sha256();
        let data = b"Some Crypto Text";

        let mut ctx = MdCtx::new().unwrap();
        ctx.digest_sign_init(Some(md), &key1).unwrap();
        ctx.digest_sign_update(data).unwrap();
        let mut signature = vec![];
        ctx.digest_sign_final_to_vec(&mut signature).unwrap();

        let bad_data = b"Some Crypto text";

        ctx.digest_verify_init(Some(md), &key1).unwrap();
        ctx.digest_verify_update(bad_data).unwrap();
        let valid = ctx.digest_verify_final(&signature).unwrap();
        assert!(!valid);
    }
}
