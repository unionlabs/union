//! Shared secret derivation.
use foreign_types::ForeignTypeRef;
use std::marker::PhantomData;
use std::ptr;

use crate::error::ErrorStack;
use crate::pkey::{HasPrivate, HasPublic, PKeyRef};
use crate::{cvt, cvt_p};

/// A type used to derive a shared secret between two keys.
pub struct Deriver<'a>(*mut ffi::EVP_PKEY_CTX, PhantomData<&'a ()>);

unsafe impl<'a> Sync for Deriver<'a> {}
unsafe impl<'a> Send for Deriver<'a> {}

#[allow(clippy::len_without_is_empty)]
impl<'a> Deriver<'a> {
    /// Creates a new `Deriver` using the provided private key.
    ///
    /// This corresponds to [`EVP_PKEY_derive_init`].
    ///
    /// [`EVP_PKEY_derive_init`]: https://www.openssl.org/docs/man1.0.2/crypto/EVP_PKEY_derive_init.html
    pub fn new<T>(key: &'a PKeyRef<T>) -> Result<Deriver<'a>, ErrorStack>
    where
        T: HasPrivate,
    {
        unsafe {
            cvt_p(ffi::EVP_PKEY_CTX_new(key.as_ptr(), ptr::null_mut()))
                .map(|p| Deriver(p, PhantomData))
                .and_then(|ctx| cvt(ffi::EVP_PKEY_derive_init(ctx.0)).map(|_| ctx))
        }
    }

    /// Sets the peer key used for secret derivation.
    ///
    /// This corresponds to [`EVP_PKEY_derive_set_peer`]:
    ///
    /// [`EVP_PKEY_derive_set_peer`]: https://www.openssl.org/docs/man1.0.2/crypto/EVP_PKEY_derive_init.html
    pub fn set_peer<T>(&mut self, key: &'a PKeyRef<T>) -> Result<(), ErrorStack>
    where
        T: HasPublic,
    {
        unsafe { cvt(ffi::EVP_PKEY_derive_set_peer(self.0, key.as_ptr())).map(|_| ()) }
    }

    /// Returns the size of the shared secret.
    ///
    /// It can be used to size the buffer passed to [`Deriver::derive`].
    ///
    /// This corresponds to [`EVP_PKEY_derive`].
    ///
    /// [`Deriver::derive`]: #method.derive
    /// [`EVP_PKEY_derive`]: https://www.openssl.org/docs/man1.0.2/crypto/EVP_PKEY_derive_init.html
    pub fn len(&mut self) -> Result<usize, ErrorStack> {
        unsafe {
            let mut len = 0;
            cvt(ffi::EVP_PKEY_derive(self.0, ptr::null_mut(), &mut len)).map(|_| len)
        }
    }

    /// Derives a shared secret between the two keys, writing it into the buffer.
    ///
    /// Returns the number of bytes written.
    ///
    /// This corresponds to [`EVP_PKEY_derive`].
    ///
    /// [`EVP_PKEY_derive`]: https://www.openssl.org/docs/man1.0.2/crypto/EVP_PKEY_derive_init.html
    pub fn derive(&mut self, buf: &mut [u8]) -> Result<usize, ErrorStack> {
        let mut len = buf.len();
        unsafe {
            cvt(ffi::EVP_PKEY_derive(
                self.0,
                buf.as_mut_ptr() as *mut _,
                &mut len,
            ))
            .map(|_| len)
        }
    }

    /// A convenience function which derives a shared secret and returns it in a new buffer.
    ///
    /// This simply wraps [`Deriver::len`] and [`Deriver::derive`].
    ///
    /// [`Deriver::len`]: #method.len
    /// [`Deriver::derive`]: #method.derive
    pub fn derive_to_vec(&mut self) -> Result<Vec<u8>, ErrorStack> {
        let len = self.len()?;
        let mut buf = vec![0; len];
        let len = self.derive(&mut buf)?;
        buf.truncate(len);
        Ok(buf)
    }
}

impl<'a> Drop for Deriver<'a> {
    fn drop(&mut self) {
        unsafe {
            ffi::EVP_PKEY_CTX_free(self.0);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::ec::{EcGroup, EcKey};
    use crate::nid::Nid;
    use crate::pkey::PKey;

    #[test]
    fn derive_without_peer() {
        let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap();
        let ec_key = EcKey::generate(&group).unwrap();
        let pkey = PKey::from_ec_key(ec_key).unwrap();
        let mut deriver = Deriver::new(&pkey).unwrap();
        deriver.derive_to_vec().unwrap_err();
    }

    #[test]
    fn test_ec_key_derive() {
        let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap();
        let ec_key = EcKey::generate(&group).unwrap();
        let ec_key2 = EcKey::generate(&group).unwrap();
        let pkey = PKey::from_ec_key(ec_key).unwrap();
        let pkey2 = PKey::from_ec_key(ec_key2).unwrap();
        let mut deriver = Deriver::new(&pkey).unwrap();
        deriver.set_peer(&pkey2).unwrap();
        let shared = deriver.derive_to_vec().unwrap();
        assert!(!shared.is_empty());
    }
}
