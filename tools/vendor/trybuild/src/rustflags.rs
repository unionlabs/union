use std::env;
use std::ffi::OsString;

const RUSTFLAGS: &str = "RUSTFLAGS";
const IGNORED_LINTS: &[&str] = &["dead_code"];

pub fn make_vec() -> Vec<&'static str> {
    let mut rustflags = vec!["--cfg", "trybuild"];

    for &lint in IGNORED_LINTS {
        rustflags.push("-A");
        rustflags.push(lint);
    }

    rustflags
}

pub fn envs() -> impl IntoIterator<Item = (&'static str, OsString)> {
    let mut rustflags = env::var_os(RUSTFLAGS)?;

    for flag in make_vec() {
        rustflags.push(" ");
        rustflags.push(flag);
    }

    Some((RUSTFLAGS, rustflags))
}
