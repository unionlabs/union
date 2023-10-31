//! The symmetric encryption context.
//!
//! # Examples
//!
//! Encrypt data with AES128 CBC
//!
//! ```
//! use openssl::cipher::Cipher;
//! use openssl::cipher_ctx::CipherCtx;
//!
//! let cipher = Cipher::aes_128_cbc();
//! let data = b"Some Crypto Text";
//! let key = b"\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F";
//! let iv = b"\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07";
//!
//! let mut ctx = CipherCtx::new().unwrap();
//! ctx.encrypt_init(Some(cipher), Some(key), Some(iv)).unwrap();
//!
//! let mut ciphertext = vec![];
//! ctx.cipher_update_vec(data, &mut ciphertext).unwrap();
//! ctx.cipher_final_vec(&mut ciphertext).unwrap();
//!
//! assert_eq!(
//!     b"\xB4\xB9\xE7\x30\xD6\xD6\xF7\xDE\x77\x3F\x1C\xFF\xB3\x3E\x44\x5A\x91\xD7\x27\x62\x87\x4D\
//!       \xFB\x3C\x5E\xC4\x59\x72\x4A\xF4\x7C\xA1",
//!     &ciphertext[..],
//! );
//! ```
//!
//! Decrypt data with AES128 CBC
//!
//! ```
//! use openssl::cipher::Cipher;
//! use openssl::cipher_ctx::CipherCtx;
//!
//! let cipher = Cipher::aes_128_cbc();
//! let data = b"\xB4\xB9\xE7\x30\xD6\xD6\xF7\xDE\x77\x3F\x1C\xFF\xB3\x3E\x44\x5A\x91\xD7\x27\x62\
//!              \x87\x4D\xFB\x3C\x5E\xC4\x59\x72\x4A\xF4\x7C\xA1";
//! let key = b"\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F";
//! let iv = b"\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07";
//!
//! let mut ctx = CipherCtx::new().unwrap();
//! ctx.decrypt_init(Some(cipher), Some(key), Some(iv)).unwrap();
//!
//! let mut plaintext = vec![];
//! ctx.cipher_update_vec(data, &mut plaintext).unwrap();
//! ctx.cipher_final_vec(&mut plaintext).unwrap();
//!
//! assert_eq!(b"Some Crypto Text", &plaintext[..]);
//! ```
#![warn(missing_docs)]

use crate::cipher::CipherRef;
use crate::error::ErrorStack;
use crate::pkey::{HasPrivate, HasPublic, PKey, PKeyRef};
use crate::{cvt, cvt_p};
use cfg_if::cfg_if;
use foreign_types::{ForeignType, ForeignTypeRef};
use libc::{c_int, c_uchar};
use openssl_macros::corresponds;
use std::convert::TryFrom;
use std::ptr;

cfg_if! {
    if #[cfg(ossl300)] {
        use ffi::EVP_CIPHER_CTX_get0_cipher;
    } else {
        use ffi::EVP_CIPHER_CTX_cipher as EVP_CIPHER_CTX_get0_cipher;
    }
}

foreign_type_and_impl_send_sync! {
    type CType = ffi::EVP_CIPHER_CTX;
    fn drop = ffi::EVP_CIPHER_CTX_free;

    /// A context object used to perform symmetric encryption operations.
    pub struct CipherCtx;
    /// A reference to a [`CipherCtx`].
    pub struct CipherCtxRef;
}

impl CipherCtx {
    /// Creates a new context.
    #[corresponds(EVP_CIPHER_CTX_new)]
    pub fn new() -> Result<Self, ErrorStack> {
        ffi::init();

        unsafe {
            let ptr = cvt_p(ffi::EVP_CIPHER_CTX_new())?;
            Ok(CipherCtx::from_ptr(ptr))
        }
    }
}

impl CipherCtxRef {
    /// Initializes the context for encryption.
    ///
    /// Normally this is called once to set all of the cipher, key, and IV. However, this process can be split up
    /// by first setting the cipher with no key or IV and then setting the key and IV with no cipher. This can be used
    /// to, for example, use a nonstandard IV size.
    ///
    /// # Panics
    ///
    /// Panics if the key buffer is smaller than the key size of the cipher, the IV buffer is smaller than the IV size
    /// of the cipher, or if a key or IV is provided before a cipher.
    #[corresponds(EVP_EncryptInit_ex)]
    pub fn encrypt_init(
        &mut self,
        type_: Option<&CipherRef>,
        key: Option<&[u8]>,
        iv: Option<&[u8]>,
    ) -> Result<(), ErrorStack> {
        self.cipher_init(type_, key, iv, ffi::EVP_EncryptInit_ex)
    }

    /// Initializes the context for decryption.
    ///
    /// Normally this is called once to set all of the cipher, key, and IV. However, this process can be split up
    /// by first setting the cipher with no key or IV and then setting the key and IV with no cipher. This can be used
    /// to, for example, use a nonstandard IV size.
    ///
    /// # Panics
    ///
    /// Panics if the key buffer is smaller than the key size of the cipher, the IV buffer is smaller than the IV size
    /// of the cipher, or if a key or IV is provided before a cipher.
    #[corresponds(EVP_DecryptInit_ex)]
    pub fn decrypt_init(
        &mut self,
        type_: Option<&CipherRef>,
        key: Option<&[u8]>,
        iv: Option<&[u8]>,
    ) -> Result<(), ErrorStack> {
        self.cipher_init(type_, key, iv, ffi::EVP_DecryptInit_ex)
    }

    fn cipher_init(
        &mut self,
        type_: Option<&CipherRef>,
        key: Option<&[u8]>,
        iv: Option<&[u8]>,
        f: unsafe extern "C" fn(
            *mut ffi::EVP_CIPHER_CTX,
            *const ffi::EVP_CIPHER,
            *mut ffi::ENGINE,
            *const c_uchar,
            *const c_uchar,
        ) -> c_int,
    ) -> Result<(), ErrorStack> {
        if let Some(key) = key {
            let key_len = type_.map_or_else(|| self.key_length(), |c| c.key_length());
            assert!(key_len <= key.len());
        }

        if let Some(iv) = iv {
            let iv_len = type_.map_or_else(|| self.iv_length(), |c| c.iv_length());
            assert!(iv_len <= iv.len());
        }

        unsafe {
            cvt(f(
                self.as_ptr(),
                type_.map_or(ptr::null(), |p| p.as_ptr()),
                ptr::null_mut(),
                key.map_or(ptr::null(), |k| k.as_ptr()),
                iv.map_or(ptr::null(), |iv| iv.as_ptr()),
            ))?;
        }

        Ok(())
    }

    /// Initializes the context to perform envelope encryption.
    ///
    /// Normally this is called once to set both the cipher and public keys. However, this process may be split up by
    /// first providing the cipher with no public keys and then setting the public keys with no cipher.
    ///
    /// `encrypted_keys` will contain the generated symmetric key encrypted with each corresponding asymmetric private
    /// key. The generated IV will be written to `iv`.
    ///
    /// # Panics
    ///
    /// Panics if `pub_keys` is not the same size as `encrypted_keys`, the IV buffer is smaller than the cipher's IV
    /// size, or if an IV is provided before the cipher.
    #[corresponds(EVP_SealInit)]
    pub fn seal_init<T>(
        &mut self,
        type_: Option<&CipherRef>,
        pub_keys: &[PKey<T>],
        encrypted_keys: &mut [Vec<u8>],
        iv: Option<&mut [u8]>,
    ) -> Result<(), ErrorStack>
    where
        T: HasPublic,
    {
        assert_eq!(pub_keys.len(), encrypted_keys.len());
        if !pub_keys.is_empty() {
            let iv_len = type_.map_or_else(|| self.iv_length(), |c| c.iv_length());
            assert!(iv.as_ref().map_or(0, |b| b.len()) >= iv_len);
        }

        for (pub_key, buf) in pub_keys.iter().zip(&mut *encrypted_keys) {
            buf.resize(pub_key.size(), 0);
        }

        let mut keys = encrypted_keys
            .iter_mut()
            .map(|b| b.as_mut_ptr())
            .collect::<Vec<_>>();
        let mut key_lengths = vec![0; pub_keys.len()];
        let pub_keys_len = i32::try_from(pub_keys.len()).unwrap();

        unsafe {
            cvt(ffi::EVP_SealInit(
                self.as_ptr(),
                type_.map_or(ptr::null(), |p| p.as_ptr()),
                keys.as_mut_ptr(),
                key_lengths.as_mut_ptr(),
                iv.map_or(ptr::null_mut(), |b| b.as_mut_ptr()),
                pub_keys.as_ptr() as *mut _,
                pub_keys_len,
            ))?;
        }

        for (buf, len) in encrypted_keys.iter_mut().zip(key_lengths) {
            buf.truncate(len as usize);
        }

        Ok(())
    }

    /// Initializes the context to perform envelope decryption.
    ///
    /// Normally this is called once with all of the arguments present. However, this process may be split up by first
    /// providing the cipher alone and then after providing the rest of the arguments in a second call.
    ///
    /// # Panics
    ///
    /// Panics if the IV buffer is smaller than the cipher's required IV size or if the IV is provided before the
    /// cipher.
    #[corresponds(EVP_OpenInit)]
    pub fn open_init<T>(
        &mut self,
        type_: Option<&CipherRef>,
        encrypted_key: &[u8],
        iv: Option<&[u8]>,
        priv_key: Option<&PKeyRef<T>>,
    ) -> Result<(), ErrorStack>
    where
        T: HasPrivate,
    {
        if priv_key.is_some() {
            let iv_len = type_.map_or_else(|| self.iv_length(), |c| c.iv_length());
            assert!(iv.map_or(0, |b| b.len()) >= iv_len);
        }

        let len = c_int::try_from(encrypted_key.len()).unwrap();
        unsafe {
            cvt(ffi::EVP_OpenInit(
                self.as_ptr(),
                type_.map_or(ptr::null(), |p| p.as_ptr()),
                encrypted_key.as_ptr(),
                len,
                iv.map_or(ptr::null(), |b| b.as_ptr()),
                priv_key.map_or(ptr::null_mut(), ForeignTypeRef::as_ptr),
            ))?;
        }

        Ok(())
    }

    fn assert_cipher(&self) {
        unsafe {
            assert!(!EVP_CIPHER_CTX_get0_cipher(self.as_ptr()).is_null());
        }
    }

    /// Returns the block size of the context's cipher.
    ///
    /// Stream ciphers will report a block size of 1.
    ///
    /// # Panics
    ///
    /// Panics if the context has not been initialized with a cipher.
    #[corresponds(EVP_CIPHER_CTX_block_size)]
    pub fn block_size(&self) -> usize {
        self.assert_cipher();

        unsafe { ffi::EVP_CIPHER_CTX_block_size(self.as_ptr()) as usize }
    }

    /// Returns the key length of the context's cipher.
    ///
    /// # Panics
    ///
    /// Panics if the context has not been initialized with a cipher.
    #[corresponds(EVP_CIPHER_CTX_key_length)]
    pub fn key_length(&self) -> usize {
        self.assert_cipher();

        unsafe { ffi::EVP_CIPHER_CTX_key_length(self.as_ptr()) as usize }
    }

    /// Generates a random key based on the configured cipher.
    ///
    /// # Panics
    ///
    /// Panics if the context has not been initialized with a cipher or if the buffer is smaller than the cipher's key
    /// length.
    ///
    /// This corresponds to [`EVP_CIPHER_CTX_rand_key`].
    ///
    /// [`EVP_CIPHER_CTX_rand_key`]: https://www.openssl.org/docs/manmaster/man3/EVP_CIPHER_CTX_rand_key.html
    #[corresponds(EVP_CIPHER_CTX_rand_key)]
    pub fn rand_key(&self, buf: &mut [u8]) -> Result<(), ErrorStack> {
        assert!(buf.len() >= self.key_length());

        unsafe {
            cvt(ffi::EVP_CIPHER_CTX_rand_key(
                self.as_ptr(),
                buf.as_mut_ptr(),
            ))?;
        }

        Ok(())
    }

    /// Sets the length of the key expected by the context.
    ///
    /// Only some ciphers support configurable key lengths.
    ///
    /// # Panics
    ///
    /// Panics if the context has not been initialized with a cipher.
    #[corresponds(EVP_CIPHER_CTX_set_key_length)]
    pub fn set_key_length(&mut self, len: usize) -> Result<(), ErrorStack> {
        self.assert_cipher();

        let len = c_int::try_from(len).unwrap();

        unsafe {
            cvt(ffi::EVP_CIPHER_CTX_set_key_length(self.as_ptr(), len))?;
        }

        Ok(())
    }

    /// Returns the length of the IV expected by this context.
    ///
    /// Returns 0 if the cipher does not use an IV.
    ///
    /// # Panics
    ///
    /// Panics if the context has not been initialized with a cipher.
    #[corresponds(EVP_CIPHER_CTX_iv_length)]
    pub fn iv_length(&self) -> usize {
        self.assert_cipher();

        unsafe { ffi::EVP_CIPHER_CTX_iv_length(self.as_ptr()) as usize }
    }

    /// Sets the length of the IV expected by this context.
    ///
    /// Only some ciphers support configurable IV lengths.
    ///
    /// # Panics
    ///
    /// Panics if the context has not been initialized with a cipher.
    #[corresponds(EVP_CIHPER_CTX_ctrl)]
    pub fn set_iv_length(&mut self, len: usize) -> Result<(), ErrorStack> {
        self.assert_cipher();

        let len = c_int::try_from(len).unwrap();

        unsafe {
            cvt(ffi::EVP_CIPHER_CTX_ctrl(
                self.as_ptr(),
                ffi::EVP_CTRL_GCM_SET_IVLEN,
                len,
                ptr::null_mut(),
            ))?;
        }

        Ok(())
    }

    /// Returns the length of the authentication tag expected by this context.
    ///
    /// Returns 0 if the cipher is not authenticated.
    ///
    /// # Panics
    ///
    /// Panics if the context has not been initialized with a cipher.
    ///
    /// Requires OpenSSL 3.0.0 or newer.
    #[corresponds(EVP_CIPHER_CTX_get_tag_length)]
    #[cfg(ossl300)]
    pub fn tag_length(&self) -> usize {
        self.assert_cipher();

        unsafe { ffi::EVP_CIPHER_CTX_get_tag_length(self.as_ptr()) as usize }
    }

    /// Retrieves the calculated authentication tag from the context.
    ///
    /// This should be called after `[Self::cipher_final]`, and is only supported by authenticated ciphers.
    ///
    /// The size of the buffer indicates the size of the tag. While some ciphers support a range of tag sizes, it is
    /// recommended to pick the maximum size.
    #[corresponds(EVP_CIPHER_CTX_ctrl)]
    pub fn tag(&self, tag: &mut [u8]) -> Result<(), ErrorStack> {
        let len = c_int::try_from(tag.len()).unwrap();

        unsafe {
            cvt(ffi::EVP_CIPHER_CTX_ctrl(
                self.as_ptr(),
                ffi::EVP_CTRL_GCM_GET_TAG,
                len,
                tag.as_mut_ptr() as *mut _,
            ))?;
        }

        Ok(())
    }

    /// Sets the length of the generated authentication tag.
    ///
    /// This must be called when encrypting with a cipher in CCM mode to use a tag size other than the default.
    #[corresponds(EVP_CIPHER_CTX_ctrl)]
    pub fn set_tag_length(&mut self, len: usize) -> Result<(), ErrorStack> {
        let len = c_int::try_from(len).unwrap();

        unsafe {
            cvt(ffi::EVP_CIPHER_CTX_ctrl(
                self.as_ptr(),
                ffi::EVP_CTRL_GCM_SET_TAG,
                len,
                ptr::null_mut(),
            ))?;
        }

        Ok(())
    }

    /// Sets the authentication tag for verification during decryption.
    #[corresponds(EVP_CIPHER_CTX_ctrl)]
    pub fn set_tag(&mut self, tag: &[u8]) -> Result<(), ErrorStack> {
        let len = c_int::try_from(tag.len()).unwrap();

        unsafe {
            cvt(ffi::EVP_CIPHER_CTX_ctrl(
                self.as_ptr(),
                ffi::EVP_CTRL_GCM_SET_TAG,
                len,
                tag.as_ptr() as *mut _,
            ))?;
        }

        Ok(())
    }

    /// Enables or disables padding.
    ///
    /// If padding is disabled, the plaintext must be an exact multiple of the cipher's block size.
    #[corresponds(EVP_CIPHER_CTX_set_padding)]
    pub fn set_padding(&mut self, padding: bool) {
        unsafe {
            ffi::EVP_CIPHER_CTX_set_padding(self.as_ptr(), padding as c_int);
        }
    }

    /// Sets the total length of plaintext data.
    ///
    /// This is required for ciphers operating in CCM mode.
    #[corresponds(EVP_CipherUpdate)]
    pub fn set_data_len(&mut self, len: usize) -> Result<(), ErrorStack> {
        let len = c_int::try_from(len).unwrap();

        unsafe {
            cvt(ffi::EVP_CipherUpdate(
                self.as_ptr(),
                ptr::null_mut(),
                &mut 0,
                ptr::null(),
                len,
            ))?;
        }

        Ok(())
    }

    /// Writes data into the context.
    ///
    /// Providing no output buffer will cause the input to be considered additional authenticated data (AAD).
    ///
    /// Returns the number of bytes written to `output`.
    ///
    /// # Panics
    ///
    /// Panics if `output.len()` is less than `input.len()` plus the cipher's block size.
    #[corresponds(EVP_CipherUpdate)]
    pub fn cipher_update(
        &mut self,
        input: &[u8],
        output: Option<&mut [u8]>,
    ) -> Result<usize, ErrorStack> {
        let inlen = c_int::try_from(input.len()).unwrap();

        if let Some(output) = &output {
            let mut block_size = self.block_size();
            if block_size == 1 {
                block_size = 0;
            }
            assert!(output.len() >= input.len() + block_size);
        }

        let mut outlen = 0;
        unsafe {
            cvt(ffi::EVP_CipherUpdate(
                self.as_ptr(),
                output.map_or(ptr::null_mut(), |b| b.as_mut_ptr()),
                &mut outlen,
                input.as_ptr(),
                inlen,
            ))?;
        }

        Ok(outlen as usize)
    }

    /// Like [`Self::cipher_update`] except that it appends output to a [`Vec`].
    pub fn cipher_update_vec(
        &mut self,
        input: &[u8],
        output: &mut Vec<u8>,
    ) -> Result<usize, ErrorStack> {
        let base = output.len();
        output.resize(base + input.len() + self.block_size(), 0);
        let len = self.cipher_update(input, Some(&mut output[base..]))?;
        output.truncate(base + len);

        Ok(len)
    }

    /// Finalizes the encryption or decryption process.
    ///
    /// Any remaining data will be written to the output buffer.
    ///
    /// Returns the number of bytes written to `output`.
    ///
    /// # Panics
    ///
    /// Panics if `output` is smaller than the cipher's block size.
    #[corresponds(EVP_CipherFinal)]
    pub fn cipher_final(&mut self, output: &mut [u8]) -> Result<usize, ErrorStack> {
        let block_size = self.block_size();
        if block_size > 1 {
            assert!(output.len() >= block_size);
        }

        let mut outl = 0;
        unsafe {
            cvt(ffi::EVP_CipherFinal(
                self.as_ptr(),
                output.as_mut_ptr(),
                &mut outl,
            ))?;
        }

        Ok(outl as usize)
    }

    /// Like [`Self::cipher_final`] except that it appends output to a [`Vec`].
    pub fn cipher_final_vec(&mut self, output: &mut Vec<u8>) -> Result<usize, ErrorStack> {
        let base = output.len();
        output.resize(base + self.block_size(), 0);
        let len = self.cipher_final(&mut output[base..])?;
        output.truncate(base + len);

        Ok(len)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cipher::Cipher;
    use std::slice;

    #[test]
    fn seal_open() {
        let private_pem = include_bytes!("../test/rsa.pem");
        let public_pem = include_bytes!("../test/rsa.pem.pub");
        let private_key = PKey::private_key_from_pem(private_pem).unwrap();
        let public_key = PKey::public_key_from_pem(public_pem).unwrap();
        let cipher = Cipher::aes_256_cbc();
        let secret = b"My secret message";

        let mut ctx = CipherCtx::new().unwrap();
        let mut encrypted_key = vec![];
        let mut iv = vec![0; cipher.iv_length()];
        let mut encrypted = vec![];
        ctx.seal_init(
            Some(cipher),
            &[public_key],
            slice::from_mut(&mut encrypted_key),
            Some(&mut iv),
        )
        .unwrap();
        ctx.cipher_update_vec(secret, &mut encrypted).unwrap();
        ctx.cipher_final_vec(&mut encrypted).unwrap();

        let mut decrypted = vec![];
        ctx.open_init(Some(cipher), &encrypted_key, Some(&iv), Some(&private_key))
            .unwrap();
        ctx.cipher_update_vec(&encrypted, &mut decrypted).unwrap();
        ctx.cipher_final_vec(&mut decrypted).unwrap();

        assert_eq!(secret, &decrypted[..]);
    }

    fn aes_128_cbc(cipher: &CipherRef) {
        // from https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38a.pdf
        let key = hex::decode("2b7e151628aed2a6abf7158809cf4f3c").unwrap();
        let iv = hex::decode("000102030405060708090a0b0c0d0e0f").unwrap();
        let pt = hex::decode("6bc1bee22e409f96e93d7e117393172aae2d8a571e03ac9c9eb76fac45af8e51")
            .unwrap();
        let ct = hex::decode("7649abac8119b246cee98e9b12e9197d5086cb9b507219ee95db113a917678b2")
            .unwrap();

        let mut ctx = CipherCtx::new().unwrap();

        ctx.encrypt_init(Some(cipher), Some(&key), Some(&iv))
            .unwrap();
        ctx.set_padding(false);

        let mut buf = vec![];
        ctx.cipher_update_vec(&pt, &mut buf).unwrap();
        ctx.cipher_final_vec(&mut buf).unwrap();

        assert_eq!(buf, ct);

        ctx.decrypt_init(Some(cipher), Some(&key), Some(&iv))
            .unwrap();
        ctx.set_padding(false);

        let mut buf = vec![];
        ctx.cipher_update_vec(&ct, &mut buf).unwrap();
        ctx.cipher_final_vec(&mut buf).unwrap();

        assert_eq!(buf, pt);
    }

    #[test]
    #[cfg(ossl300)]
    fn fetched_aes_128_cbc() {
        let cipher = Cipher::fetch(None, "AES-128-CBC", None).unwrap();
        aes_128_cbc(&cipher);
    }

    #[test]
    fn default_aes_128_cbc() {
        let cipher = Cipher::aes_128_cbc();
        aes_128_cbc(cipher);
    }
}
