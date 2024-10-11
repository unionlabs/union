use std::os::raw::c_char;

static VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "\0"); // Add trailing NULL byte for C string

/// Returns a version number of this library as a C string.
///
/// The string is owned by libwasmvm and must not be mutated or destroyed by the caller.
#[no_mangle]
pub extern "C" fn version_str() -> *const c_char {
    VERSION.as_ptr() as *const _
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    #[test]
    fn version_works() {
        // Returns the same pointer every time
        let ptr1 = version_str();
        let ptr2 = version_str();
        assert_eq!(ptr1, ptr2);

        // Contains correct data
        let version_ptr = version_str();
        let version_str = unsafe { CStr::from_ptr(version_ptr) }.to_str().unwrap();
        // assert_eq!(version_str, "1.2.3");

        let mut parts = version_str.split('-');
        let version_core = parts.next().unwrap();
        let components = version_core.split('.').collect::<Vec<_>>();
        assert_eq!(components.len(), 3);
        assert!(
            components[0].chars().all(|c| c.is_ascii_digit()),
            "Invalid major component: '{}'",
            components[0]
        );
        assert!(
            components[1].chars().all(|c| c.is_ascii_digit()),
            "Invalid minor component: '{}'",
            components[1]
        );
        assert!(
            components[2].chars().all(|c| c.is_ascii_digit()),
            "Invalid patch component: '{}'",
            components[2]
        );
        if let Some(prerelease) = parts.next() {
            assert!(prerelease
                .chars()
                .all(|c| c == '.' || c.is_ascii_alphanumeric()));
        }
        assert_eq!(parts.next(), None);
    }
}
