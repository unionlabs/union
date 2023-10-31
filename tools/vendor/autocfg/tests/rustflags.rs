extern crate autocfg;

use std::env;

/// Tests that autocfg uses the RUSTFLAGS or CARGO_ENCODED_RUSTFLAGS
/// environment variables when running rustc.
#[test]
fn test_with_sysroot() {
    // Use the same path as this test binary.
    let dir = env::current_exe().unwrap().parent().unwrap().to_path_buf();
    env::set_var("OUT_DIR", &format!("{}", dir.display()));

    // If we have encoded rustflags, they take precedence, even if empty.
    env::set_var("CARGO_ENCODED_RUSTFLAGS", "");
    env::set_var("RUSTFLAGS", &format!("-L {}", dir.display()));
    let ac = autocfg::AutoCfg::new().unwrap();
    assert!(ac.probe_sysroot_crate("std"));
    assert!(!ac.probe_sysroot_crate("autocfg"));

    // Now try again with useful encoded args.
    env::set_var(
        "CARGO_ENCODED_RUSTFLAGS",
        &format!("-L\x1f{}", dir.display()),
    );
    let ac = autocfg::AutoCfg::new().unwrap();
    assert!(ac.probe_sysroot_crate("autocfg"));

    // Try the old-style RUSTFLAGS, ensuring HOST != TARGET.
    env::remove_var("CARGO_ENCODED_RUSTFLAGS");
    env::set_var("HOST", "lol");
    let ac = autocfg::AutoCfg::new().unwrap();
    assert!(ac.probe_sysroot_crate("autocfg"));
}
