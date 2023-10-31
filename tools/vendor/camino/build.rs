// Copyright (c) The camino Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Adapted from
//! https://github.com/dtolnay/syn/blob/a54fb0098c6679f1312113ae2eec0305c51c7390/build.rs.

use std::{env, process::Command, str};

// The rustc-cfg strings below are *not* public API. Please let us know by
// opening a GitHub issue if your build environment requires some way to enable
// these cfgs other than by executing our build script.
fn main() {
    let compiler = match rustc_version() {
        Some(compiler) => compiler,
        None => return,
    };

    // NOTE:
    // Adding a new cfg gated by Rust version MUST be accompanied by an addition to the matrix in
    // .github/workflows/ci.yml.
    if compiler.minor >= 44 {
        println!("cargo:rustc-cfg=path_buf_capacity");
    }
}

struct Compiler {
    minor: u32,
}

fn rustc_version() -> Option<Compiler> {
    let rustc = env::var_os("RUSTC")?;
    let output = Command::new(rustc).arg("--version").output().ok()?;
    let version = str::from_utf8(&output.stdout).ok()?;
    let mut pieces = version.split('.');
    if pieces.next() != Some("rustc 1") {
        return None;
    }
    let minor = pieces.next()?.parse().ok()?;
    Some(Compiler { minor })
}
