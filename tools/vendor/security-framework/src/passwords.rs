//! Support for password entries in the keychain.  Works on both iOS and macOS.
//!
//! If you want the extended keychain facilities only available on macOS, use the
//! version of these functions in the macOS extensions module.

use crate::base::Result;
use crate::{cvt, Error};
use core_foundation::base::{CFType, TCFType};
use core_foundation::boolean::CFBoolean;
use core_foundation::data::CFData;
use core_foundation::dictionary::CFDictionary;
use core_foundation::number::CFNumber;
use core_foundation::string::CFString;
use core_foundation_sys::base::{CFGetTypeID, CFRelease, CFTypeRef};
use core_foundation_sys::data::CFDataRef;
use security_framework_sys::base::{errSecDuplicateItem, errSecParam};
use security_framework_sys::item::{
    kSecAttrAccount, kSecAttrAuthenticationType, kSecAttrPath, kSecAttrPort, kSecAttrProtocol,
    kSecAttrSecurityDomain, kSecAttrServer, kSecAttrService, kSecClass, kSecClassGenericPassword,
    kSecClassInternetPassword, kSecReturnData, kSecValueData,
};
use security_framework_sys::keychain::{SecAuthenticationType, SecProtocolType};
use security_framework_sys::keychain_item::{
    SecItemAdd, SecItemCopyMatching, SecItemDelete, SecItemUpdate,
};

/// Set a generic password for the given service and account.
/// Creates or updates a keychain entry.
pub fn set_generic_password(service: &str, account: &str, password: &[u8]) -> Result<()> {
    let mut query = generic_password_query(service, account);
    set_password_internal(&mut query, password)
}

/// Get the generic password for the given service and account.  If no matching
/// keychain entry exists, fails with error code `errSecItemNotFound`.
pub fn get_generic_password(service: &str, account: &str) -> Result<Vec<u8>> {
    let mut query = generic_password_query(service, account);
    query.push((
        unsafe { CFString::wrap_under_get_rule(kSecReturnData) },
        CFBoolean::from(true).as_CFType(),
    ));
    let params = CFDictionary::from_CFType_pairs(&query);
    let mut ret: CFTypeRef = std::ptr::null();
    cvt(unsafe { SecItemCopyMatching(params.as_concrete_TypeRef(), &mut ret) })?;
    get_password_and_release(ret)
}

/// Delete the generic password keychain entry for the given service and account.
/// If none exists, fails with error code `errSecItemNotFound`.
pub fn delete_generic_password(service: &str, account: &str) -> Result<()> {
    let query = generic_password_query(service, account);
    let params = CFDictionary::from_CFType_pairs(&query);
    cvt(unsafe { SecItemDelete(params.as_concrete_TypeRef()) })
}

/// Set an internet password for the given endpoint parameters.
/// Creates or updates a keychain entry.
#[allow(clippy::too_many_arguments)]
pub fn set_internet_password(
    server: &str,
    security_domain: Option<&str>,
    account: &str,
    path: &str,
    port: Option<u16>,
    protocol: SecProtocolType,
    authentication_type: SecAuthenticationType,
    password: &[u8],
) -> Result<()> {
    let mut query = internet_password_query(
        server,
        security_domain,
        account,
        path,
        port,
        protocol,
        authentication_type,
    );
    set_password_internal(&mut query, password)
}

/// Get the internet password for the given endpoint parameters.  If no matching
/// keychain entry exists, fails with error code `errSecItemNotFound`.
pub fn get_internet_password(
    server: &str,
    security_domain: Option<&str>,
    account: &str,
    path: &str,
    port: Option<u16>,
    protocol: SecProtocolType,
    authentication_type: SecAuthenticationType,
) -> Result<Vec<u8>> {
    let mut query = internet_password_query(
        server,
        security_domain,
        account,
        path,
        port,
        protocol,
        authentication_type,
    );
    query.push((
        unsafe { CFString::wrap_under_get_rule(kSecReturnData) },
        CFBoolean::from(true).as_CFType(),
    ));
    let params = CFDictionary::from_CFType_pairs(&query);
    let mut ret: CFTypeRef = std::ptr::null();
    cvt(unsafe { SecItemCopyMatching(params.as_concrete_TypeRef(), &mut ret) })?;
    get_password_and_release(ret)
}

/// Delete the internet password for the given endpoint parameters.
/// If none exists, fails with error code `errSecItemNotFound`.
pub fn delete_internet_password(
    server: &str,
    security_domain: Option<&str>,
    account: &str,
    path: &str,
    port: Option<u16>,
    protocol: SecProtocolType,
    authentication_type: SecAuthenticationType,
) -> Result<()> {
    let query = internet_password_query(
        server,
        security_domain,
        account,
        path,
        port,
        protocol,
        authentication_type,
    );
    let params = CFDictionary::from_CFType_pairs(&query);
    cvt(unsafe { SecItemDelete(params.as_concrete_TypeRef()) })
}

// Generic passwords are identified by service and account.  They have other
// attributes, but this interface doesn't allow specifying them.
fn generic_password_query(service: &str, account: &str) -> Vec<(CFString, CFType)> {
    let query = vec![
        (
            unsafe { CFString::wrap_under_get_rule(kSecClass) },
            unsafe { CFString::wrap_under_get_rule(kSecClassGenericPassword).as_CFType() },
        ),
        (
            unsafe { CFString::wrap_under_get_rule(kSecAttrService) },
            CFString::from(service).as_CFType(),
        ),
        (
            unsafe { CFString::wrap_under_get_rule(kSecAttrAccount) },
            CFString::from(account).as_CFType(),
        ),
    ];
    query
}

// Internet passwords are identified by a number of attributes.
// They can have others, but this interface doesn't allow specifying them.
fn internet_password_query(
    server: &str,
    security_domain: Option<&str>,
    account: &str,
    path: &str,
    port: Option<u16>,
    protocol: SecProtocolType,
    authentication_type: SecAuthenticationType,
) -> Vec<(CFString, CFType)> {
    let mut query = vec![
        (
            unsafe { CFString::wrap_under_get_rule(kSecClass) },
            unsafe { CFString::wrap_under_get_rule(kSecClassInternetPassword) }.as_CFType(),
        ),
        (
            unsafe { CFString::wrap_under_get_rule(kSecAttrServer) },
            CFString::from(server).as_CFType(),
        ),
        (
            unsafe { CFString::wrap_under_get_rule(kSecAttrPath) },
            CFString::from(path).as_CFType(),
        ),
        (
            unsafe { CFString::wrap_under_get_rule(kSecAttrAccount) },
            CFString::from(account).as_CFType(),
        ),
        (
            unsafe { CFString::wrap_under_get_rule(kSecAttrProtocol) },
            CFNumber::from(protocol as i32).as_CFType(),
        ),
        (
            unsafe { CFString::wrap_under_get_rule(kSecAttrAuthenticationType) },
            CFNumber::from(authentication_type as i32).as_CFType(),
        ),
    ];
    if let Some(domain) = security_domain {
        query.push((
            unsafe { CFString::wrap_under_get_rule(kSecAttrSecurityDomain) },
            CFString::from(domain).as_CFType(),
        ))
    }
    if let Some(port) = port {
        query.push((
            unsafe { CFString::wrap_under_get_rule(kSecAttrPort) },
            CFNumber::from(port as i32).as_CFType(),
        ))
    }
    query
}

// This starts by trying to create the password with the given query params.
// If the creation attempt reveals that one exists, its password is updated.
fn set_password_internal(query: &mut Vec<(CFString, CFType)>, password: &[u8]) -> Result<()> {
    let query_len = query.len();
    query.push((
        unsafe { CFString::wrap_under_get_rule(kSecValueData) },
        CFData::from_buffer(password).as_CFType(),
    ));
    let params = CFDictionary::from_CFType_pairs(query);
    let mut ret = std::ptr::null();
    let status = unsafe { SecItemAdd(params.as_concrete_TypeRef(), &mut ret) };
    if status == errSecDuplicateItem {
        let params = CFDictionary::from_CFType_pairs(&query[0..query_len]);
        let update = CFDictionary::from_CFType_pairs(&query[query_len..]);
        cvt(unsafe { SecItemUpdate(params.as_concrete_TypeRef(), update.as_concrete_TypeRef()) })
    } else {
        cvt(status)
    }
}

// Having retrieved a password entry, this copies and returns the password.
//
// # Safety
// The data element passed in is assumed to have been returned from a Copy
// call, so it's released after we are done with it.
fn get_password_and_release(data: CFTypeRef) -> Result<Vec<u8>> {
    if !data.is_null() {
        let type_id = unsafe { CFGetTypeID(data) };
        if type_id == CFData::type_id() {
            let val = unsafe { CFData::wrap_under_create_rule(data as CFDataRef) };
            let mut vec = Vec::new();
            vec.extend_from_slice(val.bytes());
            return Ok(vec);
        } else {
            // unexpected: we got a reference to some other type.
            // Release it to make sure there's no leak, but
            // we can't return the password in this case.
            unsafe { CFRelease(data) };
        }
    }
    Err(Error::from_code(errSecParam))
}

#[cfg(test)]
mod test {
    use security_framework_sys::base::errSecItemNotFound;
    use super::*;

    #[test]
    fn missing_generic() {
        let name = "a string not likely to already be in the keychain as service or account";
        let result = delete_generic_password(name, name);
        match result {
            Ok(()) => (),   // this is ok because the name _might_ be in the keychain
            Err(err) if err.code() == errSecItemNotFound => (),
            Err(err) => panic!("missing_generic: delete failed with status: {}", err.code()),
        };
        let result = get_generic_password(name, name);
        match result {
            Ok(bytes) => panic!("missing_generic: get returned {:?}", bytes),
            Err(err) if err.code() == errSecItemNotFound => (),
            Err(err) => panic!("missing_generic: get failed with status: {}", err.code()),
        };
        let result = delete_generic_password(name, name);
        match result {
            Ok(()) => panic!("missing_generic: second delete found a password"),
            Err(err) if err.code() == errSecItemNotFound => (),
            Err(err) => panic!("missing_generic: delete failed with status: {}", err.code()),
        };
    }

    #[test]
    fn roundtrip_generic() {
        let name = "roundtrip_generic";
        set_generic_password(name, name, name.as_bytes()).expect("set_generic_password");
        let pass = get_generic_password(name, name).expect("get_generic_password");
        assert_eq!(name.as_bytes(), pass);
        delete_generic_password(name, name).expect("delete_generic_password")
    }

    #[test]
    fn update_generic() {
        let name = "update_generic";
        set_generic_password(name, name, name.as_bytes()).expect("set_generic_password");
        let alternate = "update_generic_alternate";
        set_generic_password(name, name, alternate.as_bytes()).expect("set_generic_password");
        let pass = get_generic_password(name, name).expect("get_generic_password");
        assert_eq!(pass, alternate.as_bytes());
        delete_generic_password(name, name).expect("delete_generic_password")
    }

    #[test]
    fn missing_internet() {
        let name = "a string not likely to already be in the keychain as service or account";
        let (server, domain, account, path, port, protocol, auth) = (
            name,
            None,
            name,
            "/",
            Some(8080u16),
            SecProtocolType::HTTP,
            SecAuthenticationType::Any,
        );
        let result = delete_internet_password(
            server,
            domain,
            account,
            path,
            port,
            protocol,
            auth,
        );
        match result {
            Ok(()) => (),   // this is ok because the name _might_ be in the keychain
            Err(err) if err.code() == errSecItemNotFound => (),
            Err(err) => panic!("missing_internet: delete failed with status: {}", err.code()),
        };
        let result = get_internet_password(
            server,
            domain,
            account,
            path,
            port,
            protocol,
            auth,
        );
        match result {
            Ok(bytes) => panic!("missing_internet: get returned {:?}", bytes),
            Err(err) if err.code() == errSecItemNotFound => (),
            Err(err) => panic!("missing_internet: get failed with status: {}", err.code()),
        };
        let result = delete_internet_password(
            server,
            domain,
            account,
            path,
            port,
            protocol,
            auth,
        );
        match result {
            Ok(()) => panic!("missing_internet: second delete found a password"),
            Err(err) if err.code() == errSecItemNotFound => (),
            Err(err) => panic!("missing_internet: delete failed with status: {}", err.code()),
        };
    }

    #[test]
    fn roundtrip_internet() {
        let name = "roundtrip_internet";
        let (server, domain, account, path, port, protocol, auth) = (
            name,
            None,
            name,
            "/",
            Some(8080u16),
            SecProtocolType::HTTP,
            SecAuthenticationType::Any,
        );
        set_internet_password(
            server,
            domain,
            account,
            path,
            port,
            protocol,
            auth,
            name.as_bytes(),
        )
        .expect("set_internet_password");
        let pass = get_internet_password(server, domain, account, path, port, protocol, auth)
            .expect("get_internet_password");
        assert_eq!(name.as_bytes(), pass);
        delete_internet_password(server, domain, account, path, port, protocol, auth)
            .expect("delete_internet_password");
    }

    #[test]
    fn update_internet() {
        let name = "update_internet";
        let (server, domain, account, path, port, protocol, auth) = (
            name,
            None,
            name,
            "/",
            Some(8080u16),
            SecProtocolType::HTTP,
            SecAuthenticationType::Any,
        );
        set_internet_password(
            server,
            domain,
            account,
            path,
            port,
            protocol,
            auth,
            name.as_bytes(),
        )
        .expect("set_internet_password");
        let alternate = "alternate_internet_password";
        set_internet_password(
            server,
            domain,
            account,
            path,
            port,
            protocol,
            auth,
            alternate.as_bytes(),
        )
        .expect("set_internet_password");
        let pass = get_internet_password(server, domain, account, path, port, protocol, auth)
            .expect("get_internet_password");
        assert_eq!(pass, alternate.as_bytes());
        delete_internet_password(server, domain, account, path, port, protocol, auth)
            .expect("delete_internet_password");
    }
}
