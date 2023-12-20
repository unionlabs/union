// file is a modified version of  https://github.com/AltSysrq/proptest/blob/proptest-derive/proptest-derive/tests/compiletest.rs

// Original copyright and license:
// Copyright 2018 The proptest developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Modifications copyright 2020 Wasmer
// Licensed under the MIT license

extern crate compiletest_rs as ct;

use std::env;

fn run_mode(src: &'static str, mode: &'static str) {
    let mut config = ct::Config::default();

    config.mode = mode.parse().expect("invalid mode");
    config.target_rustcflags = Some("-L ../../target/debug/deps".to_owned());
    if let Ok(name) = env::var("TESTNAME") {
        config.filters.push(name);
    }
    config.src_base = format!("tests/{}", src).into();

    // hack to make this work on OSX: we probably don't need it though
    /*if std::env::var("DYLD_LIBRARY_PATH").is_err() {
        let val =    std::env::var("DYLD_FALLBACK_LIBRARY_PATH").unwrap();
        std::env::set_var("DYLD_LIBRARY_PATH", val);
    }
    config.link_deps();*/

    // Uncomment this if you have the "multiple crates named `wasmer` issue". Massively slows
    // down test iteration though...
    config.clean_rmeta();

    ct::run_tests(&config);
}

#[test]
#[ignore] // ignored by default because it needs to essentially run `cargo clean` to work correctly
          // and that's really, really slow
fn compile_test() {
    run_mode("compile-fail", "compile-fail");
}
