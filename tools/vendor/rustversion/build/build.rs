#![allow(
    clippy::enum_glob_use,
    clippy::must_use_candidate,
    clippy::single_match_else
)]

mod rustc;

use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::process::{self, Command};

fn main() {
    println!("cargo:rerun-if-changed=build/build.rs");

    let rustc = env::var_os("RUSTC").unwrap_or_else(|| OsString::from("rustc"));

    let mut is_clippy_driver = false;
    let version = loop {
        let mut command = Command::new(&rustc);
        if is_clippy_driver {
            command.arg("--rustc");
        }
        command.arg("--version");

        let output = match command.output() {
            Ok(output) => output,
            Err(e) => {
                let rustc = rustc.to_string_lossy();
                eprintln!("Error: failed to run `{} --version`: {}", rustc, e);
                process::exit(1);
            }
        };

        let string = match String::from_utf8(output.stdout) {
            Ok(string) => string,
            Err(e) => {
                let rustc = rustc.to_string_lossy();
                eprintln!(
                    "Error: failed to parse output of `{} --version`: {}",
                    rustc, e,
                );
                process::exit(1);
            }
        };

        break match rustc::parse(&string) {
            rustc::ParseResult::Success(version) => version,
            rustc::ParseResult::OopsClippy if !is_clippy_driver => {
                is_clippy_driver = true;
                continue;
            }
            rustc::ParseResult::Unrecognized | rustc::ParseResult::OopsClippy => {
                eprintln!(
                    "Error: unexpected output from `rustc --version`: {:?}\n\n\
                    Please file an issue in https://github.com/dtolnay/rustversion",
                    string
                );
                process::exit(1);
            }
        };
    };

    if version.minor < 38 {
        // Prior to 1.38, a #[proc_macro] is not allowed to be named `cfg`.
        println!("cargo:rustc-cfg=cfg_macro_not_allowed");
    }

    let version = format!("{:#?}\n", version);
    let out_dir = env::var_os("OUT_DIR").expect("OUT_DIR not set");
    let out_file = Path::new(&out_dir).join("version.expr");
    fs::write(out_file, version).expect("failed to write version.expr");
}
