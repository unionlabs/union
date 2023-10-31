//! The standard defining the format of public key certificates.
//!
//! An `X509` certificate binds an identity to a public key, and is either
//! signed by a certificate authority (CA) or self-signed. An entity that gets
//! a hold of a certificate can both verify your identity (via a CA) and encrypt
//! data with the included public key. `X509` certificates are used in many
//! Internet protocols, including SSL/TLS, which is the basis for HTTPS,
//! the secure protocol for browsing the web.

use cfg_if::cfg_if;
use foreign_types::{ForeignType, ForeignTypeRef};
use libc::{c_int, c_long};
use std::cmp;
use std::error::Error;
use std::ffi::{CStr, CString};
use std::fmt;
use std::marker::PhantomData;
use std::mem;
use std::path::Path;
use std::ptr;
use std::slice;
use std::str;

use crate::asn1::{
    Asn1BitStringRef, Asn1IntegerRef, Asn1ObjectRef, Asn1StringRef, Asn1TimeRef, Asn1Type,
};
use crate::bio::MemBioSlice;
use crate::conf::ConfRef;
use crate::error::ErrorStack;
use crate::ex_data::Index;
use crate::hash::{DigestBytes, MessageDigest};
use crate::nid::Nid;
use crate::pkey::{HasPrivate, HasPublic, PKey, PKeyRef, Public};
use crate::ssl::SslRef;
use crate::stack::{Stack, StackRef, Stackable};
use crate::string::OpensslString;
use crate::util::{ForeignTypeExt, ForeignTypeRefExt};
use crate::{cvt, cvt_n, cvt_p};
use openssl_macros::corresponds;

#[cfg(any(ossl102, libressl261))]
pub mod verify;

pub mod extension;
pub mod store;

#[cfg(test)]
mod tests;

foreign_type_and_impl_send_sync! {
    type CType = ffi::X509_STORE_CTX;
    fn drop = ffi::X509_STORE_CTX_free;

    /// An `X509` certificate store context.
    pub struct X509StoreContext;

    /// A reference to an [`X509StoreContext`].
    pub struct X509StoreContextRef;
}

impl X509StoreContext {
    /// Returns the index which can be used to obtain a reference to the `Ssl` associated with a
    /// context.
    #[corresponds(SSL_get_ex_data_X509_STORE_CTX_idx)]
    pub fn ssl_idx() -> Result<Index<X509StoreContext, SslRef>, ErrorStack> {
        unsafe { cvt_n(ffi::SSL_get_ex_data_X509_STORE_CTX_idx()).map(|idx| Index::from_raw(idx)) }
    }

    /// Creates a new `X509StoreContext` instance.
    #[corresponds(X509_STORE_CTX_new)]
    pub fn new() -> Result<X509StoreContext, ErrorStack> {
        unsafe {
            ffi::init();
            cvt_p(ffi::X509_STORE_CTX_new()).map(X509StoreContext)
        }
    }
}

impl X509StoreContextRef {
    /// Returns application data pertaining to an `X509` store context.
    #[corresponds(X509_STORE_CTX_get_ex_data)]
    pub fn ex_data<T>(&self, index: Index<X509StoreContext, T>) -> Option<&T> {
        unsafe {
            let data = ffi::X509_STORE_CTX_get_ex_data(self.as_ptr(), index.as_raw());
            if data.is_null() {
                None
            } else {
                Some(&*(data as *const T))
            }
        }
    }

    /// Returns the error code of the context.
    #[corresponds(X509_STORE_CTX_get_error)]
    pub fn error(&self) -> X509VerifyResult {
        unsafe { X509VerifyResult::from_raw(ffi::X509_STORE_CTX_get_error(self.as_ptr())) }
    }

    /// Initializes this context with the given certificate, certificates chain and certificate
    /// store. After initializing the context, the `with_context` closure is called with the prepared
    /// context. As long as the closure is running, the context stays initialized and can be used
    /// to e.g. verify a certificate. The context will be cleaned up, after the closure finished.
    ///
    /// * `trust` - The certificate store with the trusted certificates.
    /// * `cert` - The certificate that should be verified.
    /// * `cert_chain` - The certificates chain.
    /// * `with_context` - The closure that is called with the initialized context.
    ///
    /// This corresponds to [`X509_STORE_CTX_init`] before calling `with_context` and to
    /// [`X509_STORE_CTX_cleanup`] after calling `with_context`.
    ///
    /// [`X509_STORE_CTX_init`]:  https://www.openssl.org/docs/man1.0.2/crypto/X509_STORE_CTX_init.html
    /// [`X509_STORE_CTX_cleanup`]:  https://www.openssl.org/docs/man1.0.2/crypto/X509_STORE_CTX_cleanup.html
    pub fn init<F, T>(
        &mut self,
        trust: &store::X509StoreRef,
        cert: &X509Ref,
        cert_chain: &StackRef<X509>,
        with_context: F,
    ) -> Result<T, ErrorStack>
    where
        F: FnOnce(&mut X509StoreContextRef) -> Result<T, ErrorStack>,
    {
        struct Cleanup<'a>(&'a mut X509StoreContextRef);

        impl<'a> Drop for Cleanup<'a> {
            fn drop(&mut self) {
                unsafe {
                    ffi::X509_STORE_CTX_cleanup(self.0.as_ptr());
                }
            }
        }

        unsafe {
            cvt(ffi::X509_STORE_CTX_init(
                self.as_ptr(),
                trust.as_ptr(),
                cert.as_ptr(),
                cert_chain.as_ptr(),
            ))?;

            let cleanup = Cleanup(self);
            with_context(cleanup.0)
        }
    }

    /// Verifies the stored certificate.
    ///
    /// Returns `true` if verification succeeds. The `error` method will return the specific
    /// validation error if the certificate was not valid.
    ///
    /// This will only work inside of a call to `init`.
    #[corresponds(X509_verify_cert)]
    pub fn verify_cert(&mut self) -> Result<bool, ErrorStack> {
        unsafe { cvt_n(ffi::X509_verify_cert(self.as_ptr())).map(|n| n != 0) }
    }

    /// Set the error code of the context.
    #[corresponds(X509_STORE_CTX_set_error)]
    pub fn set_error(&mut self, result: X509VerifyResult) {
        unsafe {
            ffi::X509_STORE_CTX_set_error(self.as_ptr(), result.as_raw());
        }
    }

    /// Returns a reference to the certificate which caused the error or None if
    /// no certificate is relevant to the error.
    #[corresponds(X509_STORE_CTX_get_current_cert)]
    pub fn current_cert(&self) -> Option<&X509Ref> {
        unsafe {
            let ptr = ffi::X509_STORE_CTX_get_current_cert(self.as_ptr());
            X509Ref::from_const_ptr_opt(ptr)
        }
    }

    /// Returns a non-negative integer representing the depth in the certificate
    /// chain where the error occurred. If it is zero it occurred in the end
    /// entity certificate, one if it is the certificate which signed the end
    /// entity certificate and so on.
    #[corresponds(X509_STORE_CTX_get_error_depth)]
    pub fn error_depth(&self) -> u32 {
        unsafe { ffi::X509_STORE_CTX_get_error_depth(self.as_ptr()) as u32 }
    }

    /// Returns a reference to a complete valid `X509` certificate chain.
    #[corresponds(X509_STORE_CTX_get0_chain)]
    pub fn chain(&self) -> Option<&StackRef<X509>> {
        unsafe {
            let chain = X509_STORE_CTX_get0_chain(self.as_ptr());

            if chain.is_null() {
                None
            } else {
                Some(StackRef::from_ptr(chain))
            }
        }
    }
}

/// A builder used to construct an `X509`.
pub struct X509Builder(X509);

impl X509Builder {
    /// Creates a new builder.
    #[corresponds(X509_new)]
    pub fn new() -> Result<X509Builder, ErrorStack> {
        unsafe {
            ffi::init();
            cvt_p(ffi::X509_new()).map(|p| X509Builder(X509(p)))
        }
    }

    /// Sets the notAfter constraint on the certificate.
    #[corresponds(X509_set1_notAfter)]
    pub fn set_not_after(&mut self, not_after: &Asn1TimeRef) -> Result<(), ErrorStack> {
        unsafe { cvt(X509_set1_notAfter(self.0.as_ptr(), not_after.as_ptr())).map(|_| ()) }
    }

    /// Sets the notBefore constraint on the certificate.
    #[corresponds(X509_set1_notBefore)]
    pub fn set_not_before(&mut self, not_before: &Asn1TimeRef) -> Result<(), ErrorStack> {
        unsafe { cvt(X509_set1_notBefore(self.0.as_ptr(), not_before.as_ptr())).map(|_| ()) }
    }

    /// Sets the version of the certificate.
    ///
    /// Note that the version is zero-indexed; that is, a certificate corresponding to version 3 of
    /// the X.509 standard should pass `2` to this method.
    #[corresponds(X509_set_version)]
    pub fn set_version(&mut self, version: i32) -> Result<(), ErrorStack> {
        unsafe { cvt(ffi::X509_set_version(self.0.as_ptr(), version.into())).map(|_| ()) }
    }

    /// Sets the serial number of the certificate.
    #[corresponds(X509_set_serialNumber)]
    pub fn set_serial_number(&mut self, serial_number: &Asn1IntegerRef) -> Result<(), ErrorStack> {
        unsafe {
            cvt(ffi::X509_set_serialNumber(
                self.0.as_ptr(),
                serial_number.as_ptr(),
            ))
            .map(|_| ())
        }
    }

    /// Sets the issuer name of the certificate.
    #[corresponds(X509_set_issuer_name)]
    pub fn set_issuer_name(&mut self, issuer_name: &X509NameRef) -> Result<(), ErrorStack> {
        unsafe {
            cvt(ffi::X509_set_issuer_name(
                self.0.as_ptr(),
                issuer_name.as_ptr(),
            ))
            .map(|_| ())
        }
    }

    /// Sets the subject name of the certificate.
    ///
    /// When building certificates, the `C`, `ST`, and `O` options are common when using the openssl command line tools.
    /// The `CN` field is used for the common name, such as a DNS name.
    ///
    /// ```
    /// use openssl::x509::{X509, X509NameBuilder};
    ///
    /// let mut x509_name = openssl::x509::X509NameBuilder::new().unwrap();
    /// x509_name.append_entry_by_text("C", "US").unwrap();
    /// x509_name.append_entry_by_text("ST", "CA").unwrap();
    /// x509_name.append_entry_by_text("O", "Some organization").unwrap();
    /// x509_name.append_entry_by_text("CN", "www.example.com").unwrap();
    /// let x509_name = x509_name.build();
    ///
    /// let mut x509 = openssl::x509::X509::builder().unwrap();
    /// x509.set_subject_name(&x509_name).unwrap();
    /// ```
    #[corresponds(X509_set_subject_name)]
    pub fn set_subject_name(&mut self, subject_name: &X509NameRef) -> Result<(), ErrorStack> {
        unsafe {
            cvt(ffi::X509_set_subject_name(
                self.0.as_ptr(),
                subject_name.as_ptr(),
            ))
            .map(|_| ())
        }
    }

    /// Sets the public key associated with the certificate.
    #[corresponds(X509_set_pubkey)]
    pub fn set_pubkey<T>(&mut self, key: &PKeyRef<T>) -> Result<(), ErrorStack>
    where
        T: HasPublic,
    {
        unsafe { cvt(ffi::X509_set_pubkey(self.0.as_ptr(), key.as_ptr())).map(|_| ()) }
    }

    /// Returns a context object which is needed to create certain X509 extension values.
    ///
    /// Set `issuer` to `None` if the certificate will be self-signed.
    #[corresponds(X509V3_set_ctx)]
    pub fn x509v3_context<'a>(
        &'a self,
        issuer: Option<&'a X509Ref>,
        conf: Option<&'a ConfRef>,
    ) -> X509v3Context<'a> {
        unsafe {
            let mut ctx = mem::zeroed();

            let issuer = match issuer {
                Some(issuer) => issuer.as_ptr(),
                None => self.0.as_ptr(),
            };
            let subject = self.0.as_ptr();
            ffi::X509V3_set_ctx(
                &mut ctx,
                issuer,
                subject,
                ptr::null_mut(),
                ptr::null_mut(),
                0,
            );

            // nodb case taken care of since we zeroed ctx above
            if let Some(conf) = conf {
                ffi::X509V3_set_nconf(&mut ctx, conf.as_ptr());
            }

            X509v3Context(ctx, PhantomData)
        }
    }

    /// Adds an X509 extension value to the certificate.
    ///
    /// This works just as `append_extension` except it takes ownership of the `X509Extension`.
    pub fn append_extension(&mut self, extension: X509Extension) -> Result<(), ErrorStack> {
        self.append_extension2(&extension)
    }

    /// Adds an X509 extension value to the certificate.
    #[corresponds(X509_add_ext)]
    pub fn append_extension2(&mut self, extension: &X509ExtensionRef) -> Result<(), ErrorStack> {
        unsafe {
            cvt(ffi::X509_add_ext(self.0.as_ptr(), extension.as_ptr(), -1))?;
            Ok(())
        }
    }

    /// Signs the certificate with a private key.
    #[corresponds(X509_sign)]
    pub fn sign<T>(&mut self, key: &PKeyRef<T>, hash: MessageDigest) -> Result<(), ErrorStack>
    where
        T: HasPrivate,
    {
        unsafe { cvt(ffi::X509_sign(self.0.as_ptr(), key.as_ptr(), hash.as_ptr())).map(|_| ()) }
    }

    /// Consumes the builder, returning the certificate.
    pub fn build(self) -> X509 {
        self.0
    }
}

foreign_type_and_impl_send_sync! {
    type CType = ffi::X509;
    fn drop = ffi::X509_free;

    /// An `X509` public key certificate.
    pub struct X509;
    /// Reference to `X509`.
    pub struct X509Ref;
}

impl X509Ref {
    /// Returns this certificate's subject name.
    #[corresponds(X509_get_subject_name)]
    pub fn subject_name(&self) -> &X509NameRef {
        unsafe {
            let name = ffi::X509_get_subject_name(self.as_ptr());
            X509NameRef::from_const_ptr_opt(name).expect("subject name must not be null")
        }
    }

    /// Returns the hash of the certificates subject
    #[corresponds(X509_subject_name_hash)]
    pub fn subject_name_hash(&self) -> u32 {
        unsafe { ffi::X509_subject_name_hash(self.as_ptr()) as u32 }
    }

    /// Returns this certificate's issuer name.
    #[corresponds(X509_get_issuer_name)]
    pub fn issuer_name(&self) -> &X509NameRef {
        unsafe {
            let name = ffi::X509_get_issuer_name(self.as_ptr());
            X509NameRef::from_const_ptr_opt(name).expect("issuer name must not be null")
        }
    }

    /// Returns the hash of the certificates issuer
    #[corresponds(X509_issuer_name_hash)]
    pub fn issuer_name_hash(&self) -> u32 {
        unsafe { ffi::X509_issuer_name_hash(self.as_ptr()) as u32 }
    }

    /// Returns this certificate's subject alternative name entries, if they exist.
    #[corresponds(X509_get_ext_d2i)]
    pub fn subject_alt_names(&self) -> Option<Stack<GeneralName>> {
        unsafe {
            let stack = ffi::X509_get_ext_d2i(
                self.as_ptr(),
                ffi::NID_subject_alt_name,
                ptr::null_mut(),
                ptr::null_mut(),
            );
            Stack::from_ptr_opt(stack as *mut _)
        }
    }

    /// Returns this certificate's issuer alternative name entries, if they exist.
    #[corresponds(X509_get_ext_d2i)]
    pub fn issuer_alt_names(&self) -> Option<Stack<GeneralName>> {
        unsafe {
            let stack = ffi::X509_get_ext_d2i(
                self.as_ptr(),
                ffi::NID_issuer_alt_name,
                ptr::null_mut(),
                ptr::null_mut(),
            );
            Stack::from_ptr_opt(stack as *mut _)
        }
    }

    /// Returns this certificate's [`authority information access`] entries, if they exist.
    ///
    /// [`authority information access`]: https://tools.ietf.org/html/rfc5280#section-4.2.2.1
    #[corresponds(X509_get_ext_d2i)]
    pub fn authority_info(&self) -> Option<Stack<AccessDescription>> {
        unsafe {
            let stack = ffi::X509_get_ext_d2i(
                self.as_ptr(),
                ffi::NID_info_access,
                ptr::null_mut(),
                ptr::null_mut(),
            );
            Stack::from_ptr_opt(stack as *mut _)
        }
    }

    #[corresponds(X509_get_pubkey)]
    pub fn public_key(&self) -> Result<PKey<Public>, ErrorStack> {
        unsafe {
            let pkey = cvt_p(ffi::X509_get_pubkey(self.as_ptr()))?;
            Ok(PKey::from_ptr(pkey))
        }
    }

    /// Returns a digest of the DER representation of the certificate.
    #[corresponds(X509_digest)]
    pub fn digest(&self, hash_type: MessageDigest) -> Result<DigestBytes, ErrorStack> {
        unsafe {
            let mut digest = DigestBytes {
                buf: [0; ffi::EVP_MAX_MD_SIZE as usize],
                len: ffi::EVP_MAX_MD_SIZE as usize,
            };
            let mut len = ffi::EVP_MAX_MD_SIZE;
            cvt(ffi::X509_digest(
                self.as_ptr(),
                hash_type.as_ptr(),
                digest.buf.as_mut_ptr() as *mut _,
                &mut len,
            ))?;
            digest.len = len as usize;

            Ok(digest)
        }
    }

    #[deprecated(since = "0.10.9", note = "renamed to digest")]
    pub fn fingerprint(&self, hash_type: MessageDigest) -> Result<Vec<u8>, ErrorStack> {
        self.digest(hash_type).map(|b| b.to_vec())
    }

    /// Returns the certificate's Not After validity period.
    #[corresponds(X509_getm_notAfter)]
    pub fn not_after(&self) -> &Asn1TimeRef {
        unsafe {
            let date = X509_getm_notAfter(self.as_ptr());
            Asn1TimeRef::from_const_ptr_opt(date).expect("not_after must not be null")
        }
    }

    /// Returns the certificate's Not Before validity period.
    #[corresponds(X509_getm_notBefore)]
    pub fn not_before(&self) -> &Asn1TimeRef {
        unsafe {
            let date = X509_getm_notBefore(self.as_ptr());
            Asn1TimeRef::from_const_ptr_opt(date).expect("not_before must not be null")
        }
    }

    /// Returns the certificate's signature
    #[corresponds(X509_get0_signature)]
    pub fn signature(&self) -> &Asn1BitStringRef {
        unsafe {
            let mut signature = ptr::null();
            X509_get0_signature(&mut signature, ptr::null_mut(), self.as_ptr());
            Asn1BitStringRef::from_const_ptr_opt(signature).expect("signature must not be null")
        }
    }

    /// Returns the certificate's signature algorithm.
    #[corresponds(X509_get0_signature)]
    pub fn signature_algorithm(&self) -> &X509AlgorithmRef {
        unsafe {
            let mut algor = ptr::null();
            X509_get0_signature(ptr::null_mut(), &mut algor, self.as_ptr());
            X509AlgorithmRef::from_const_ptr_opt(algor)
                .expect("signature algorithm must not be null")
        }
    }

    /// Returns the list of OCSP responder URLs specified in the certificate's Authority Information
    /// Access field.
    #[corresponds(X509_get1_ocsp)]
    pub fn ocsp_responders(&self) -> Result<Stack<OpensslString>, ErrorStack> {
        unsafe { cvt_p(ffi::X509_get1_ocsp(self.as_ptr())).map(|p| Stack::from_ptr(p)) }
    }

    /// Checks that this certificate issued `subject`.
    #[corresponds(X509_check_issued)]
    pub fn issued(&self, subject: &X509Ref) -> X509VerifyResult {
        unsafe {
            let r = ffi::X509_check_issued(self.as_ptr(), subject.as_ptr());
            X509VerifyResult::from_raw(r)
        }
    }

    /// Returns certificate version. If this certificate has no explicit version set, it defaults to
    /// version 1.
    ///
    /// Note that `0` return value stands for version 1, `1` for version 2 and so on.
    #[corresponds(X509_get_version)]
    #[cfg(ossl110)]
    pub fn version(&self) -> i32 {
        unsafe { ffi::X509_get_version(self.as_ptr()) as i32 }
    }

    /// Check if the certificate is signed using the given public key.
    ///
    /// Only the signature is checked: no other checks (such as certificate chain validity)
    /// are performed.
    ///
    /// Returns `true` if verification succeeds.
    #[corresponds(X509_verify)]
    pub fn verify<T>(&self, key: &PKeyRef<T>) -> Result<bool, ErrorStack>
    where
        T: HasPublic,
    {
        unsafe { cvt_n(ffi::X509_verify(self.as_ptr(), key.as_ptr())).map(|n| n != 0) }
    }

    /// Returns this certificate's serial number.
    #[corresponds(X509_get_serialNumber)]
    pub fn serial_number(&self) -> &Asn1IntegerRef {
        unsafe {
            let r = ffi::X509_get_serialNumber(self.as_ptr());
            Asn1IntegerRef::from_const_ptr_opt(r).expect("serial number must not be null")
        }
    }

    to_pem! {
        /// Serializes the certificate into a PEM-encoded X509 structure.
        ///
        /// The output will have a header of `-----BEGIN CERTIFICATE-----`.
        #[corresponds(PEM_write_bio_X509)]
        to_pem,
        ffi::PEM_write_bio_X509
    }

    to_der! {
        /// Serializes the certificate into a DER-encoded X509 structure.
        #[corresponds(i2d_X509)]
        to_der,
        ffi::i2d_X509
    }

    to_pem! {
        /// Converts the certificate to human readable text.
        #[corresponds(X509_print)]
        to_text,
        ffi::X509_print
    }
}

impl ToOwned for X509Ref {
    type Owned = X509;

    fn to_owned(&self) -> X509 {
        unsafe {
            X509_up_ref(self.as_ptr());
            X509::from_ptr(self.as_ptr())
        }
    }
}

impl Ord for X509Ref {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        // X509_cmp returns a number <0 for less than, 0 for equal and >0 for greater than.
        // It can't fail if both pointers are valid, which we know is true.
        let cmp = unsafe { ffi::X509_cmp(self.as_ptr(), other.as_ptr()) };
        cmp.cmp(&0)
    }
}

impl PartialOrd for X509Ref {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialOrd<X509> for X509Ref {
    fn partial_cmp(&self, other: &X509) -> Option<cmp::Ordering> {
        <X509Ref as PartialOrd<X509Ref>>::partial_cmp(self, other)
    }
}

impl PartialEq for X509Ref {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == cmp::Ordering::Equal
    }
}

impl PartialEq<X509> for X509Ref {
    fn eq(&self, other: &X509) -> bool {
        <X509Ref as PartialEq<X509Ref>>::eq(self, other)
    }
}

impl Eq for X509Ref {}

impl X509 {
    /// Returns a new builder.
    pub fn builder() -> Result<X509Builder, ErrorStack> {
        X509Builder::new()
    }

    from_pem! {
        /// Deserializes a PEM-encoded X509 structure.
        ///
        /// The input should have a header of `-----BEGIN CERTIFICATE-----`.
        #[corresponds(PEM_read_bio_X509)]
        from_pem,
        X509,
        ffi::PEM_read_bio_X509
    }

    from_der! {
        /// Deserializes a DER-encoded X509 structure.
        #[corresponds(d2i_X509)]
        from_der,
        X509,
        ffi::d2i_X509
    }

    /// Deserializes a list of PEM-formatted certificates.
    #[corresponds(PEM_read_bio_X509)]
    pub fn stack_from_pem(pem: &[u8]) -> Result<Vec<X509>, ErrorStack> {
        unsafe {
            ffi::init();
            let bio = MemBioSlice::new(pem)?;

            let mut certs = vec![];
            loop {
                let r =
                    ffi::PEM_read_bio_X509(bio.as_ptr(), ptr::null_mut(), None, ptr::null_mut());
                if r.is_null() {
                    let err = ffi::ERR_peek_last_error();
                    if ffi::ERR_GET_LIB(err) == ffi::ERR_LIB_PEM
                        && ffi::ERR_GET_REASON(err) == ffi::PEM_R_NO_START_LINE
                    {
                        ffi::ERR_clear_error();
                        break;
                    }

                    return Err(ErrorStack::get());
                } else {
                    certs.push(X509(r));
                }
            }

            Ok(certs)
        }
    }
}

impl Clone for X509 {
    fn clone(&self) -> X509 {
        X509Ref::to_owned(self)
    }
}

impl fmt::Debug for X509 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let serial = match &self.serial_number().to_bn() {
            Ok(bn) => match bn.to_hex_str() {
                Ok(hex) => hex.to_string(),
                Err(_) => "".to_string(),
            },
            Err(_) => "".to_string(),
        };
        let mut debug_struct = formatter.debug_struct("X509");
        debug_struct.field("serial_number", &serial);
        debug_struct.field("signature_algorithm", &self.signature_algorithm().object());
        debug_struct.field("issuer", &self.issuer_name());
        debug_struct.field("subject", &self.subject_name());
        if let Some(subject_alt_names) = &self.subject_alt_names() {
            debug_struct.field("subject_alt_names", subject_alt_names);
        }
        debug_struct.field("not_before", &self.not_before());
        debug_struct.field("not_after", &self.not_after());

        if let Ok(public_key) = &self.public_key() {
            debug_struct.field("public_key", public_key);
        };
        // TODO: Print extensions once they are supported on the X509 struct.

        debug_struct.finish()
    }
}

impl AsRef<X509Ref> for X509Ref {
    fn as_ref(&self) -> &X509Ref {
        self
    }
}

impl Stackable for X509 {
    type StackType = ffi::stack_st_X509;
}

impl Ord for X509 {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        X509Ref::cmp(self, other)
    }
}

impl PartialOrd for X509 {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        X509Ref::partial_cmp(self, other)
    }
}

impl PartialOrd<X509Ref> for X509 {
    fn partial_cmp(&self, other: &X509Ref) -> Option<cmp::Ordering> {
        X509Ref::partial_cmp(self, other)
    }
}

impl PartialEq for X509 {
    fn eq(&self, other: &Self) -> bool {
        X509Ref::eq(self, other)
    }
}

impl PartialEq<X509Ref> for X509 {
    fn eq(&self, other: &X509Ref) -> bool {
        X509Ref::eq(self, other)
    }
}

impl Eq for X509 {}

/// A context object required to construct certain `X509` extension values.
pub struct X509v3Context<'a>(ffi::X509V3_CTX, PhantomData<(&'a X509Ref, &'a ConfRef)>);

impl<'a> X509v3Context<'a> {
    pub fn as_ptr(&self) -> *mut ffi::X509V3_CTX {
        &self.0 as *const _ as *mut _
    }
}

foreign_type_and_impl_send_sync! {
    type CType = ffi::X509_EXTENSION;
    fn drop = ffi::X509_EXTENSION_free;

    /// Permit additional fields to be added to an `X509` v3 certificate.
    pub struct X509Extension;
    /// Reference to `X509Extension`.
    pub struct X509ExtensionRef;
}

impl Stackable for X509Extension {
    type StackType = ffi::stack_st_X509_EXTENSION;
}

impl X509Extension {
    /// Constructs an X509 extension value. See `man x509v3_config` for information on supported
    /// names and their value formats.
    ///
    /// Some extension types, such as `subjectAlternativeName`, require an `X509v3Context` to be
    /// provided.
    ///
    /// See the extension module for builder types which will construct certain common extensions.
    pub fn new(
        conf: Option<&ConfRef>,
        context: Option<&X509v3Context<'_>>,
        name: &str,
        value: &str,
    ) -> Result<X509Extension, ErrorStack> {
        let name = CString::new(name).unwrap();
        let value = CString::new(value).unwrap();
        unsafe {
            ffi::init();
            let conf = conf.map_or(ptr::null_mut(), ConfRef::as_ptr);
            let context = context.map_or(ptr::null_mut(), X509v3Context::as_ptr);
            let name = name.as_ptr() as *mut _;
            let value = value.as_ptr() as *mut _;

            cvt_p(ffi::X509V3_EXT_nconf(conf, context, name, value)).map(X509Extension)
        }
    }

    /// Constructs an X509 extension value. See `man x509v3_config` for information on supported
    /// extensions and their value formats.
    ///
    /// Some extension types, such as `nid::SUBJECT_ALTERNATIVE_NAME`, require an `X509v3Context` to
    /// be provided.
    ///
    /// See the extension module for builder types which will construct certain common extensions.
    pub fn new_nid(
        conf: Option<&ConfRef>,
        context: Option<&X509v3Context<'_>>,
        name: Nid,
        value: &str,
    ) -> Result<X509Extension, ErrorStack> {
        let value = CString::new(value).unwrap();
        unsafe {
            ffi::init();
            let conf = conf.map_or(ptr::null_mut(), ConfRef::as_ptr);
            let context = context.map_or(ptr::null_mut(), X509v3Context::as_ptr);
            let name = name.as_raw();
            let value = value.as_ptr() as *mut _;

            cvt_p(ffi::X509V3_EXT_nconf_nid(conf, context, name, value)).map(X509Extension)
        }
    }

    /// Adds an alias for an extension
    ///
    /// # Safety
    ///
    /// This method modifies global state without locking and therefore is not thread safe
    #[corresponds(X509V3_EXT_add_alias)]
    pub unsafe fn add_alias(to: Nid, from: Nid) -> Result<(), ErrorStack> {
        ffi::init();
        cvt(ffi::X509V3_EXT_add_alias(to.as_raw(), from.as_raw())).map(|_| ())
    }
}

/// A builder used to construct an `X509Name`.
pub struct X509NameBuilder(X509Name);

impl X509NameBuilder {
    /// Creates a new builder.
    pub fn new() -> Result<X509NameBuilder, ErrorStack> {
        unsafe {
            ffi::init();
            cvt_p(ffi::X509_NAME_new()).map(|p| X509NameBuilder(X509Name(p)))
        }
    }

    /// Add a field entry by str.
    ///
    /// This corresponds to [`X509_NAME_add_entry_by_txt`].
    ///
    /// [`X509_NAME_add_entry_by_txt`]: https://www.openssl.org/docs/man1.1.0/crypto/X509_NAME_add_entry_by_txt.html
    pub fn append_entry_by_text(&mut self, field: &str, value: &str) -> Result<(), ErrorStack> {
        unsafe {
            let field = CString::new(field).unwrap();
            assert!(value.len() <= c_int::max_value() as usize);
            cvt(ffi::X509_NAME_add_entry_by_txt(
                self.0.as_ptr(),
                field.as_ptr() as *mut _,
                ffi::MBSTRING_UTF8,
                value.as_ptr(),
                value.len() as c_int,
                -1,
                0,
            ))
            .map(|_| ())
        }
    }

    /// Add a field entry by str with a specific type.
    ///
    /// This corresponds to [`X509_NAME_add_entry_by_txt`].
    ///
    /// [`X509_NAME_add_entry_by_txt`]: https://www.openssl.org/docs/man1.1.0/crypto/X509_NAME_add_entry_by_txt.html
    pub fn append_entry_by_text_with_type(
        &mut self,
        field: &str,
        value: &str,
        ty: Asn1Type,
    ) -> Result<(), ErrorStack> {
        unsafe {
            let field = CString::new(field).unwrap();
            assert!(value.len() <= c_int::max_value() as usize);
            cvt(ffi::X509_NAME_add_entry_by_txt(
                self.0.as_ptr(),
                field.as_ptr() as *mut _,
                ty.as_raw(),
                value.as_ptr(),
                value.len() as c_int,
                -1,
                0,
            ))
            .map(|_| ())
        }
    }

    /// Add a field entry by NID.
    ///
    /// This corresponds to [`X509_NAME_add_entry_by_NID`].
    ///
    /// [`X509_NAME_add_entry_by_NID`]: https://www.openssl.org/docs/man1.1.0/crypto/X509_NAME_add_entry_by_NID.html
    pub fn append_entry_by_nid(&mut self, field: Nid, value: &str) -> Result<(), ErrorStack> {
        unsafe {
            assert!(value.len() <= c_int::max_value() as usize);
            cvt(ffi::X509_NAME_add_entry_by_NID(
                self.0.as_ptr(),
                field.as_raw(),
                ffi::MBSTRING_UTF8,
                value.as_ptr() as *mut _,
                value.len() as c_int,
                -1,
                0,
            ))
            .map(|_| ())
        }
    }

    /// Add a field entry by NID with a specific type.
    ///
    /// This corresponds to [`X509_NAME_add_entry_by_NID`].
    ///
    /// [`X509_NAME_add_entry_by_NID`]: https://www.openssl.org/docs/man1.1.0/crypto/X509_NAME_add_entry_by_NID.html
    pub fn append_entry_by_nid_with_type(
        &mut self,
        field: Nid,
        value: &str,
        ty: Asn1Type,
    ) -> Result<(), ErrorStack> {
        unsafe {
            assert!(value.len() <= c_int::max_value() as usize);
            cvt(ffi::X509_NAME_add_entry_by_NID(
                self.0.as_ptr(),
                field.as_raw(),
                ty.as_raw(),
                value.as_ptr() as *mut _,
                value.len() as c_int,
                -1,
                0,
            ))
            .map(|_| ())
        }
    }

    /// Return an `X509Name`.
    pub fn build(self) -> X509Name {
        self.0
    }
}

foreign_type_and_impl_send_sync! {
    type CType = ffi::X509_NAME;
    fn drop = ffi::X509_NAME_free;

    /// The names of an `X509` certificate.
    pub struct X509Name;
    /// Reference to `X509Name`.
    pub struct X509NameRef;
}

impl X509Name {
    /// Returns a new builder.
    pub fn builder() -> Result<X509NameBuilder, ErrorStack> {
        X509NameBuilder::new()
    }

    /// Loads subject names from a file containing PEM-formatted certificates.
    ///
    /// This is commonly used in conjunction with `SslContextBuilder::set_client_ca_list`.
    pub fn load_client_ca_file<P: AsRef<Path>>(file: P) -> Result<Stack<X509Name>, ErrorStack> {
        let file = CString::new(file.as_ref().as_os_str().to_str().unwrap()).unwrap();
        unsafe { cvt_p(ffi::SSL_load_client_CA_file(file.as_ptr())).map(|p| Stack::from_ptr(p)) }
    }

    from_der! {
        /// Deserializes a DER-encoded X509 name structure.
        ///
        /// This corresponds to [`d2i_X509_NAME`].
        ///
        /// [`d2i_X509_NAME`]: https://www.openssl.org/docs/manmaster/man3/d2i_X509_NAME.html
        from_der,
        X509Name,
        ffi::d2i_X509_NAME
    }
}

impl Stackable for X509Name {
    type StackType = ffi::stack_st_X509_NAME;
}

impl X509NameRef {
    /// Returns the name entries by the nid.
    pub fn entries_by_nid(&self, nid: Nid) -> X509NameEntries<'_> {
        X509NameEntries {
            name: self,
            nid: Some(nid),
            loc: -1,
        }
    }

    /// Returns an iterator over all `X509NameEntry` values
    pub fn entries(&self) -> X509NameEntries<'_> {
        X509NameEntries {
            name: self,
            nid: None,
            loc: -1,
        }
    }

    to_der! {
        /// Serializes the certificate into a DER-encoded X509 name structure.
        ///
        /// This corresponds to [`i2d_X509_NAME`].
        ///
        /// [`i2d_X509_NAME`]: https://www.openssl.org/docs/man1.1.0/crypto/i2d_X509_NAME.html
        to_der,
        ffi::i2d_X509_NAME
    }
}

impl fmt::Debug for X509NameRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_list().entries(self.entries()).finish()
    }
}

/// A type to destructure and examine an `X509Name`.
pub struct X509NameEntries<'a> {
    name: &'a X509NameRef,
    nid: Option<Nid>,
    loc: c_int,
}

impl<'a> Iterator for X509NameEntries<'a> {
    type Item = &'a X509NameEntryRef;

    fn next(&mut self) -> Option<&'a X509NameEntryRef> {
        unsafe {
            match self.nid {
                Some(nid) => {
                    // There is a `Nid` specified to search for
                    self.loc =
                        ffi::X509_NAME_get_index_by_NID(self.name.as_ptr(), nid.as_raw(), self.loc);
                    if self.loc == -1 {
                        return None;
                    }
                }
                None => {
                    // Iterate over all `Nid`s
                    self.loc += 1;
                    if self.loc >= ffi::X509_NAME_entry_count(self.name.as_ptr()) {
                        return None;
                    }
                }
            }

            let entry = ffi::X509_NAME_get_entry(self.name.as_ptr(), self.loc);

            Some(X509NameEntryRef::from_const_ptr_opt(entry).expect("entry must not be null"))
        }
    }
}

foreign_type_and_impl_send_sync! {
    type CType = ffi::X509_NAME_ENTRY;
    fn drop = ffi::X509_NAME_ENTRY_free;

    /// A name entry associated with a `X509Name`.
    pub struct X509NameEntry;
    /// Reference to `X509NameEntry`.
    pub struct X509NameEntryRef;
}

impl X509NameEntryRef {
    /// Returns the field value of an `X509NameEntry`.
    ///
    /// This corresponds to [`X509_NAME_ENTRY_get_data`].
    ///
    /// [`X509_NAME_ENTRY_get_data`]: https://www.openssl.org/docs/man1.1.0/crypto/X509_NAME_ENTRY_get_data.html
    pub fn data(&self) -> &Asn1StringRef {
        unsafe {
            let data = ffi::X509_NAME_ENTRY_get_data(self.as_ptr());
            Asn1StringRef::from_ptr(data)
        }
    }

    /// Returns the `Asn1Object` value of an `X509NameEntry`.
    /// This is useful for finding out about the actual `Nid` when iterating over all `X509NameEntries`.
    ///
    /// This corresponds to [`X509_NAME_ENTRY_get_object`].
    ///
    /// [`X509_NAME_ENTRY_get_object`]: https://www.openssl.org/docs/man1.1.0/crypto/X509_NAME_ENTRY_get_object.html
    pub fn object(&self) -> &Asn1ObjectRef {
        unsafe {
            let object = ffi::X509_NAME_ENTRY_get_object(self.as_ptr());
            Asn1ObjectRef::from_ptr(object)
        }
    }
}

impl fmt::Debug for X509NameEntryRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_fmt(format_args!("{:?} = {:?}", self.object(), self.data()))
    }
}

/// A builder used to construct an `X509Req`.
pub struct X509ReqBuilder(X509Req);

impl X509ReqBuilder {
    /// Returns a builder for a certificate request.
    ///
    /// This corresponds to [`X509_REQ_new`].
    ///
    ///[`X509_REQ_new`]: https://www.openssl.org/docs/man1.1.0/crypto/X509_REQ_new.html
    pub fn new() -> Result<X509ReqBuilder, ErrorStack> {
        unsafe {
            ffi::init();
            cvt_p(ffi::X509_REQ_new()).map(|p| X509ReqBuilder(X509Req(p)))
        }
    }

    /// Set the numerical value of the version field.
    ///
    /// This corresponds to [`X509_REQ_set_version`].
    ///
    ///[`X509_REQ_set_version`]: https://www.openssl.org/docs/man1.1.0/crypto/X509_REQ_set_version.html
    pub fn set_version(&mut self, version: i32) -> Result<(), ErrorStack> {
        unsafe { cvt(ffi::X509_REQ_set_version(self.0.as_ptr(), version.into())).map(|_| ()) }
    }

    /// Set the issuer name.
    ///
    /// This corresponds to [`X509_REQ_set_subject_name`].
    ///
    /// [`X509_REQ_set_subject_name`]: https://www.openssl.org/docs/man1.1.0/crypto/X509_REQ_set_subject_name.html
    pub fn set_subject_name(&mut self, subject_name: &X509NameRef) -> Result<(), ErrorStack> {
        unsafe {
            cvt(ffi::X509_REQ_set_subject_name(
                self.0.as_ptr(),
                subject_name.as_ptr(),
            ))
            .map(|_| ())
        }
    }

    /// Set the public key.
    ///
    /// This corresponds to [`X509_REQ_set_pubkey`].
    ///
    /// [`X509_REQ_set_pubkey`]: https://www.openssl.org/docs/man1.1.0/crypto/X509_REQ_set_pubkey.html
    pub fn set_pubkey<T>(&mut self, key: &PKeyRef<T>) -> Result<(), ErrorStack>
    where
        T: HasPublic,
    {
        unsafe { cvt(ffi::X509_REQ_set_pubkey(self.0.as_ptr(), key.as_ptr())).map(|_| ()) }
    }

    /// Return an `X509v3Context`. This context object can be used to construct
    /// certain `X509` extensions.
    pub fn x509v3_context<'a>(&'a self, conf: Option<&'a ConfRef>) -> X509v3Context<'a> {
        unsafe {
            let mut ctx = mem::zeroed();

            ffi::X509V3_set_ctx(
                &mut ctx,
                ptr::null_mut(),
                ptr::null_mut(),
                self.0.as_ptr(),
                ptr::null_mut(),
                0,
            );

            // nodb case taken care of since we zeroed ctx above
            if let Some(conf) = conf {
                ffi::X509V3_set_nconf(&mut ctx, conf.as_ptr());
            }

            X509v3Context(ctx, PhantomData)
        }
    }

    /// Permits any number of extension fields to be added to the certificate.
    pub fn add_extensions(
        &mut self,
        extensions: &StackRef<X509Extension>,
    ) -> Result<(), ErrorStack> {
        unsafe {
            cvt(ffi::X509_REQ_add_extensions(
                self.0.as_ptr(),
                extensions.as_ptr(),
            ))
            .map(|_| ())
        }
    }

    /// Sign the request using a private key.
    ///
    /// This corresponds to [`X509_REQ_sign`].
    ///
    /// [`X509_REQ_sign`]: https://www.openssl.org/docs/man1.1.0/crypto/X509_REQ_sign.html
    pub fn sign<T>(&mut self, key: &PKeyRef<T>, hash: MessageDigest) -> Result<(), ErrorStack>
    where
        T: HasPrivate,
    {
        unsafe {
            cvt(ffi::X509_REQ_sign(
                self.0.as_ptr(),
                key.as_ptr(),
                hash.as_ptr(),
            ))
            .map(|_| ())
        }
    }

    /// Returns the `X509Req`.
    pub fn build(self) -> X509Req {
        self.0
    }
}

foreign_type_and_impl_send_sync! {
    type CType = ffi::X509_REQ;
    fn drop = ffi::X509_REQ_free;

    /// An `X509` certificate request.
    pub struct X509Req;
    /// Reference to `X509Req`.
    pub struct X509ReqRef;
}

impl X509Req {
    /// A builder for `X509Req`.
    pub fn builder() -> Result<X509ReqBuilder, ErrorStack> {
        X509ReqBuilder::new()
    }

    from_pem! {
        /// Deserializes a PEM-encoded PKCS#10 certificate request structure.
        ///
        /// The input should have a header of `-----BEGIN CERTIFICATE REQUEST-----`.
        ///
        /// This corresponds to [`PEM_read_bio_X509_REQ`].
        ///
        /// [`PEM_read_bio_X509_REQ`]: https://www.openssl.org/docs/man1.0.2/crypto/PEM_read_bio_X509_REQ.html
        from_pem,
        X509Req,
        ffi::PEM_read_bio_X509_REQ
    }

    from_der! {
        /// Deserializes a DER-encoded PKCS#10 certificate request structure.
        ///
        /// This corresponds to [`d2i_X509_REQ`].
        ///
        /// [`d2i_X509_REQ`]: https://www.openssl.org/docs/man1.1.0/crypto/d2i_X509_REQ.html
        from_der,
        X509Req,
        ffi::d2i_X509_REQ
    }
}

impl X509ReqRef {
    to_pem! {
        /// Serializes the certificate request to a PEM-encoded PKCS#10 structure.
        ///
        /// The output will have a header of `-----BEGIN CERTIFICATE REQUEST-----`.
        ///
        /// This corresponds to [`PEM_write_bio_X509_REQ`].
        ///
        /// [`PEM_write_bio_X509_REQ`]: https://www.openssl.org/docs/man1.0.2/crypto/PEM_write_bio_X509_REQ.html
        to_pem,
        ffi::PEM_write_bio_X509_REQ
    }

    to_der! {
        /// Serializes the certificate request to a DER-encoded PKCS#10 structure.
        ///
        /// This corresponds to [`i2d_X509_REQ`].
        ///
        /// [`i2d_X509_REQ`]: https://www.openssl.org/docs/man1.0.2/crypto/i2d_X509_REQ.html
        to_der,
        ffi::i2d_X509_REQ
    }

    /// Returns the numerical value of the version field of the certificate request.
    ///
    /// This corresponds to [`X509_REQ_get_version`]
    ///
    /// [`X509_REQ_get_version`]: https://www.openssl.org/docs/man1.1.0/crypto/X509_REQ_get_version.html
    pub fn version(&self) -> i32 {
        unsafe { X509_REQ_get_version(self.as_ptr()) as i32 }
    }

    /// Returns the subject name of the certificate request.
    ///
    /// This corresponds to [`X509_REQ_get_subject_name`]
    ///
    /// [`X509_REQ_get_subject_name`]: https://www.openssl.org/docs/man1.1.0/crypto/X509_REQ_get_subject_name.html
    pub fn subject_name(&self) -> &X509NameRef {
        unsafe {
            let name = X509_REQ_get_subject_name(self.as_ptr());
            X509NameRef::from_const_ptr_opt(name).expect("subject name must not be null")
        }
    }

    /// Returns the public key of the certificate request.
    ///
    /// This corresponds to [`X509_REQ_get_pubkey"]
    ///
    /// [`X509_REQ_get_pubkey`]: https://www.openssl.org/docs/man1.1.0/crypto/X509_REQ_get_pubkey.html
    pub fn public_key(&self) -> Result<PKey<Public>, ErrorStack> {
        unsafe {
            let key = cvt_p(ffi::X509_REQ_get_pubkey(self.as_ptr()))?;
            Ok(PKey::from_ptr(key))
        }
    }

    /// Check if the certificate request is signed using the given public key.
    ///
    /// Returns `true` if verification succeeds.
    ///
    /// This corresponds to [`X509_REQ_verify"].
    ///
    /// [`X509_REQ_verify`]: https://www.openssl.org/docs/man1.1.0/crypto/X509_REQ_verify.html
    pub fn verify<T>(&self, key: &PKeyRef<T>) -> Result<bool, ErrorStack>
    where
        T: HasPublic,
    {
        unsafe { cvt_n(ffi::X509_REQ_verify(self.as_ptr(), key.as_ptr())).map(|n| n != 0) }
    }

    /// Returns the extensions of the certificate request.
    ///
    /// This corresponds to [`X509_REQ_get_extensions"]
    pub fn extensions(&self) -> Result<Stack<X509Extension>, ErrorStack> {
        unsafe {
            let extensions = cvt_p(ffi::X509_REQ_get_extensions(self.as_ptr()))?;
            Ok(Stack::from_ptr(extensions))
        }
    }
}

/// The result of peer certificate verification.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct X509VerifyResult(c_int);

impl fmt::Debug for X509VerifyResult {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("X509VerifyResult")
            .field("code", &self.0)
            .field("error", &self.error_string())
            .finish()
    }
}

impl fmt::Display for X509VerifyResult {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str(self.error_string())
    }
}

impl Error for X509VerifyResult {}

impl X509VerifyResult {
    /// Creates an `X509VerifyResult` from a raw error number.
    ///
    /// # Safety
    ///
    /// Some methods on `X509VerifyResult` are not thread safe if the error
    /// number is invalid.
    pub unsafe fn from_raw(err: c_int) -> X509VerifyResult {
        X509VerifyResult(err)
    }

    /// Return the integer representation of an `X509VerifyResult`.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn as_raw(&self) -> c_int {
        self.0
    }

    /// Return a human readable error string from the verification error.
    ///
    /// This corresponds to [`X509_verify_cert_error_string`].
    ///
    /// [`X509_verify_cert_error_string`]: https://www.openssl.org/docs/man1.1.0/crypto/X509_verify_cert_error_string.html
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn error_string(&self) -> &'static str {
        ffi::init();

        unsafe {
            let s = ffi::X509_verify_cert_error_string(self.0 as c_long);
            str::from_utf8(CStr::from_ptr(s).to_bytes()).unwrap()
        }
    }

    /// Successful peer certificate verification.
    pub const OK: X509VerifyResult = X509VerifyResult(ffi::X509_V_OK);
    /// Application verification failure.
    pub const APPLICATION_VERIFICATION: X509VerifyResult =
        X509VerifyResult(ffi::X509_V_ERR_APPLICATION_VERIFICATION);
}

foreign_type_and_impl_send_sync! {
    type CType = ffi::GENERAL_NAME;
    fn drop = ffi::GENERAL_NAME_free;

    /// An `X509` certificate alternative names.
    pub struct GeneralName;
    /// Reference to `GeneralName`.
    pub struct GeneralNameRef;
}

impl GeneralNameRef {
    fn ia5_string(&self, ffi_type: c_int) -> Option<&str> {
        unsafe {
            if (*self.as_ptr()).type_ != ffi_type {
                return None;
            }

            let ptr = ASN1_STRING_get0_data((*self.as_ptr()).d as *mut _);
            let len = ffi::ASN1_STRING_length((*self.as_ptr()).d as *mut _);

            let slice = slice::from_raw_parts(ptr as *const u8, len as usize);
            // IA5Strings are stated to be ASCII (specifically IA5). Hopefully
            // OpenSSL checks that when loading a certificate but if not we'll
            // use this instead of from_utf8_unchecked just in case.
            str::from_utf8(slice).ok()
        }
    }

    /// Returns the contents of this `GeneralName` if it is an `rfc822Name`.
    pub fn email(&self) -> Option<&str> {
        self.ia5_string(ffi::GEN_EMAIL)
    }

    /// Returns the contents of this `GeneralName` if it is a `dNSName`.
    pub fn dnsname(&self) -> Option<&str> {
        self.ia5_string(ffi::GEN_DNS)
    }

    /// Returns the contents of this `GeneralName` if it is an `uniformResourceIdentifier`.
    pub fn uri(&self) -> Option<&str> {
        self.ia5_string(ffi::GEN_URI)
    }

    /// Returns the contents of this `GeneralName` if it is an `iPAddress`.
    pub fn ipaddress(&self) -> Option<&[u8]> {
        unsafe {
            if (*self.as_ptr()).type_ != ffi::GEN_IPADD {
                return None;
            }

            let ptr = ASN1_STRING_get0_data((*self.as_ptr()).d as *mut _);
            let len = ffi::ASN1_STRING_length((*self.as_ptr()).d as *mut _);

            Some(slice::from_raw_parts(ptr as *const u8, len as usize))
        }
    }
}

impl fmt::Debug for GeneralNameRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(email) = self.email() {
            formatter.write_str(email)
        } else if let Some(dnsname) = self.dnsname() {
            formatter.write_str(dnsname)
        } else if let Some(uri) = self.uri() {
            formatter.write_str(uri)
        } else if let Some(ipaddress) = self.ipaddress() {
            let result = String::from_utf8_lossy(ipaddress);
            formatter.write_str(&result)
        } else {
            formatter.write_str("(empty)")
        }
    }
}

impl Stackable for GeneralName {
    type StackType = ffi::stack_st_GENERAL_NAME;
}

foreign_type_and_impl_send_sync! {
    type CType = ffi::ACCESS_DESCRIPTION;
    fn drop = ffi::ACCESS_DESCRIPTION_free;

    /// `AccessDescription` of certificate authority information.
    pub struct AccessDescription;
    /// Reference to `AccessDescription`.
    pub struct AccessDescriptionRef;
}

impl AccessDescriptionRef {
    /// Returns the access method OID.
    pub fn method(&self) -> &Asn1ObjectRef {
        unsafe { Asn1ObjectRef::from_ptr((*self.as_ptr()).method) }
    }

    // Returns the access location.
    pub fn location(&self) -> &GeneralNameRef {
        unsafe { GeneralNameRef::from_ptr((*self.as_ptr()).location) }
    }
}

impl Stackable for AccessDescription {
    type StackType = ffi::stack_st_ACCESS_DESCRIPTION;
}

foreign_type_and_impl_send_sync! {
    type CType = ffi::X509_ALGOR;
    fn drop = ffi::X509_ALGOR_free;

    /// An `X509` certificate signature algorithm.
    pub struct X509Algorithm;
    /// Reference to `X509Algorithm`.
    pub struct X509AlgorithmRef;
}

impl X509AlgorithmRef {
    /// Returns the ASN.1 OID of this algorithm.
    pub fn object(&self) -> &Asn1ObjectRef {
        unsafe {
            let mut oid = ptr::null();
            X509_ALGOR_get0(&mut oid, ptr::null_mut(), ptr::null_mut(), self.as_ptr());
            Asn1ObjectRef::from_const_ptr_opt(oid).expect("algorithm oid must not be null")
        }
    }
}

foreign_type_and_impl_send_sync! {
    type CType = ffi::X509_OBJECT;
    fn drop = X509_OBJECT_free;

    /// An `X509` or an X509 certificate revocation list.
    pub struct X509Object;
    /// Reference to `X509Object`
    pub struct X509ObjectRef;
}

impl X509ObjectRef {
    pub fn x509(&self) -> Option<&X509Ref> {
        unsafe {
            let ptr = X509_OBJECT_get0_X509(self.as_ptr());
            X509Ref::from_const_ptr_opt(ptr)
        }
    }
}

impl Stackable for X509Object {
    type StackType = ffi::stack_st_X509_OBJECT;
}

cfg_if! {
    if #[cfg(any(ossl110, libressl273))] {
        use ffi::{X509_getm_notAfter, X509_getm_notBefore, X509_up_ref, X509_get0_signature};
    } else {
        #[allow(bad_style)]
        unsafe fn X509_getm_notAfter(x: *mut ffi::X509) -> *mut ffi::ASN1_TIME {
            (*(*(*x).cert_info).validity).notAfter
        }

        #[allow(bad_style)]
        unsafe fn X509_getm_notBefore(x: *mut ffi::X509) -> *mut ffi::ASN1_TIME {
            (*(*(*x).cert_info).validity).notBefore
        }

        #[allow(bad_style)]
        unsafe fn X509_up_ref(x: *mut ffi::X509) {
            ffi::CRYPTO_add_lock(
                &mut (*x).references,
                1,
                ffi::CRYPTO_LOCK_X509,
                "mod.rs\0".as_ptr() as *const _,
                line!() as c_int,
            );
        }

        #[allow(bad_style)]
        unsafe fn X509_get0_signature(
            psig: *mut *const ffi::ASN1_BIT_STRING,
            palg: *mut *const ffi::X509_ALGOR,
            x: *const ffi::X509,
        ) {
            if !psig.is_null() {
                *psig = (*x).signature;
            }
            if !palg.is_null() {
                *palg = (*x).sig_alg;
            }
        }
    }
}

cfg_if! {
    if #[cfg(any(ossl110, libressl350))] {
        use ffi::{
            X509_ALGOR_get0, ASN1_STRING_get0_data, X509_STORE_CTX_get0_chain, X509_set1_notAfter,
            X509_set1_notBefore, X509_REQ_get_version, X509_REQ_get_subject_name,
        };
    } else {
        use ffi::{
            ASN1_STRING_data as ASN1_STRING_get0_data,
            X509_STORE_CTX_get_chain as X509_STORE_CTX_get0_chain,
            X509_set_notAfter as X509_set1_notAfter,
            X509_set_notBefore as X509_set1_notBefore,
        };

        #[allow(bad_style)]
        unsafe fn X509_REQ_get_version(x: *mut ffi::X509_REQ) -> ::libc::c_long {
            ffi::ASN1_INTEGER_get((*(*x).req_info).version)
        }

        #[allow(bad_style)]
        unsafe fn X509_REQ_get_subject_name(x: *mut ffi::X509_REQ) -> *mut ::ffi::X509_NAME {
            (*(*x).req_info).subject
        }

        #[allow(bad_style)]
        unsafe fn X509_ALGOR_get0(
            paobj: *mut *const ffi::ASN1_OBJECT,
            pptype: *mut c_int,
            pval: *mut *mut ::libc::c_void,
            alg: *const ffi::X509_ALGOR,
        ) {
            if !paobj.is_null() {
                *paobj = (*alg).algorithm;
            }
            assert!(pptype.is_null());
            assert!(pval.is_null());
        }
    }
}

cfg_if! {
    if #[cfg(any(ossl110, libressl270))] {
        use ffi::X509_OBJECT_get0_X509;
    } else {
        #[allow(bad_style)]
        unsafe fn X509_OBJECT_get0_X509(x: *mut ffi::X509_OBJECT) -> *mut ffi::X509 {
            if (*x).type_ == ffi::X509_LU_X509 {
                (*x).data.x509
            } else {
                ptr::null_mut()
            }
        }
    }
}

cfg_if! {
    if #[cfg(any(ossl110, libressl350))] {
        use ffi::X509_OBJECT_free;
    } else {
        #[allow(bad_style)]
        unsafe fn X509_OBJECT_free(x: *mut ffi::X509_OBJECT) {
            ffi::X509_OBJECT_free_contents(x);
            ffi::CRYPTO_free(x as *mut libc::c_void);
        }
    }
}
