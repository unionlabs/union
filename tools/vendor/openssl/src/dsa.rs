//! Digital Signatures
//!
//! DSA ensures a message originated from a known sender, and was not modified.
//! DSA uses asymmetrical keys and an algorithm to output a signature of the message
//! using the private key that can be validated with the public key but not be generated
//! without the private key.

use cfg_if::cfg_if;
use foreign_types::{ForeignType, ForeignTypeRef};
use libc::c_int;
use std::fmt;
use std::mem;
use std::ptr;

use crate::bn::{BigNum, BigNumRef};
use crate::error::ErrorStack;
use crate::pkey::{HasParams, HasPrivate, HasPublic, Private, Public};
use crate::util::ForeignTypeRefExt;
use crate::{cvt, cvt_p};
use openssl_macros::corresponds;

generic_foreign_type_and_impl_send_sync! {
    type CType = ffi::DSA;
    fn drop = ffi::DSA_free;

    /// Object representing DSA keys.
    ///
    /// A DSA object contains the parameters p, q, and g.  There is a private
    /// and public key.  The values p, g, and q are:
    ///
    /// * `p`: DSA prime parameter
    /// * `q`: DSA sub-prime parameter
    /// * `g`: DSA base parameter
    ///
    /// These values are used to calculate a pair of asymmetrical keys used for
    /// signing.
    ///
    /// OpenSSL documentation at [`DSA_new`]
    ///
    /// [`DSA_new`]: https://www.openssl.org/docs/man1.1.0/crypto/DSA_new.html
    ///
    /// # Examples
    ///
    /// ```
    /// use openssl::dsa::Dsa;
    /// use openssl::error::ErrorStack;
    /// use openssl::pkey::Private;
    ///
    /// fn create_dsa() -> Result<Dsa<Private>, ErrorStack> {
    ///     let sign = Dsa::generate(2048)?;
    ///     Ok(sign)
    /// }
    /// # fn main() {
    /// #    create_dsa();
    /// # }
    /// ```
    pub struct Dsa<T>;
    /// Reference to [`Dsa`].
    ///
    /// [`Dsa`]: struct.Dsa.html
    pub struct DsaRef<T>;
}

impl<T> Clone for Dsa<T> {
    fn clone(&self) -> Dsa<T> {
        (**self).to_owned()
    }
}

impl<T> ToOwned for DsaRef<T> {
    type Owned = Dsa<T>;

    fn to_owned(&self) -> Dsa<T> {
        unsafe {
            ffi::DSA_up_ref(self.as_ptr());
            Dsa::from_ptr(self.as_ptr())
        }
    }
}

impl<T> DsaRef<T>
where
    T: HasPublic,
{
    to_pem! {
        /// Serializes the public key into a PEM-encoded SubjectPublicKeyInfo structure.
        ///
        /// The output will have a header of `-----BEGIN PUBLIC KEY-----`.
        #[corresponds(PEM_write_bio_DSA_PUBKEY)]
        public_key_to_pem,
        ffi::PEM_write_bio_DSA_PUBKEY
    }

    to_der! {
        /// Serializes the public key into a DER-encoded SubjectPublicKeyInfo structure.
        #[corresponds(i2d_DSA_PUBKEY)]
        public_key_to_der,
        ffi::i2d_DSA_PUBKEY
    }

    /// Returns a reference to the public key component of `self`.
    #[corresponds(DSA_get0_key)]
    pub fn pub_key(&self) -> &BigNumRef {
        unsafe {
            let mut pub_key = ptr::null();
            DSA_get0_key(self.as_ptr(), &mut pub_key, ptr::null_mut());
            BigNumRef::from_const_ptr(pub_key)
        }
    }
}

impl<T> DsaRef<T>
where
    T: HasPrivate,
{
    private_key_to_pem! {
        /// Serializes the private key to a PEM-encoded DSAPrivateKey structure.
        ///
        /// The output will have a header of `-----BEGIN DSA PRIVATE KEY-----`.
        #[corresponds(PEM_write_bio_DSAPrivateKey)]
        private_key_to_pem,
        /// Serializes the private key to a PEM-encoded encrypted DSAPrivateKey structure.
        ///
        /// The output will have a header of `-----BEGIN DSA PRIVATE KEY-----`.
        #[corresponds(PEM_write_bio_DSAPrivateKey)]
        private_key_to_pem_passphrase,
        ffi::PEM_write_bio_DSAPrivateKey
    }

    /// Returns a reference to the private key component of `self`.
    #[corresponds(DSA_get0_key)]
    pub fn priv_key(&self) -> &BigNumRef {
        unsafe {
            let mut priv_key = ptr::null();
            DSA_get0_key(self.as_ptr(), ptr::null_mut(), &mut priv_key);
            BigNumRef::from_const_ptr(priv_key)
        }
    }
}

impl<T> DsaRef<T>
where
    T: HasParams,
{
    /// Returns the maximum size of the signature output by `self` in bytes.
    #[corresponds(DSA_size)]
    pub fn size(&self) -> u32 {
        unsafe { ffi::DSA_size(self.as_ptr()) as u32 }
    }

    /// Returns the DSA prime parameter of `self`.
    #[corresponds(DSA_get0_pqg)]
    pub fn p(&self) -> &BigNumRef {
        unsafe {
            let mut p = ptr::null();
            DSA_get0_pqg(self.as_ptr(), &mut p, ptr::null_mut(), ptr::null_mut());
            BigNumRef::from_const_ptr(p)
        }
    }

    /// Returns the DSA sub-prime parameter of `self`.
    #[corresponds(DSA_get0_pqg)]
    pub fn q(&self) -> &BigNumRef {
        unsafe {
            let mut q = ptr::null();
            DSA_get0_pqg(self.as_ptr(), ptr::null_mut(), &mut q, ptr::null_mut());
            BigNumRef::from_const_ptr(q)
        }
    }

    /// Returns the DSA base parameter of `self`.
    #[corresponds(DSA_get0_pqg)]
    pub fn g(&self) -> &BigNumRef {
        unsafe {
            let mut g = ptr::null();
            DSA_get0_pqg(self.as_ptr(), ptr::null_mut(), ptr::null_mut(), &mut g);
            BigNumRef::from_const_ptr(g)
        }
    }
}

impl Dsa<Private> {
    /// Generate a DSA key pair.
    ///
    /// Calls [`DSA_generate_parameters_ex`] to populate the `p`, `g`, and `q` values.
    /// These values are used to generate the key pair with [`DSA_generate_key`].
    ///
    /// The `bits` parameter corresponds to the length of the prime `p`.
    ///
    /// [`DSA_generate_parameters_ex`]: https://www.openssl.org/docs/man1.1.0/crypto/DSA_generate_parameters_ex.html
    /// [`DSA_generate_key`]: https://www.openssl.org/docs/man1.1.0/crypto/DSA_generate_key.html
    pub fn generate(bits: u32) -> Result<Dsa<Private>, ErrorStack> {
        ffi::init();
        unsafe {
            let dsa = Dsa::from_ptr(cvt_p(ffi::DSA_new())?);
            cvt(ffi::DSA_generate_parameters_ex(
                dsa.0,
                bits as c_int,
                ptr::null(),
                0,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
            ))?;
            cvt(ffi::DSA_generate_key(dsa.0))?;
            Ok(dsa)
        }
    }

    /// Create a DSA key pair with the given parameters
    ///
    /// `p`, `q` and `g` are the common parameters.
    /// `priv_key` is the private component of the key pair.
    /// `pub_key` is the public component of the key. Can be computed via `g^(priv_key) mod p`
    pub fn from_private_components(
        p: BigNum,
        q: BigNum,
        g: BigNum,
        priv_key: BigNum,
        pub_key: BigNum,
    ) -> Result<Dsa<Private>, ErrorStack> {
        ffi::init();
        unsafe {
            let dsa = Dsa::from_ptr(cvt_p(ffi::DSA_new())?);
            cvt(DSA_set0_pqg(dsa.0, p.as_ptr(), q.as_ptr(), g.as_ptr()))?;
            mem::forget((p, q, g));
            cvt(DSA_set0_key(dsa.0, pub_key.as_ptr(), priv_key.as_ptr()))?;
            mem::forget((pub_key, priv_key));
            Ok(dsa)
        }
    }
}

impl Dsa<Public> {
    from_pem! {
        /// Decodes a PEM-encoded SubjectPublicKeyInfo structure containing a DSA key.
        ///
        /// The input should have a header of `-----BEGIN PUBLIC KEY-----`.
        #[corresponds(PEM_read_bio_DSA_PUBKEY)]
        public_key_from_pem,
        Dsa<Public>,
        ffi::PEM_read_bio_DSA_PUBKEY
    }

    from_der! {
        /// Decodes a DER-encoded SubjectPublicKeyInfo structure containing a DSA key.
        #[corresponds(d2i_DSA_PUBKEY)]
        public_key_from_der,
        Dsa<Public>,
        ffi::d2i_DSA_PUBKEY
    }

    /// Create a new DSA key with only public components.
    ///
    /// `p`, `q` and `g` are the common parameters.
    /// `pub_key` is the public component of the key.
    pub fn from_public_components(
        p: BigNum,
        q: BigNum,
        g: BigNum,
        pub_key: BigNum,
    ) -> Result<Dsa<Public>, ErrorStack> {
        ffi::init();
        unsafe {
            let dsa = Dsa::from_ptr(cvt_p(ffi::DSA_new())?);
            cvt(DSA_set0_pqg(dsa.0, p.as_ptr(), q.as_ptr(), g.as_ptr()))?;
            mem::forget((p, q, g));
            cvt(DSA_set0_key(dsa.0, pub_key.as_ptr(), ptr::null_mut()))?;
            mem::forget(pub_key);
            Ok(dsa)
        }
    }
}

impl<T> fmt::Debug for Dsa<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DSA")
    }
}

cfg_if! {
    if #[cfg(any(ossl110, libressl273))] {
        use ffi::{DSA_get0_key, DSA_get0_pqg, DSA_set0_key, DSA_set0_pqg};
    } else {
        #[allow(bad_style)]
        unsafe fn DSA_get0_pqg(
            d: *mut ffi::DSA,
            p: *mut *const ffi::BIGNUM,
            q: *mut *const ffi::BIGNUM,
            g: *mut *const ffi::BIGNUM)
        {
            if !p.is_null() {
                *p = (*d).p;
            }
            if !q.is_null() {
                *q = (*d).q;
            }
            if !g.is_null() {
                *g = (*d).g;
            }
        }

        #[allow(bad_style)]
        unsafe fn DSA_get0_key(
            d: *mut ffi::DSA,
            pub_key: *mut *const ffi::BIGNUM,
            priv_key: *mut *const ffi::BIGNUM)
        {
            if !pub_key.is_null() {
                *pub_key = (*d).pub_key;
            }
            if !priv_key.is_null() {
                *priv_key = (*d).priv_key;
            }
        }

        #[allow(bad_style)]
        unsafe fn DSA_set0_key(
            d: *mut ffi::DSA,
            pub_key: *mut ffi::BIGNUM,
            priv_key: *mut ffi::BIGNUM) -> c_int
        {
            (*d).pub_key = pub_key;
            (*d).priv_key = priv_key;
            1
        }

        #[allow(bad_style)]
        unsafe fn DSA_set0_pqg(
            d: *mut ffi::DSA,
            p: *mut ffi::BIGNUM,
            q: *mut ffi::BIGNUM,
            g: *mut ffi::BIGNUM) -> c_int
        {
            (*d).p = p;
            (*d).q = q;
            (*d).g = g;
            1
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::bn::BigNumContext;
    use crate::hash::MessageDigest;
    use crate::pkey::PKey;
    use crate::sign::{Signer, Verifier};

    #[test]
    pub fn test_generate() {
        Dsa::generate(1024).unwrap();
    }

    #[test]
    fn test_pubkey_generation() {
        let dsa = Dsa::generate(1024).unwrap();
        let p = dsa.p();
        let g = dsa.g();
        let priv_key = dsa.priv_key();
        let pub_key = dsa.pub_key();
        let mut ctx = BigNumContext::new().unwrap();
        let mut calc = BigNum::new().unwrap();
        calc.mod_exp(g, priv_key, p, &mut ctx).unwrap();
        assert_eq!(&calc, pub_key)
    }

    #[test]
    fn test_priv_key_from_parts() {
        let p = BigNum::from_u32(283).unwrap();
        let q = BigNum::from_u32(47).unwrap();
        let g = BigNum::from_u32(60).unwrap();
        let priv_key = BigNum::from_u32(15).unwrap();
        let pub_key = BigNum::from_u32(207).unwrap();

        let dsa = Dsa::from_private_components(p, q, g, priv_key, pub_key).unwrap();
        assert_eq!(dsa.pub_key(), &BigNum::from_u32(207).unwrap());
        assert_eq!(dsa.priv_key(), &BigNum::from_u32(15).unwrap());
        assert_eq!(dsa.p(), &BigNum::from_u32(283).unwrap());
        assert_eq!(dsa.q(), &BigNum::from_u32(47).unwrap());
        assert_eq!(dsa.g(), &BigNum::from_u32(60).unwrap());
    }

    #[test]
    fn test_pub_key_from_parts() {
        let p = BigNum::from_u32(283).unwrap();
        let q = BigNum::from_u32(47).unwrap();
        let g = BigNum::from_u32(60).unwrap();
        let pub_key = BigNum::from_u32(207).unwrap();

        let dsa = Dsa::from_public_components(p, q, g, pub_key).unwrap();
        assert_eq!(dsa.pub_key(), &BigNum::from_u32(207).unwrap());
        assert_eq!(dsa.p(), &BigNum::from_u32(283).unwrap());
        assert_eq!(dsa.q(), &BigNum::from_u32(47).unwrap());
        assert_eq!(dsa.g(), &BigNum::from_u32(60).unwrap());
    }

    #[test]
    fn test_signature() {
        const TEST_DATA: &[u8] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let dsa_ref = Dsa::generate(1024).unwrap();

        let p = dsa_ref.p();
        let q = dsa_ref.q();
        let g = dsa_ref.g();

        let pub_key = dsa_ref.pub_key();
        let priv_key = dsa_ref.priv_key();

        let priv_key = Dsa::from_private_components(
            BigNumRef::to_owned(p).unwrap(),
            BigNumRef::to_owned(q).unwrap(),
            BigNumRef::to_owned(g).unwrap(),
            BigNumRef::to_owned(priv_key).unwrap(),
            BigNumRef::to_owned(pub_key).unwrap(),
        )
        .unwrap();
        let priv_key = PKey::from_dsa(priv_key).unwrap();

        let pub_key = Dsa::from_public_components(
            BigNumRef::to_owned(p).unwrap(),
            BigNumRef::to_owned(q).unwrap(),
            BigNumRef::to_owned(g).unwrap(),
            BigNumRef::to_owned(pub_key).unwrap(),
        )
        .unwrap();
        let pub_key = PKey::from_dsa(pub_key).unwrap();

        let mut signer = Signer::new(MessageDigest::sha256(), &priv_key).unwrap();
        signer.update(TEST_DATA).unwrap();

        let signature = signer.sign_to_vec().unwrap();
        let mut verifier = Verifier::new(MessageDigest::sha256(), &pub_key).unwrap();
        verifier.update(TEST_DATA).unwrap();
        assert!(verifier.verify(&signature[..]).unwrap());
    }

    #[test]
    #[allow(clippy::redundant_clone)]
    fn clone() {
        let key = Dsa::generate(2048).unwrap();
        drop(key.clone());
    }
}
