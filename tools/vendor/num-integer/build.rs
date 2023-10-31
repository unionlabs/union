extern crate autocfg;

use std::env;

fn main() {
    // If the "i128" feature is explicity requested, don't bother probing for it.
    // It will still cause a build error if that was set improperly.
    if env::var_os("CARGO_FEATURE_I128").is_some() || autocfg::new().probe_type("i128") {
        autocfg::emit("has_i128");
    }

    autocfg::rerun_path("build.rs");
}
