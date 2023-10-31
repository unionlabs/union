//! SMIME implementation using CMS
//!
//! CMS (PKCS#7) is an encryption standard.  It allows signing and encrypting data using
//! X.509 certificates.  The OpenSSL implementation of CMS is used in email encryption
//! generated from a `Vec` of bytes.  This `Vec` follows the smime protocol standards.
//! Data accepted by this module will be smime type `enveloped-data`.

use bitflags::bitflags;
use foreign_types::{ForeignType, ForeignTypeRef};
use libc::c_uint;
use std::ptr;

use crate::bio::{MemBio, MemBioSlice};
use crate::error::ErrorStack;
use crate::pkey::{HasPrivate, PKeyRef};
use crate::stack::StackRef;
use crate::symm::Cipher;
use crate::x509::{X509Ref, X509};
use crate::{cvt, cvt_p};
use openssl_macros::corresponds;

bitflags! {
    pub struct CMSOptions : c_uint {
        const TEXT = ffi::CMS_TEXT;
        const CMS_NOCERTS = ffi::CMS_NOCERTS;
        const NO_CONTENT_VERIFY = ffi::CMS_NO_CONTENT_VERIFY;
        const NO_ATTR_VERIFY = ffi::CMS_NO_ATTR_VERIFY;
        const NOSIGS = ffi::CMS_NOSIGS;
        const NOINTERN = ffi::CMS_NOINTERN;
        const NO_SIGNER_CERT_VERIFY = ffi::CMS_NO_SIGNER_CERT_VERIFY;
        const NOVERIFY = ffi::CMS_NOVERIFY;
        const DETACHED = ffi::CMS_DETACHED;
        const BINARY = ffi::CMS_BINARY;
        const NOATTR = ffi::CMS_NOATTR;
        const NOSMIMECAP = ffi::CMS_NOSMIMECAP;
        const NOOLDMIMETYPE = ffi::CMS_NOOLDMIMETYPE;
        const CRLFEOL = ffi::CMS_CRLFEOL;
        const STREAM = ffi::CMS_STREAM;
        const NOCRL = ffi::CMS_NOCRL;
        const PARTIAL = ffi::CMS_PARTIAL;
        const REUSE_DIGEST = ffi::CMS_REUSE_DIGEST;
        const USE_KEYID = ffi::CMS_USE_KEYID;
        const DEBUG_DECRYPT = ffi::CMS_DEBUG_DECRYPT;
        #[cfg(all(not(libressl), not(ossl101)))]
        const KEY_PARAM = ffi::CMS_KEY_PARAM;
        #[cfg(all(not(libressl), not(ossl101), not(ossl102)))]
        const ASCIICRLF = ffi::CMS_ASCIICRLF;
    }
}

foreign_type_and_impl_send_sync! {
    type CType = ffi::CMS_ContentInfo;
    fn drop = ffi::CMS_ContentInfo_free;

    /// High level CMS wrapper
    ///
    /// CMS supports nesting various types of data, including signatures, certificates,
    /// encrypted data, smime messages (encrypted email), and data digest.  The ContentInfo
    /// content type is the encapsulation of all those content types.  [`RFC 5652`] describes
    /// CMS and OpenSSL follows this RFC's implementation.
    ///
    /// [`RFC 5652`]: https://tools.ietf.org/html/rfc5652#page-6
    pub struct CmsContentInfo;
    /// Reference to [`CMSContentInfo`]
    ///
    /// [`CMSContentInfo`]:struct.CmsContentInfo.html
    pub struct CmsContentInfoRef;
}

impl CmsContentInfoRef {
    /// Given the sender's private key, `pkey` and the recipient's certificate, `cert`,
    /// decrypt the data in `self`.
    #[corresponds(CMS_decrypt)]
    pub fn decrypt<T>(&self, pkey: &PKeyRef<T>, cert: &X509) -> Result<Vec<u8>, ErrorStack>
    where
        T: HasPrivate,
    {
        unsafe {
            let pkey = pkey.as_ptr();
            let cert = cert.as_ptr();
            let out = MemBio::new()?;

            cvt(ffi::CMS_decrypt(
                self.as_ptr(),
                pkey,
                cert,
                ptr::null_mut(),
                out.as_ptr(),
                0,
            ))?;

            Ok(out.get_buf().to_owned())
        }
    }

    /// Given the sender's private key, `pkey`,
    /// decrypt the data in `self` without validating the recipient certificate.
    ///
    /// *Warning*: Not checking the recipient certificate may leave you vulnerable to Bleichenbacher's attack on PKCS#1 v1.5 RSA padding.
    #[corresponds(CMS_decrypt)]
    // FIXME merge into decrypt
    pub fn decrypt_without_cert_check<T>(&self, pkey: &PKeyRef<T>) -> Result<Vec<u8>, ErrorStack>
    where
        T: HasPrivate,
    {
        unsafe {
            let pkey = pkey.as_ptr();
            let out = MemBio::new()?;

            cvt(ffi::CMS_decrypt(
                self.as_ptr(),
                pkey,
                ptr::null_mut(),
                ptr::null_mut(),
                out.as_ptr(),
                0,
            ))?;

            Ok(out.get_buf().to_owned())
        }
    }

    to_der! {
        /// Serializes this CmsContentInfo using DER.
        #[corresponds(i2d_CMS_ContentInfo)]
        to_der,
        ffi::i2d_CMS_ContentInfo
    }

    to_pem! {
        /// Serializes this CmsContentInfo using DER.
        #[corresponds(PEM_write_bio_CMS)]
        to_pem,
        ffi::PEM_write_bio_CMS
    }
}

impl CmsContentInfo {
    /// Parses a smime formatted `vec` of bytes into a `CmsContentInfo`.
    #[corresponds(SMIME_read_CMS)]
    pub fn smime_read_cms(smime: &[u8]) -> Result<CmsContentInfo, ErrorStack> {
        unsafe {
            let bio = MemBioSlice::new(smime)?;

            let cms = cvt_p(ffi::SMIME_read_CMS(bio.as_ptr(), ptr::null_mut()))?;

            Ok(CmsContentInfo::from_ptr(cms))
        }
    }

    from_der! {
        /// Deserializes a DER-encoded ContentInfo structure.
        #[corresponds(d2i_CMS_ContentInfo)]
        from_der,
        CmsContentInfo,
        ffi::d2i_CMS_ContentInfo
    }

    from_pem! {
        /// Deserializes a PEM-encoded ContentInfo structure.
        #[corresponds(PEM_read_bio_CMS)]
        from_pem,
        CmsContentInfo,
        ffi::PEM_read_bio_CMS
    }

    /// Given a signing cert `signcert`, private key `pkey`, a certificate stack `certs`,
    /// data `data` and flags `flags`, create a CmsContentInfo struct.
    ///
    /// All arguments are optional.
    #[corresponds(CMS_sign)]
    pub fn sign<T>(
        signcert: Option<&X509Ref>,
        pkey: Option<&PKeyRef<T>>,
        certs: Option<&StackRef<X509>>,
        data: Option<&[u8]>,
        flags: CMSOptions,
    ) -> Result<CmsContentInfo, ErrorStack>
    where
        T: HasPrivate,
    {
        unsafe {
            let signcert = signcert.map_or(ptr::null_mut(), |p| p.as_ptr());
            let pkey = pkey.map_or(ptr::null_mut(), |p| p.as_ptr());
            let data_bio = match data {
                Some(data) => Some(MemBioSlice::new(data)?),
                None => None,
            };
            let data_bio_ptr = data_bio.as_ref().map_or(ptr::null_mut(), |p| p.as_ptr());
            let certs = certs.map_or(ptr::null_mut(), |p| p.as_ptr());

            let cms = cvt_p(ffi::CMS_sign(
                signcert,
                pkey,
                certs,
                data_bio_ptr,
                flags.bits(),
            ))?;

            Ok(CmsContentInfo::from_ptr(cms))
        }
    }

    /// Given a certificate stack `certs`, data `data`, cipher `cipher` and flags `flags`,
    /// create a CmsContentInfo struct.
    ///
    /// OpenSSL documentation at [`CMS_encrypt`]
    ///
    /// [`CMS_encrypt`]: https://www.openssl.org/docs/manmaster/man3/CMS_encrypt.html
    #[corresponds(CMS_encrypt)]
    pub fn encrypt(
        certs: &StackRef<X509>,
        data: &[u8],
        cipher: Cipher,
        flags: CMSOptions,
    ) -> Result<CmsContentInfo, ErrorStack> {
        unsafe {
            let data_bio = MemBioSlice::new(data)?;

            let cms = cvt_p(ffi::CMS_encrypt(
                certs.as_ptr(),
                data_bio.as_ptr(),
                cipher.as_ptr(),
                flags.bits(),
            ))?;

            Ok(CmsContentInfo::from_ptr(cms))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::pkcs12::Pkcs12;
    use crate::stack::Stack;
    use crate::x509::X509;

    #[test]
    fn cms_encrypt_decrypt() {
        #[cfg(ossl300)]
        let _provider = crate::provider::Provider::try_load(None, "legacy", true).unwrap();

        // load cert with public key only
        let pub_cert_bytes = include_bytes!("../test/cms_pubkey.der");
        let pub_cert = X509::from_der(pub_cert_bytes).expect("failed to load pub cert");

        // load cert with private key
        let priv_cert_bytes = include_bytes!("../test/cms.p12");
        let priv_cert = Pkcs12::from_der(priv_cert_bytes).expect("failed to load priv cert");
        let priv_cert = priv_cert
            .parse("mypass")
            .expect("failed to parse priv cert");

        // encrypt cms message using public key cert
        let input = String::from("My Message");
        let mut cert_stack = Stack::new().expect("failed to create stack");
        cert_stack
            .push(pub_cert)
            .expect("failed to add pub cert to stack");

        let encrypt = CmsContentInfo::encrypt(
            &cert_stack,
            input.as_bytes(),
            Cipher::des_ede3_cbc(),
            CMSOptions::empty(),
        )
        .expect("failed create encrypted cms");

        // decrypt cms message using private key cert (DER)
        {
            let encrypted_der = encrypt.to_der().expect("failed to create der from cms");
            let decrypt =
                CmsContentInfo::from_der(&encrypted_der).expect("failed read cms from der");

            let decrypt_with_cert_check = decrypt
                .decrypt(&priv_cert.pkey, &priv_cert.cert)
                .expect("failed to decrypt cms");
            let decrypt_with_cert_check = String::from_utf8(decrypt_with_cert_check)
                .expect("failed to create string from cms content");

            let decrypt_without_cert_check = decrypt
                .decrypt_without_cert_check(&priv_cert.pkey)
                .expect("failed to decrypt cms");
            let decrypt_without_cert_check = String::from_utf8(decrypt_without_cert_check)
                .expect("failed to create string from cms content");

            assert_eq!(input, decrypt_with_cert_check);
            assert_eq!(input, decrypt_without_cert_check);
        }

        // decrypt cms message using private key cert (PEM)
        {
            let encrypted_pem = encrypt.to_pem().expect("failed to create pem from cms");
            let decrypt =
                CmsContentInfo::from_pem(&encrypted_pem).expect("failed read cms from pem");

            let decrypt_with_cert_check = decrypt
                .decrypt(&priv_cert.pkey, &priv_cert.cert)
                .expect("failed to decrypt cms");
            let decrypt_with_cert_check = String::from_utf8(decrypt_with_cert_check)
                .expect("failed to create string from cms content");

            let decrypt_without_cert_check = decrypt
                .decrypt_without_cert_check(&priv_cert.pkey)
                .expect("failed to decrypt cms");
            let decrypt_without_cert_check = String::from_utf8(decrypt_without_cert_check)
                .expect("failed to create string from cms content");

            assert_eq!(input, decrypt_with_cert_check);
            assert_eq!(input, decrypt_without_cert_check);
        }
    }
}
