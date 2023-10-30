//! io-lifetimes provides safe, portable, and convenient conversions from types
//! implementing `IntoFilelike` and `FromSocketlike` to types implementing
//! `FromFilelike` and `IntoSocketlike`, respectively.

#![cfg_attr(io_lifetimes_use_std, feature(io_safety))]

use io_lifetimes::FromFilelike;
use std::fs::File;
use std::io::{self, Read};
use std::process::{Command, Stdio};

fn main() -> io::Result<()> {
    let mut child = Command::new("cargo")
        .arg("--help")
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to execute child");

    // Convert from `ChildStderr` into `File` without any platform-specific
    // code or `unsafe`!
    let mut file = File::from_into_filelike(child.stdout.take().unwrap());

    // Well, this example is not actually that cool, because `File` doesn't let
    // you do anything that you couldn't already do with `ChildStderr` etc., but
    // it's useful outside of standard library types.
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;

    Ok(())
}
