winapi-util
===========
This crate provides a smattering of safe wrappers around various parts of the
[winapi](https://crates.io/crates/winapi) crate.

[![Build status](https://github.com/BurntSushi/winapi-util/workflows/ci/badge.svg)](https://github.com/BurntSushi/winapi-util/actions)
[![](http://meritbadge.herokuapp.com/winapi-util)](https://crates.io/crates/winapi-util)

Dual-licensed under MIT or the [UNLICENSE](http://unlicense.org).


### Documentation

https://docs.rs/winapi-util


### Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
winapi-util = "0.1"
```


### Notes

This crate was born out of frustration with having to write lots of little
ffi utility bindings in a variety of crates in order to get Windows support.
Eventually, I started needing to copy & paste a lot of those utility routines.
Since they are utility routines, they often don't make sense to expose directly
in the crate in which they are defined. Instead of continuing this process,
I decided to make a crate instead.

Normally, I'm not a huge fan of "utility" crates like this that don't have a
well defined scope, but this is primarily a practical endeavor to make it
easier to isolate Windows specific ffi code.

While I don't have a long term vision for this crate, I will welcome additional
PRs that add more high level routines/types on an as-needed basis.

**WARNING:** I am not a Windows developer, so extra review to make sure I've
got things right is most appreciated.


### Minimum Rust version policy

This crate's minimum supported `rustc` version is `1.34.0`.

The current policy is that the minimum Rust version required to use this crate
can be increased in non-breaking version updates. For example, if `crate 1.0`
requires Rust 1.20.0, then `crate 1.0.z` for all values of `z` will also
require Rust 1.20.0 or newer. However, `crate 1.y` for `y > 0` may require a
newer minimum version of Rust.

In general, this crate will be conservative with respect to the minimum
supported version of Rust.
