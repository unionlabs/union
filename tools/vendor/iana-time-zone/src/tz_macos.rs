pub(crate) fn get_timezone_inner() -> Result<String, crate::GetTimezoneError> {
    get_timezone().ok_or(crate::GetTimezoneError::OsError)
}

#[inline]
fn get_timezone() -> Option<String> {
    // The longest name in the IANA time zone database is 25 ASCII characters long.
    const MAX_LEN: usize = 32;
    let mut buf = [0; MAX_LEN];

    // Get system time zone, and borrow its name.
    let tz = system_time_zone::SystemTimeZone::new()?;
    let name = tz.name()?;

    // If the name is encoded in UTF-8, copy it directly.
    let name = if let Some(name) = name.as_utf8() {
        name
    } else {
        // Otherwise convert the name to UTF-8.
        name.to_utf8(&mut buf)?
    };

    if name.is_empty() || name.len() >= MAX_LEN {
        // The name should not be empty, or excessively long.
        None
    } else {
        Some(name.to_owned())
    }
}

mod system_time_zone {
    //! create a safe wrapper around `CFTimeZoneRef`

    use core_foundation_sys::base::{CFRelease, CFTypeRef};
    use core_foundation_sys::timezone::{CFTimeZoneCopySystem, CFTimeZoneGetName, CFTimeZoneRef};

    pub(crate) struct SystemTimeZone(CFTimeZoneRef);

    impl Drop for SystemTimeZone {
        fn drop(&mut self) {
            // SAFETY: `SystemTimeZone` is only ever created with a valid `CFTimeZoneRef`.
            unsafe { CFRelease(self.0 as CFTypeRef) };
        }
    }

    impl SystemTimeZone {
        pub(crate) fn new() -> Option<Self> {
            // SAFETY: No invariants to uphold. We'll release the pointer when we don't need it anymore.
            let v: CFTimeZoneRef = unsafe { CFTimeZoneCopySystem() };
            if v.is_null() {
                None
            } else {
                Some(SystemTimeZone(v))
            }
        }

        /// Get the time zone name as a [super::string_ref::StringRef].
        ///
        /// The lifetime of the `StringRef` is bound to our lifetime. Mutable
        /// access is also prevented by taking a reference to `self`.
        pub(crate) fn name(&self) -> Option<super::string_ref::StringRef<'_, Self>> {
            // SAFETY: `SystemTimeZone` is only ever created with a valid `CFTimeZoneRef`.
            let string = unsafe { CFTimeZoneGetName(self.0) };
            if string.is_null() {
                None
            } else {
                // SAFETY: here we ensure that `string` is a valid pointer.
                Some(unsafe { super::string_ref::StringRef::new(string, self) })
            }
        }
    }
}

mod string_ref {
    //! create safe wrapper around `CFStringRef`

    use std::convert::TryInto;

    use core_foundation_sys::base::{Boolean, CFRange};
    use core_foundation_sys::string::{
        kCFStringEncodingUTF8, CFStringGetBytes, CFStringGetCStringPtr, CFStringGetLength,
        CFStringRef,
    };

    pub(crate) struct StringRef<'a, T> {
        string: CFStringRef,
        // We exclude mutable access to the parent by taking a reference to the
        // parent (rather than, for example, just using a marker to enforce the
        // parent's lifetime).
        _parent: &'a T,
    }

    impl<'a, T> StringRef<'a, T> {
        // SAFETY: `StringRef` must be valid pointer
        pub(crate) unsafe fn new(string: CFStringRef, _parent: &'a T) -> Self {
            Self { string, _parent }
        }

        pub(crate) fn as_utf8(&self) -> Option<&'a str> {
            // SAFETY: `StringRef` is only ever created with a valid `CFStringRef`.
            let v = unsafe { CFStringGetCStringPtr(self.string, kCFStringEncodingUTF8) };
            if !v.is_null() {
                // SAFETY: `CFStringGetCStringPtr()` returns NUL-terminated strings.
                let v = unsafe { std::ffi::CStr::from_ptr(v) };
                if let Ok(v) = v.to_str() {
                    return Some(v);
                }
            }
            None
        }

        pub(crate) fn to_utf8<'b>(&self, buf: &'b mut [u8]) -> Option<&'b str> {
            // SAFETY: `StringRef` is only ever created with a valid `CFStringRef`.
            let length = unsafe { CFStringGetLength(self.string) };

            let mut buf_bytes = 0;
            let range = CFRange {
                location: 0,
                length,
            };

            let converted_bytes = unsafe {
                // SAFETY: `StringRef` is only ever created with a valid `CFStringRef`.
                CFStringGetBytes(
                    self.string,
                    range,
                    kCFStringEncodingUTF8,
                    b'\0',
                    false as Boolean,
                    buf.as_mut_ptr(),
                    buf.len() as isize,
                    &mut buf_bytes,
                )
            };
            if converted_bytes != length {
                return None;
            }

            let len = buf_bytes.try_into().ok()?;
            let s = buf.get(..len)?;
            std::str::from_utf8(s).ok()
        }
    }
}
