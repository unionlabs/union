# error-code

[![Crates.io](https://img.shields.io/crates/v/error-code.svg)](https://crates.io/crates/error-code)
[![Documentation](https://docs.rs/error-code/badge.svg)](https://docs.rs/crate/error-code/)
[![Build](https://github.com/DoumanAsh/error-code/workflows/Rust/badge.svg)](https://github.com/DoumanAsh/error-code/actions?query=workflow%3ARust)

Alternative `Error` for Rust.

It's goal is to be able to provide simplified `Error` which would work in `no_std` environment

# Features

- `std` - enables `std::error::Error` implementation

# Categories

Library introduces the concept of categories, similar to that of C++ `std::error_category`.
Each category can be used to describe set of integral error codes.

Following implementations are builtin:

- `Posix` - POSIX category. To access integer constants use [libc](https://crates.io/crates/libc)
- `System` - System category. To access integer constants use [libc](https://crates.io/crates/libc) on unix, and [winapi](https://crates.io/crates/winapi) on Windows
- `Plain` - Plain errors without any category.
