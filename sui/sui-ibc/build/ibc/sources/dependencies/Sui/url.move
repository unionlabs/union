// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// URL: standard Uniform Resource Locator string
module sui::url;

use std::ascii::String;

/// Standard Uniform Resource Locator (URL) string.
public struct Url has store, copy, drop {
    // TODO: validate URL format
    url: String,
}

/// Create a `Url`, with no validation
public fun new_unsafe(url: String): Url {
    Url { url }
}

/// Create a `Url` with no validation from bytes
/// Note: this will abort if `bytes` is not valid ASCII
public fun new_unsafe_from_bytes(bytes: vector<u8>): Url {
    let url = bytes.to_ascii_string();
    Url { url }
}

/// Get inner URL
public fun inner_url(self: &Url): String {
    self.url
}

/// Update the inner URL
public fun update(self: &mut Url, url: String) {
    self.url = url;
}
