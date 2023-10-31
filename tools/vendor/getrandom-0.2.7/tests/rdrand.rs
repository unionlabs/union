// We only test the RDRAND-based RNG source on supported architectures.
#![cfg(any(target_arch = "x86_64", target_arch = "x86"))]

// rdrand.rs expects to be part of the getrandom main crate, so we need these
// additional imports to get rdrand.rs to compile.
use getrandom::Error;
#[macro_use]
extern crate cfg_if;
#[path = "../src/rdrand.rs"]
mod rdrand;
#[path = "../src/util.rs"]
mod util;

use rdrand::getrandom_inner as getrandom_impl;
mod common;
