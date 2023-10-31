// Copyright 2015-2016 Brian Smith.
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHORS DISCLAIM ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY
// SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION
// OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN
// CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

//! Constant-time operations.

use crate::{c, error};

/// Returns `Ok(())` if `a == b` and `Err(error::Unspecified)` otherwise.
/// The comparison of `a` and `b` is done in constant time with respect to the
/// contents of each, but NOT in constant time with respect to the lengths of
/// `a` and `b`.
pub fn verify_slices_are_equal(a: &[u8], b: &[u8]) -> Result<(), error::Unspecified> {
    if a.len() != b.len() {
        return Err(error::Unspecified);
    }
    let result = unsafe { GFp_memcmp(a.as_ptr(), b.as_ptr(), a.len()) };
    match result {
        0 => Ok(()),
        _ => Err(error::Unspecified),
    }
}

extern "C" {
    fn GFp_memcmp(a: *const u8, b: *const u8, len: c::size_t) -> c::int;
}

#[cfg(test)]
mod tests {
    use crate::{bssl, error};

    #[test]
    fn test_constant_time() -> Result<(), error::Unspecified> {
        extern "C" {
            fn bssl_constant_time_test_main() -> bssl::Result;
        }
        Result::from(unsafe { bssl_constant_time_test_main() })
    }
}
