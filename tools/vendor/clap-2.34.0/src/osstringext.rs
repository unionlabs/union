use std::ffi::OsStr;
#[cfg(not(any(target_os = "windows", target_arch = "wasm32")))]
use std::os::unix::ffi::OsStrExt;
#[cfg(any(target_os = "windows", target_arch = "wasm32"))]
use crate::INVALID_UTF8;

#[cfg(any(target_os = "windows", target_arch = "wasm32"))]
pub trait OsStrExt3 {
    fn from_bytes(b: &[u8]) -> &Self;
    fn as_bytes(&self) -> &[u8];
}

#[doc(hidden)]
pub trait OsStrExt2 {
    fn starts_with(&self, s: &[u8]) -> bool;
    fn split_at_byte(&self, b: u8) -> (&OsStr, &OsStr);
    fn split_at(&self, i: usize) -> (&OsStr, &OsStr);
    fn trim_left_matches(&self, b: u8) -> &OsStr;
    fn contains_byte(&self, b: u8) -> bool;
    fn split(&self, b: u8) -> OsSplit;
}

// A starts-with implementation that does not panic when the OsStr contains
// invalid Unicode.
//
// A Windows OsStr is usually UTF-16. If `prefix` is valid UTF-8, we can
// re-encode it as UTF-16, and ask whether `osstr` starts with the same series
// of u16 code units. If `prefix` is not valid UTF-8, then this comparison
// isn't meaningful, and we just return false.
#[cfg(target_os = "windows")]
fn windows_osstr_starts_with(osstr: &OsStr, prefix: &[u8]) -> bool {
    use std::os::windows::ffi::OsStrExt;
    let prefix_str = if let Ok(s) = std::str::from_utf8(prefix) {
        s
    } else {
        return false;
    };
    let mut osstr_units = osstr.encode_wide();
    let mut prefix_units = prefix_str.encode_utf16();
    loop {
        match (osstr_units.next(), prefix_units.next()) {
            // These code units match. Keep looping.
            (Some(o), Some(p)) if o == p => continue,
            // We've reached the end of the prefix. It's a match.
            (_, None) => return true,
            // Otherwise, it's not a match.
            _ => return false,
        }
    }
}

#[test]
#[cfg(target_os = "windows")]
fn test_windows_osstr_starts_with() {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    fn from_ascii(ascii: &[u8]) -> OsString {
        let u16_vec: Vec<u16> = ascii.iter().map(|&c| c as u16).collect();
        OsString::from_wide(&u16_vec)
    }

    // Test all the basic cases.
    assert!(windows_osstr_starts_with(&from_ascii(b"abcdef"), b"abc"));
    assert!(windows_osstr_starts_with(&from_ascii(b"abcdef"), b"abcdef"));
    assert!(!windows_osstr_starts_with(&from_ascii(b"abcdef"), b"def"));
    assert!(!windows_osstr_starts_with(&from_ascii(b"abc"), b"abcd"));

    // Test the case where the candidate prefix is not valid UTF-8. Note that a
    // standalone \xff byte is valid ASCII but not valid UTF-8. Thus although
    // these strings look identical, they do not match.
    assert!(!windows_osstr_starts_with(&from_ascii(b"\xff"), b"\xff"));

    // Test the case where the OsString is not valid UTF-16. It should still be
    // possible to match the valid characters at the front.
    //
    // UTF-16 surrogate characters are only valid in pairs. Including one on
    // the end by itself makes this invalid UTF-16.
    let surrogate_char: u16 = 0xDC00;
    let invalid_unicode =
        OsString::from_wide(&['a' as u16, 'b' as u16, 'c' as u16, surrogate_char]);
    assert!(
        invalid_unicode.to_str().is_none(),
        "This string is invalid Unicode, and conversion to &str should fail.",
    );
    assert!(windows_osstr_starts_with(&invalid_unicode, b"abc"));
    assert!(!windows_osstr_starts_with(&invalid_unicode, b"abcd"));
}

#[cfg(any(target_os = "windows", target_arch = "wasm32"))]
impl OsStrExt3 for OsStr {
    fn from_bytes(b: &[u8]) -> &Self {
        use std::mem;
        unsafe { mem::transmute(b) }
    }
    fn as_bytes(&self) -> &[u8] {
        self.to_str().map(|s| s.as_bytes()).expect(INVALID_UTF8)
    }
}

impl OsStrExt2 for OsStr {
    fn starts_with(&self, s: &[u8]) -> bool {
        #[cfg(target_os = "windows")]
        {
            // On Windows, the as_bytes() method will panic if the OsStr
            // contains invalid Unicode. To avoid this, we use a
            // Windows-specific starts-with function that doesn't rely on
            // as_bytes(). This is necessary for Windows command line
            // applications to handle non-Unicode arguments successfully. This
            // allows common cases like `clap.exe [invalid]` to succeed, though
            // cases that require string splitting will still fail, like
            // `clap.exe --arg=[invalid]`. Note that this entire module is
            // replaced in Clap 3.x, so this workaround is specific to the 2.x
            // branch.
            windows_osstr_starts_with(self, s)
        }
        #[cfg(not(target_os = "windows"))]
        {
            self.as_bytes().starts_with(s)
        }
    }

    fn contains_byte(&self, byte: u8) -> bool {
        for b in self.as_bytes() {
            if b == &byte {
                return true;
            }
        }
        false
    }

    fn split_at_byte(&self, byte: u8) -> (&OsStr, &OsStr) {
        for (i, b) in self.as_bytes().iter().enumerate() {
            if b == &byte {
                return (
                    OsStr::from_bytes(&self.as_bytes()[..i]),
                    OsStr::from_bytes(&self.as_bytes()[i + 1..]),
                );
            }
        }
        (
            &*self,
            OsStr::from_bytes(&self.as_bytes()[self.len()..self.len()]),
        )
    }

    fn trim_left_matches(&self, byte: u8) -> &OsStr {
        let mut found = false;
        for (i, b) in self.as_bytes().iter().enumerate() {
            if b != &byte {
                return OsStr::from_bytes(&self.as_bytes()[i..]);
            } else {
                found = true;
            }
        }
        if found {
            return OsStr::from_bytes(&self.as_bytes()[self.len()..]);
        }
        &*self
    }

    fn split_at(&self, i: usize) -> (&OsStr, &OsStr) {
        (
            OsStr::from_bytes(&self.as_bytes()[..i]),
            OsStr::from_bytes(&self.as_bytes()[i..]),
        )
    }

    fn split(&self, b: u8) -> OsSplit {
        OsSplit {
            sep: b,
            val: self.as_bytes(),
            pos: 0,
        }
    }
}

#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct OsSplit<'a> {
    sep: u8,
    val: &'a [u8],
    pos: usize,
}

impl<'a> Iterator for OsSplit<'a> {
    type Item = &'a OsStr;

    fn next(&mut self) -> Option<&'a OsStr> {
        debugln!("OsSplit::next: self={:?}", self);
        if self.pos == self.val.len() {
            return None;
        }
        let start = self.pos;
        for b in &self.val[start..] {
            self.pos += 1;
            if *b == self.sep {
                return Some(OsStr::from_bytes(&self.val[start..self.pos - 1]));
            }
        }
        Some(OsStr::from_bytes(&self.val[start..]))
    }
}
