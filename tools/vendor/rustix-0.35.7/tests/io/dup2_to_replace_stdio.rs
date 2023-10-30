#![cfg(not(target_os = "wasi"))]

use std::env;
use std::process::Command;

/// Use `dup2` to replace the stdin and stdout file descriptors.
#[test]
fn dup2_to_replace_stdio() {
    // This test modifies the stdio file descriptors, so we run it in a
    // separate process so that it doesn't interfere with the test harness.
    assert!(Command::new(env::var("CARGO").unwrap())
        .arg("run")
        .arg("--example")
        .arg("dup2_to_replace_stdio")
        .status()
        .unwrap()
        .success());
}
