//! The asymmetric encryption context.
//!
//! # Examples
//!
//! Encrypt data with RSA
//!
//! ```
//! use openssl::rsa::Rsa;
//! use openssl::pkey::PKey;
//! use openssl::pkey_ctx::PkeyCtx;
//!
//! let key = Rsa::generate(4096).unwrap();
//! let key = PKey::from_rsa(key).unwrap();
//!
//! let mut ctx = PkeyCtx::new(&key).unwrap();
//! ctx.encrypt_init().unwrap();
//!
//! let data = b"Some Crypto Text";
//! let mut ciphertext = vec![];
//! ctx.encrypt_to_vec(data, &mut ciphertext).unwrap();
//! ```
//!
//! Generate a CMAC key
//!
//! ```
//! use openssl::pkey_ctx::PkeyCtx;
//! use openssl::pkey::Id;
//! use openssl::cipher::Cipher;
//!
//! let mut ctx = PkeyCtx::new_id(Id::CMAC).unwrap();
//! ctx.keygen_init().unwrap();
//! ctx.set_keygen_cipher(Cipher::aes_128_cbc()).unwrap();
//! ctx.set_keygen_mac_key(b"0123456789abcdef").unwrap();
//! let cmac_key = ctx.keygen().unwrap();
//! ```
//!
//! Sign and verify data with RSA
//!
//! ```
//! use openssl::pkey_ctx::PkeyCtx;
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
//! let mut ctx = PkeyCtx::new(&key).unwrap();
//! ctx.sign_init().unwrap();
//! let mut signature = vec![];
//! ctx.sign_to_vec(text, &mut signature).unwrap();
//!
//! // Verify the signature.
//! let mut ctx = PkeyCtx::new(&key).unwrap();
//! ctx.verify_init().unwrap();
//! let valid = ctx.verify(text, &signature).unwrap();
//! assert!(valid);
//! ```
use crate::cipher::CipherRef;
use crate::error::ErrorStack;
use crate::md::MdRef;
use crate::pkey::{HasPrivate, HasPublic, Id, PKey, PKeyRef, Private};
use crate::rsa::Padding;
use crate::{cvt, cvt_n, cvt_p};
use foreign_types::{ForeignType, ForeignTypeRef};
use libc::c_int;
use openssl_macros::corresponds;
use std::convert::TryFrom;
use std::ptr;

/// HKDF modes of operation.
#[cfg(ossl111)]
pub struct HkdfMode(c_int);

#[cfg(ossl111)]
impl HkdfMode {
    pub const EXTRACT_THEN_EXPAND: Self = HkdfMode(ffi::EVP_PKEY_HKDEF_MODE_EXTRACT_AND_EXPAND);
    pub const EXTRACT_ONLY: Self = HkdfMode(ffi::EVP_PKEY_HKDEF_MODE_EXTRACT_ONLY);
    pub const EXPAND_ONLY: Self = HkdfMode(ffi::EVP_PKEY_HKDEF_MODE_EXPAND_ONLY);
}

generic_foreign_type_and_impl_send_sync! {
    type CType = ffi::EVP_PKEY_CTX;
    fn drop = ffi::EVP_PKEY_CTX_free;

    /// A context object which can perform asymmetric cryptography operations.
    pub struct PkeyCtx<T>;
    /// A reference to a [`PkeyCtx`].
    pub struct PkeyCtxRef<T>;
}

impl<T> PkeyCtx<T> {
    /// Creates a new pkey context using the provided key.
    #[corresponds(EVP_PKEY_CTX_new)]
    #[inline]
    pub fn new(pkey: &PKeyRef<T>) -> Result<Self, ErrorStack> {
        unsafe {
            let ptr = cvt_p(ffi::EVP_PKEY_CTX_new(pkey.as_ptr(), ptr::null_mut()))?;
            Ok(PkeyCtx::from_ptr(ptr))
        }
    }
}

impl PkeyCtx<()> {
    /// Creates a new pkey context for the specified algorithm ID.
    #[corresponds(EVP_PKEY_new_id)]
    #[inline]
    pub fn new_id(id: Id) -> Result<Self, ErrorStack> {
        unsafe {
            let ptr = cvt_p(ffi::EVP_PKEY_CTX_new_id(id.as_raw(), ptr::null_mut()))?;
            Ok(PkeyCtx::from_ptr(ptr))
        }
    }
}

impl<T> PkeyCtxRef<T>
where
    T: HasPublic,
{
    /// Prepares the context for encryption using the public key.
    #[corresponds(EVP_PKEY_encrypt_init)]
    #[inline]
    pub fn encrypt_init(&mut self) -> Result<(), ErrorStack> {
        unsafe {
            cvt(ffi::EVP_PKEY_encrypt_init(self.as_ptr()))?;
        }

        Ok(())
    }

    /// Prepares the context for signature verification using the public key.
    #[corresponds(EVP_PKEY_verify_init)]
    #[inline]
    pub fn verify_init(&mut self) -> Result<(), ErrorStack> {
        unsafe {
            cvt(ffi::EVP_PKEY_verify_init(self.as_ptr()))?;
        }

        Ok(())
    }

    /// Encrypts data using the public key.
    ///
    /// If `to` is set to `None`, an upper bound on the number of bytes required for the output buffer will be
    /// returned.
    #[corresponds(EVP_PKEY_encrypt)]
    #[inline]
    pub fn encrypt(&mut self, from: &[u8], to: Option<&mut [u8]>) -> Result<usize, ErrorStack> {
        let mut written = to.as_ref().map_or(0, |b| b.len());
        unsafe {
            cvt(ffi::EVP_PKEY_encrypt(
                self.as_ptr(),
                to.map_or(ptr::null_mut(), |b| b.as_mut_ptr()),
                &mut written,
                from.as_ptr(),
                from.len(),
            ))?;
        }

        Ok(written)
    }

    /// Like [`Self::encrypt`] but appends ciphertext to a [`Vec`].
    pub fn encrypt_to_vec(&mut self, from: &[u8], out: &mut Vec<u8>) -> Result<usize, ErrorStack> {
        let base = out.len();
        let len = self.encrypt(from, None)?;
        out.resize(base + len, 0);
        let len = self.encrypt(from, Some(&mut out[base..]))?;
        out.truncate(base + len);
        Ok(len)
    }

    /// Verifies the signature of data using the public key.
    ///
    /// Returns `Ok(true)` if the signature is valid, `Ok(false)` if the signature is invalid, and `Err` if an error
    /// occurred.
    ///
    /// # Note
    ///
    /// This verifies the signature of the *raw* data. It is more common to compute and verify the signature of the
    /// cryptographic hash of an arbitrary amount of data. The [`MdCtx`](crate::md_ctx::MdCtx) type can be used to do
    /// that.
    #[corresponds(EVP_PKEY_verify)]
    #[inline]
    pub fn verify(&mut self, data: &[u8], sig: &[u8]) -> Result<bool, ErrorStack> {
        unsafe {
            let r = cvt_n(ffi::EVP_PKEY_verify(
                self.as_ptr(),
                sig.as_ptr(),
                sig.len(),
                data.as_ptr(),
                data.len(),
            ))?;
            Ok(r == 1)
        }
    }
}

impl<T> PkeyCtxRef<T>
where
    T: HasPrivate,
{
    /// Prepares the context for decryption using the private key.
    #[corresponds(EVP_PKEY_decrypt_init)]
    #[inline]
    pub fn decrypt_init(&mut self) -> Result<(), ErrorStack> {
        unsafe {
            cvt(ffi::EVP_PKEY_decrypt_init(self.as_ptr()))?;
        }

        Ok(())
    }

    /// Prepares the context for signing using the private key.
    #[corresponds(EVP_PKEY_sign_init)]
    #[inline]
    pub fn sign_init(&mut self) -> Result<(), ErrorStack> {
        unsafe {
            cvt(ffi::EVP_PKEY_sign_init(self.as_ptr()))?;
        }

        Ok(())
    }

    /// Sets the peer key used for secret derivation.
    #[corresponds(EVP_PKEY_derive_set_peer)]
    pub fn derive_set_peer<U>(&mut self, key: &PKeyRef<U>) -> Result<(), ErrorStack>
    where
        U: HasPublic,
    {
        unsafe {
            cvt(ffi::EVP_PKEY_derive_set_peer(self.as_ptr(), key.as_ptr()))?;
        }

        Ok(())
    }

    /// Decrypts data using the private key.
    ///
    /// If `to` is set to `None`, an upper bound on the number of bytes required for the output buffer will be
    /// returned.
    #[corresponds(EVP_PKEY_decrypt)]
    #[inline]
    pub fn decrypt(&mut self, from: &[u8], to: Option<&mut [u8]>) -> Result<usize, ErrorStack> {
        let mut written = to.as_ref().map_or(0, |b| b.len());
        unsafe {
            cvt(ffi::EVP_PKEY_decrypt(
                self.as_ptr(),
                to.map_or(ptr::null_mut(), |b| b.as_mut_ptr()),
                &mut written,
                from.as_ptr(),
                from.len(),
            ))?;
        }

        Ok(written)
    }

    /// Like [`Self::decrypt`] but appends plaintext to a [`Vec`].
    pub fn decrypt_to_vec(&mut self, from: &[u8], out: &mut Vec<u8>) -> Result<usize, ErrorStack> {
        let base = out.len();
        let len = self.decrypt(from, None)?;
        out.resize(base + len, 0);
        let len = self.decrypt(from, Some(&mut out[base..]))?;
        out.truncate(base + len);
        Ok(len)
    }

    /// Signs the contents of `data`.
    ///
    /// If `sig` is set to `None`, an upper bound on the number of bytes required for the output buffer will be
    /// returned.
    ///
    /// # Note
    ///
    /// This computes the signature of the *raw* bytes of `data`. It is more common to sign the cryptographic hash of
    /// an arbitrary amount of data. The [`MdCtx`](crate::md_ctx::MdCtx) type can be used to do that.
    #[corresponds(EVP_PKEY_sign)]
    #[inline]
    pub fn sign(&mut self, data: &[u8], sig: Option<&mut [u8]>) -> Result<usize, ErrorStack> {
        let mut written = sig.as_ref().map_or(0, |b| b.len());
        unsafe {
            cvt(ffi::EVP_PKEY_sign(
                self.as_ptr(),
                sig.map_or(ptr::null_mut(), |b| b.as_mut_ptr()),
                &mut written,
                data.as_ptr(),
                data.len(),
            ))?;
        }

        Ok(written)
    }

    /// Like [`Self::sign`] but appends the signature to a [`Vec`].
    pub fn sign_to_vec(&mut self, data: &[u8], sig: &mut Vec<u8>) -> Result<usize, ErrorStack> {
        let base = sig.len();
        let len = self.sign(data, None)?;
        sig.resize(base + len, 0);
        let len = self.sign(data, Some(&mut sig[base..]))?;
        sig.truncate(base + len);
        Ok(len)
    }
}

impl<T> PkeyCtxRef<T> {
    /// Prepares the context for shared secret derivation.
    #[corresponds(EVP_PKEY_derive_init)]
    #[inline]
    pub fn derive_init(&mut self) -> Result<(), ErrorStack> {
        unsafe {
            cvt(ffi::EVP_PKEY_derive_init(self.as_ptr()))?;
        }

        Ok(())
    }

    /// Prepares the context for key generation.
    #[corresponds(EVP_PKEY_keygen_init)]
    #[inline]
    pub fn keygen_init(&mut self) -> Result<(), ErrorStack> {
        unsafe {
            cvt(ffi::EVP_PKEY_keygen_init(self.as_ptr()))?;
        }

        Ok(())
    }

    /// Returns the RSA padding mode in use.
    ///
    /// This is only useful for RSA keys.
    #[corresponds(EVP_PKEY_CTX_get_rsa_padding)]
    #[inline]
    pub fn rsa_padding(&self) -> Result<Padding, ErrorStack> {
        let mut pad = 0;
        unsafe {
            cvt(ffi::EVP_PKEY_CTX_get_rsa_padding(self.as_ptr(), &mut pad))?;
        }

        Ok(Padding::from_raw(pad))
    }

    /// Sets the RSA padding mode.
    ///
    /// This is only useful for RSA keys.
    #[corresponds(EVP_PKEY_CTX_set_rsa_padding)]
    #[inline]
    pub fn set_rsa_padding(&mut self, padding: Padding) -> Result<(), ErrorStack> {
        unsafe {
            cvt(ffi::EVP_PKEY_CTX_set_rsa_padding(
                self.as_ptr(),
                padding.as_raw(),
            ))?;
        }

        Ok(())
    }

    /// Sets the RSA MGF1 algorithm.
    ///
    /// This is only useful for RSA keys.
    #[corresponds(EVP_PKEY_CTX_set_rsa_mgf1_md)]
    #[inline]
    pub fn set_rsa_mgf1_md(&mut self, md: &MdRef) -> Result<(), ErrorStack> {
        unsafe {
            cvt(ffi::EVP_PKEY_CTX_set_rsa_mgf1_md(
                self.as_ptr(),
                md.as_ptr(),
            ))?;
        }

        Ok(())
    }

    /// Sets the RSA OAEP algorithm.
    ///
    /// This is only useful for RSA keys.
    #[corresponds(EVP_PKEY_CTX_set_rsa_oaep_md)]
    #[cfg(any(ossl102, libressl310))]
    #[inline]
    pub fn set_rsa_oaep_md(&mut self, md: &MdRef) -> Result<(), ErrorStack> {
        unsafe {
            cvt(ffi::EVP_PKEY_CTX_set_rsa_oaep_md(
                self.as_ptr(),
                md.as_ptr() as *mut _,
            ))?;
        }

        Ok(())
    }

    /// Sets the RSA OAEP label.
    ///
    /// This is only useful for RSA keys.
    #[corresponds(EVP_PKEY_CTX_set0_rsa_oaep_label)]
    #[cfg(any(ossl102, libressl310))]
    pub fn set_rsa_oaep_label(&mut self, label: &[u8]) -> Result<(), ErrorStack> {
        let len = c_int::try_from(label.len()).unwrap();

        unsafe {
            let p = ffi::OPENSSL_malloc(label.len() as _);
            ptr::copy_nonoverlapping(label.as_ptr(), p as *mut _, label.len());

            let r = cvt(ffi::EVP_PKEY_CTX_set0_rsa_oaep_label(self.as_ptr(), p, len));
            if r.is_err() {
                ffi::OPENSSL_free(p);
            }
            r?;
        }

        Ok(())
    }

    /// Sets the cipher used during key generation.
    #[corresponds(EVP_PKEY_CTX_ctrl)]
    #[inline]
    pub fn set_keygen_cipher(&mut self, cipher: &CipherRef) -> Result<(), ErrorStack> {
        unsafe {
            cvt(ffi::EVP_PKEY_CTX_ctrl(
                self.as_ptr(),
                -1,
                ffi::EVP_PKEY_OP_KEYGEN,
                ffi::EVP_PKEY_CTRL_CIPHER,
                0,
                cipher.as_ptr() as *mut _,
            ))?;
        }

        Ok(())
    }

    /// Sets the key MAC key used during key generation.
    #[corresponds(EVP_PKEY_CTX_ctrl)]
    #[inline]
    pub fn set_keygen_mac_key(&mut self, key: &[u8]) -> Result<(), ErrorStack> {
        let len = c_int::try_from(key.len()).unwrap();

        unsafe {
            cvt(ffi::EVP_PKEY_CTX_ctrl(
                self.as_ptr(),
                -1,
                ffi::EVP_PKEY_OP_KEYGEN,
                ffi::EVP_PKEY_CTRL_SET_MAC_KEY,
                len,
                key.as_ptr() as *mut _,
            ))?;
        }

        Ok(())
    }

    /// Sets the digest used for HKDF derivation.
    ///
    /// Requires OpenSSL 1.1.0 or newer.
    #[corresponds(EVP_PKEY_CTX_set_hkdf_md)]
    #[cfg(ossl110)]
    #[inline]
    pub fn set_hkdf_md(&mut self, digest: &MdRef) -> Result<(), ErrorStack> {
        unsafe {
            cvt(ffi::EVP_PKEY_CTX_set_hkdf_md(
                self.as_ptr(),
                digest.as_ptr(),
            ))?;
        }

        Ok(())
    }

    /// Sets the HKDF mode of operation.
    ///
    /// Defaults to [`HkdfMode::EXTRACT_THEN_EXPAND`].
    ///
    /// Requires OpenSSL 1.1.1 or newer.
    #[corresponds(EVP_PKEY_CTX_set_hkdf_mode)]
    #[cfg(ossl111)]
    #[inline]
    pub fn set_hkdf_mode(&mut self, mode: HkdfMode) -> Result<(), ErrorStack> {
        unsafe {
            cvt(ffi::EVP_PKEY_CTX_set_hkdf_mode(self.as_ptr(), mode.0))?;
        }

        Ok(())
    }

    /// Sets the input keying material for HKDF generation.
    ///
    /// Requires OpenSSL 1.1.0 or newer.
    #[corresponds(EVP_PKEY_CTX_set1_hkdf_key)]
    #[cfg(ossl110)]
    #[inline]
    pub fn set_hkdf_key(&mut self, key: &[u8]) -> Result<(), ErrorStack> {
        let len = c_int::try_from(key.len()).unwrap();

        unsafe {
            cvt(ffi::EVP_PKEY_CTX_set1_hkdf_key(
                self.as_ptr(),
                key.as_ptr(),
                len,
            ))?;
        }

        Ok(())
    }

    /// Sets the salt value for HKDF generation.
    ///
    /// Requires OpenSSL 1.1.0 or newer.
    #[corresponds(EVP_PKEY_CTX_set1_hkdf_salt)]
    #[cfg(ossl110)]
    #[inline]
    pub fn set_hkdf_salt(&mut self, salt: &[u8]) -> Result<(), ErrorStack> {
        let len = c_int::try_from(salt.len()).unwrap();

        unsafe {
            cvt(ffi::EVP_PKEY_CTX_set1_hkdf_salt(
                self.as_ptr(),
                salt.as_ptr(),
                len,
            ))?;
        }

        Ok(())
    }

    /// Appends info bytes for HKDF generation.
    ///
    /// Requires OpenSSL 1.1.0 or newer.
    #[corresponds(EVP_PKEY_CTX_add1_hkdf_info)]
    #[cfg(ossl110)]
    #[inline]
    pub fn add_hkdf_info(&mut self, info: &[u8]) -> Result<(), ErrorStack> {
        let len = c_int::try_from(info.len()).unwrap();

        unsafe {
            cvt(ffi::EVP_PKEY_CTX_add1_hkdf_info(
                self.as_ptr(),
                info.as_ptr(),
                len,
            ))?;
        }

        Ok(())
    }

    /// Derives a shared secret between two keys.
    ///
    /// If `buf` is set to `None`, an upper bound on the number of bytes required for the buffer will be returned.
    #[corresponds(EVP_PKEY_derive)]
    pub fn derive(&mut self, buf: Option<&mut [u8]>) -> Result<usize, ErrorStack> {
        let mut len = buf.as_ref().map_or(0, |b| b.len());
        unsafe {
            cvt(ffi::EVP_PKEY_derive(
                self.as_ptr(),
                buf.map_or(ptr::null_mut(), |b| b.as_mut_ptr()),
                &mut len,
            ))?;
        }

        Ok(len)
    }

    /// Like [`Self::derive`] but appends the secret to a [`Vec`].
    pub fn derive_to_vec(&mut self, buf: &mut Vec<u8>) -> Result<usize, ErrorStack> {
        let base = buf.len();
        let len = self.derive(None)?;
        buf.resize(base + len, 0);
        let len = self.derive(Some(&mut buf[base..]))?;
        buf.truncate(base + len);
        Ok(len)
    }

    /// Generates a new public/private keypair.
    #[corresponds(EVP_PKEY_keygen)]
    #[inline]
    pub fn keygen(&mut self) -> Result<PKey<Private>, ErrorStack> {
        unsafe {
            let mut key = ptr::null_mut();
            cvt(ffi::EVP_PKEY_keygen(self.as_ptr(), &mut key))?;
            Ok(PKey::from_ptr(key))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cipher::Cipher;
    use crate::ec::{EcGroup, EcKey};
    #[cfg(any(ossl102, libressl310))]
    use crate::md::Md;
    use crate::nid::Nid;
    use crate::pkey::PKey;
    use crate::rsa::Rsa;

    #[test]
    fn rsa() {
        let key = include_bytes!("../test/rsa.pem");
        let rsa = Rsa::private_key_from_pem(key).unwrap();
        let pkey = PKey::from_rsa(rsa).unwrap();

        let mut ctx = PkeyCtx::new(&pkey).unwrap();
        ctx.encrypt_init().unwrap();
        ctx.set_rsa_padding(Padding::PKCS1).unwrap();

        let pt = "hello world".as_bytes();
        let mut ct = vec![];
        ctx.encrypt_to_vec(pt, &mut ct).unwrap();

        ctx.decrypt_init().unwrap();
        ctx.set_rsa_padding(Padding::PKCS1).unwrap();

        let mut out = vec![];
        ctx.decrypt_to_vec(&ct, &mut out).unwrap();

        assert_eq!(pt, out);
    }

    #[test]
    #[cfg(any(ossl102, libressl310))]
    fn rsa_oaep() {
        let key = include_bytes!("../test/rsa.pem");
        let rsa = Rsa::private_key_from_pem(key).unwrap();
        let pkey = PKey::from_rsa(rsa).unwrap();

        let mut ctx = PkeyCtx::new(&pkey).unwrap();
        ctx.encrypt_init().unwrap();
        ctx.set_rsa_padding(Padding::PKCS1_OAEP).unwrap();
        ctx.set_rsa_oaep_md(Md::sha256()).unwrap();
        ctx.set_rsa_mgf1_md(Md::sha256()).unwrap();

        let pt = "hello world".as_bytes();
        let mut ct = vec![];
        ctx.encrypt_to_vec(pt, &mut ct).unwrap();

        ctx.decrypt_init().unwrap();
        ctx.set_rsa_padding(Padding::PKCS1_OAEP).unwrap();
        ctx.set_rsa_oaep_md(Md::sha256()).unwrap();
        ctx.set_rsa_mgf1_md(Md::sha256()).unwrap();

        let mut out = vec![];
        ctx.decrypt_to_vec(&ct, &mut out).unwrap();

        assert_eq!(pt, out);
    }

    #[test]
    fn derive() {
        let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap();
        let key1 = EcKey::generate(&group).unwrap();
        let key1 = PKey::from_ec_key(key1).unwrap();
        let key2 = EcKey::generate(&group).unwrap();
        let key2 = PKey::from_ec_key(key2).unwrap();

        let mut ctx = PkeyCtx::new(&key1).unwrap();
        ctx.derive_init().unwrap();
        ctx.derive_set_peer(&key2).unwrap();

        let mut buf = vec![];
        ctx.derive_to_vec(&mut buf).unwrap();
    }

    #[test]
    fn cmac_keygen() {
        let mut ctx = PkeyCtx::new_id(Id::CMAC).unwrap();
        ctx.keygen_init().unwrap();
        ctx.set_keygen_cipher(Cipher::aes_128_cbc()).unwrap();
        ctx.set_keygen_mac_key(&hex::decode("9294727a3638bb1c13f48ef8158bfc9d").unwrap())
            .unwrap();
        ctx.keygen().unwrap();
    }

    #[test]
    #[cfg(ossl110)]
    fn hkdf() {
        let mut ctx = PkeyCtx::new_id(Id::HKDF).unwrap();
        ctx.derive_init().unwrap();
        ctx.set_hkdf_md(Md::sha256()).unwrap();
        ctx.set_hkdf_key(&hex::decode("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b").unwrap())
            .unwrap();
        ctx.set_hkdf_salt(&hex::decode("000102030405060708090a0b0c").unwrap())
            .unwrap();
        ctx.add_hkdf_info(&hex::decode("f0f1f2f3f4f5f6f7f8f9").unwrap())
            .unwrap();
        let mut out = [0; 42];
        ctx.derive(Some(&mut out)).unwrap();

        assert_eq!(
            &out[..],
            hex::decode("3cb25f25faacd57a90434f64d0362f2a2d2d0a90cf1a5a4c5db02d56ecc4c5bf34007208d5b887185865")
                .unwrap()
        );
    }

    #[test]
    #[cfg(ossl111)]
    fn hkdf_expand() {
        let mut ctx = PkeyCtx::new_id(Id::HKDF).unwrap();
        ctx.derive_init().unwrap();
        ctx.set_hkdf_mode(HkdfMode::EXPAND_ONLY).unwrap();
        ctx.set_hkdf_md(Md::sha256()).unwrap();
        ctx.set_hkdf_key(
            &hex::decode("077709362c2e32df0ddc3f0dc47bba6390b6c73bb50f9c3122ec844ad7c2b3e5")
                .unwrap(),
        )
        .unwrap();
        ctx.add_hkdf_info(&hex::decode("f0f1f2f3f4f5f6f7f8f9").unwrap())
            .unwrap();
        let mut out = [0; 42];
        ctx.derive(Some(&mut out)).unwrap();

        assert_eq!(
            &out[..],
            hex::decode("3cb25f25faacd57a90434f64d0362f2a2d2d0a90cf1a5a4c5db02d56ecc4c5bf34007208d5b887185865")
                .unwrap()
        );
    }

    #[test]
    #[cfg(ossl111)]
    fn hkdf_extract() {
        let mut ctx = PkeyCtx::new_id(Id::HKDF).unwrap();
        ctx.derive_init().unwrap();
        ctx.set_hkdf_mode(HkdfMode::EXTRACT_ONLY).unwrap();
        ctx.set_hkdf_md(Md::sha256()).unwrap();
        ctx.set_hkdf_key(&hex::decode("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b").unwrap())
            .unwrap();
        ctx.set_hkdf_salt(&hex::decode("000102030405060708090a0b0c").unwrap())
            .unwrap();
        let mut out = vec![];
        ctx.derive_to_vec(&mut out).unwrap();

        assert_eq!(
            &out[..],
            hex::decode("077709362c2e32df0ddc3f0dc47bba6390b6c73bb50f9c3122ec844ad7c2b3e5")
                .unwrap()
        );
    }

    #[test]
    fn verify_fail() {
        let key1 = Rsa::generate(4096).unwrap();
        let key1 = PKey::from_rsa(key1).unwrap();

        let data = b"Some Crypto Text";

        let mut ctx = PkeyCtx::new(&key1).unwrap();
        ctx.sign_init().unwrap();
        let mut signature = vec![];
        ctx.sign_to_vec(data, &mut signature).unwrap();

        let bad_data = b"Some Crypto text";

        ctx.verify_init().unwrap();
        let valid = ctx.verify(bad_data, &signature).unwrap();
        assert!(!valid);
    }
}
