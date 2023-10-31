#![cfg(not(miri))]
#![warn(rust_2018_idioms, single_use_lifetimes)]

use std::{
    env,
    process::{Command, ExitStatus, Stdio},
};

const PATH: &str = "tests/expand/**/*.rs";

#[rustversion::attr(not(nightly), ignore)]
#[test]
fn expandtest() {
    let is_ci = env::var_os("CI").is_some();
    let cargo = &*env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    if !has_command(&[cargo, "expand"]) || !has_command(&[cargo, "fmt"]) {
        if is_ci {
            panic!("expandtest requires rustfmt and cargo-expand");
        }
        return;
    }

    let args = &["--all-features"];
    if is_ci {
        macrotest::expand_without_refresh_args(PATH, args);
    } else {
        env::set_var("MACROTEST", "overwrite");
        macrotest::expand_args(PATH, args);
    }
}

fn has_command(command: &[&str]) -> bool {
    Command::new(command[0])
        .args(&command[1..])
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .as_ref()
        .map(ExitStatus::success)
        .unwrap_or(false)
}
