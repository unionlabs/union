//! Cross platform FFI helpers.

use std::ffi::CStr;

// The system property named 'persist.sys.timezone' contains the name of the
// current timezone.
//
// From https://android.googlesource.com/platform/bionic/+/gingerbread-release/libc/docs/OVERVIEW.TXT#79:
//
// > The name of the current timezone is taken from the TZ environment variable,
// > if defined. Otherwise, the system property named 'persist.sys.timezone' is
// > checked instead.
const ANDROID_TIMEZONE_PROPERTY_NAME: &[u8] = b"persist.sys.timezone\0";

/// Return a [`CStr`] to access the timezone from an Android system properties
/// environment.
pub(crate) fn android_timezone_property_name() -> &'static CStr {
    // In tests or debug mode, opt into extra runtime checks.
    if cfg!(any(test, debug_assertions)) {
        return CStr::from_bytes_with_nul(ANDROID_TIMEZONE_PROPERTY_NAME).unwrap();
    }

    // SAFETY: the key is NUL-terminated and there are no other NULs, this
    // invariant is checked in tests.
    unsafe { CStr::from_bytes_with_nul_unchecked(ANDROID_TIMEZONE_PROPERTY_NAME) }
}

#[cfg(test)]
mod tests {
    use std::ffi::CStr;

    use super::{android_timezone_property_name, ANDROID_TIMEZONE_PROPERTY_NAME};

    #[test]
    fn test_android_timezone_property_name_is_valid_cstr() {
        CStr::from_bytes_with_nul(ANDROID_TIMEZONE_PROPERTY_NAME).unwrap();

        let mut invalid_property_name = ANDROID_TIMEZONE_PROPERTY_NAME.to_owned();
        invalid_property_name.push(b'\0');
        CStr::from_bytes_with_nul(&invalid_property_name).unwrap_err();
    }

    #[test]
    fn test_android_timezone_property_name_getter() {
        let key = android_timezone_property_name().to_bytes_with_nul();
        assert_eq!(key, ANDROID_TIMEZONE_PROPERTY_NAME);
        std::str::from_utf8(key).unwrap();
    }
}
