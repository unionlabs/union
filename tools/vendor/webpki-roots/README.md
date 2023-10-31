# webpki-roots
This is a crate containing Mozilla's root certificates for use with
the [webpki](https://github.com/briansmith/webpki) or
[rustls](https://github.com/rustls/rustls) crates.

This crate is inspired by [certifi.io](https://certifi.io/en/latest/) and
uses the services provided by [mkcert.org](https://mkcert.org/).

[![webpki-roots](https://github.com/rustls/webpki-roots/actions/workflows/build.yml/badge.svg?branch=main)](https://github.com/rustls/webpki-roots/actions/workflows/build.yml)
[![Crate](https://img.shields.io/crates/v/webpki-roots.svg)](https://crates.io/crates/webpki-roots)

# License
The underlying data is MPL-licensed, and `src/lib.rs`
is therefore a derived work.

# Regenerating sources
You will need python3 and curl.

Run `build.py` which will output a new version of `src/lib.rs`.  You can now
compare and audit.  The code is generated in deterministic order so changes
to the source should only result from upstream changes.
