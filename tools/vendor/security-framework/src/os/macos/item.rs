//! OSX specific functionality for items.

use core_foundation::base::TCFType;
use core_foundation::string::CFString;
use core_foundation_sys::string::CFStringRef;
use security_framework_sys::item::*;

use crate::item::ItemSearchOptions;
use crate::os::macos::keychain::SecKeychain;
use crate::ItemSearchOptionsInternals;

/// Types of `SecKey`s.
#[derive(Debug, Copy, Clone)]
pub struct KeyType(CFStringRef);

#[allow(missing_docs)]
impl KeyType {
    #[inline(always)]
    pub fn rsa() -> Self {
        unsafe { Self(kSecAttrKeyTypeRSA) }
    }

    #[inline(always)]
    pub fn dsa() -> Self {
        unsafe { Self(kSecAttrKeyTypeDSA) }
    }

    #[inline(always)]
    pub fn aes() -> Self {
        unsafe { Self(kSecAttrKeyTypeAES) }
    }

    #[inline(always)]
    pub fn des() -> Self {
        unsafe { Self(kSecAttrKeyTypeDES) }
    }

    #[inline(always)]
    pub fn triple_des() -> Self {
        unsafe { Self(kSecAttrKeyType3DES) }
    }

    #[inline(always)]
    pub fn rc4() -> Self {
        unsafe { Self(kSecAttrKeyTypeRC4) }
    }

    #[inline(always)]
    pub fn cast() -> Self {
        unsafe { Self(kSecAttrKeyTypeCAST) }
    }

    #[cfg(feature = "OSX_10_9")]
    #[inline(always)]
    pub fn ec() -> Self {
        unsafe { Self(kSecAttrKeyTypeEC) }
    }

    pub(crate) fn to_str(self) -> CFString {
        unsafe { CFString::wrap_under_get_rule(self.0) }
    }
}

/// An extension trait adding OSX specific functionality to `ItemSearchOptions`.
pub trait ItemSearchOptionsExt {
    /// Search within the specified keychains.
    ///
    /// If this is not called, the default keychain will be searched.
    fn keychains(&mut self, keychains: &[SecKeychain]) -> &mut Self;
}

impl ItemSearchOptionsExt for ItemSearchOptions {
    #[inline(always)]
    fn keychains(&mut self, keychains: &[SecKeychain]) -> &mut Self {
        ItemSearchOptionsInternals::keychains(self, keychains)
    }
}

#[cfg(test)]
mod test {
    use crate::item::*;
    use crate::os::macos::certificate::SecCertificateExt;
    use crate::os::macos::item::ItemSearchOptionsExt;
    use crate::os::macos::test::keychain;
    use tempdir::TempDir;

    #[test]
    fn find_certificate() {
        let dir = p!(TempDir::new("find_certificate"));
        let keychain = keychain(dir.path());
        let results = p!(ItemSearchOptions::new()
            .keychains(&[keychain])
            .class(ItemClass::certificate())
            .search());
        assert_eq!(1, results.len());
        let certificate = match results[0] {
            SearchResult::Ref(Reference::Certificate(ref cert)) => cert,
            _ => panic!("expected certificate"),
        };
        assert_eq!("foobar.com", p!(certificate.common_name()));
    }
}
