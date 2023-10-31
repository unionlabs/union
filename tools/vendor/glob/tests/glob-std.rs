// Copyright 2013-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// ignore-windows TempDir may cause IoError on windows: #10462

#![cfg_attr(test, deny(warnings))]

extern crate glob;
extern crate tempdir;

use glob::glob;
use std::env;
use std::fs;
use std::path::PathBuf;
use tempdir::TempDir;

#[test]
fn main() {
    fn mk_file(path: &str, directory: bool) {
        if directory {
            fs::create_dir(path).unwrap();
        } else {
            fs::File::create(path).unwrap();
        }
    }

    fn glob_vec(pattern: &str) -> Vec<PathBuf> {
        glob(pattern).unwrap().map(|r| r.unwrap()).collect()
    }

    let root = TempDir::new("glob-tests");
    let root = root.ok().expect("Should have created a temp directory");
    assert!(env::set_current_dir(root.path()).is_ok());

    mk_file("aaa", true);
    mk_file("aaa/apple", true);
    mk_file("aaa/orange", true);
    mk_file("aaa/tomato", true);
    mk_file("aaa/tomato/tomato.txt", false);
    mk_file("aaa/tomato/tomoto.txt", false);
    mk_file("bbb", true);
    mk_file("bbb/specials", true);
    mk_file("bbb/specials/!", false);

    // windows does not allow `*` or `?` characters to exist in filenames
    if env::consts::FAMILY != "windows" {
        mk_file("bbb/specials/*", false);
        mk_file("bbb/specials/?", false);
    }

    mk_file("bbb/specials/[", false);
    mk_file("bbb/specials/]", false);
    mk_file("ccc", true);
    mk_file("xyz", true);
    mk_file("xyz/x", false);
    mk_file("xyz/y", false);
    mk_file("xyz/z", false);

    mk_file("r", true);
    mk_file("r/current_dir.md", false);
    mk_file("r/one", true);
    mk_file("r/one/a.md", false);
    mk_file("r/one/another", true);
    mk_file("r/one/another/a.md", false);
    mk_file("r/one/another/deep", true);
    mk_file("r/one/another/deep/spelunking.md", false);
    mk_file("r/another", true);
    mk_file("r/another/a.md", false);
    mk_file("r/two", true);
    mk_file("r/two/b.md", false);
    mk_file("r/three", true);
    mk_file("r/three/c.md", false);

    // all recursive entities
    assert_eq!(
        glob_vec("r/**"),
        vec!(
            PathBuf::from("r/another"),
            PathBuf::from("r/one"),
            PathBuf::from("r/one/another"),
            PathBuf::from("r/one/another/deep"),
            PathBuf::from("r/three"),
            PathBuf::from("r/two")
        )
    );

    // collapse consecutive recursive patterns
    assert_eq!(
        glob_vec("r/**/**"),
        vec!(
            PathBuf::from("r/another"),
            PathBuf::from("r/one"),
            PathBuf::from("r/one/another"),
            PathBuf::from("r/one/another/deep"),
            PathBuf::from("r/three"),
            PathBuf::from("r/two")
        )
    );

    assert_eq!(
        glob_vec("r/**/*"),
        vec!(
            PathBuf::from("r/another"),
            PathBuf::from("r/another/a.md"),
            PathBuf::from("r/current_dir.md"),
            PathBuf::from("r/one"),
            PathBuf::from("r/one/a.md"),
            PathBuf::from("r/one/another"),
            PathBuf::from("r/one/another/a.md"),
            PathBuf::from("r/one/another/deep"),
            PathBuf::from("r/one/another/deep/spelunking.md"),
            PathBuf::from("r/three"),
            PathBuf::from("r/three/c.md"),
            PathBuf::from("r/two"),
            PathBuf::from("r/two/b.md")
        )
    );

    // followed by a wildcard
    assert_eq!(
        glob_vec("r/**/*.md"),
        vec!(
            PathBuf::from("r/another/a.md"),
            PathBuf::from("r/current_dir.md"),
            PathBuf::from("r/one/a.md"),
            PathBuf::from("r/one/another/a.md"),
            PathBuf::from("r/one/another/deep/spelunking.md"),
            PathBuf::from("r/three/c.md"),
            PathBuf::from("r/two/b.md")
        )
    );

    // followed by a precise pattern
    assert_eq!(
        glob_vec("r/one/**/a.md"),
        vec!(
            PathBuf::from("r/one/a.md"),
            PathBuf::from("r/one/another/a.md")
        )
    );

    // followed by another recursive pattern
    // collapses consecutive recursives into one
    assert_eq!(
        glob_vec("r/one/**/**/a.md"),
        vec!(
            PathBuf::from("r/one/a.md"),
            PathBuf::from("r/one/another/a.md")
        )
    );

    // followed by two precise patterns
    assert_eq!(
        glob_vec("r/**/another/a.md"),
        vec!(
            PathBuf::from("r/another/a.md"),
            PathBuf::from("r/one/another/a.md")
        )
    );

    assert_eq!(glob_vec(""), Vec::<PathBuf>::new());
    assert_eq!(glob_vec("."), vec!(PathBuf::from(".")));
    assert_eq!(glob_vec(".."), vec!(PathBuf::from("..")));

    assert_eq!(glob_vec("aaa"), vec!(PathBuf::from("aaa")));
    assert_eq!(glob_vec("aaa/"), vec!(PathBuf::from("aaa")));
    assert_eq!(glob_vec("a"), Vec::<PathBuf>::new());
    assert_eq!(glob_vec("aa"), Vec::<PathBuf>::new());
    assert_eq!(glob_vec("aaaa"), Vec::<PathBuf>::new());

    assert_eq!(glob_vec("aaa/apple"), vec!(PathBuf::from("aaa/apple")));
    assert_eq!(glob_vec("aaa/apple/nope"), Vec::<PathBuf>::new());

    // windows should support both / and \ as directory separators
    if env::consts::FAMILY == "windows" {
        assert_eq!(glob_vec("aaa\\apple"), vec!(PathBuf::from("aaa/apple")));
    }

    assert_eq!(
        glob_vec("???/"),
        vec!(
            PathBuf::from("aaa"),
            PathBuf::from("bbb"),
            PathBuf::from("ccc"),
            PathBuf::from("xyz")
        )
    );

    assert_eq!(
        glob_vec("aaa/tomato/tom?to.txt"),
        vec!(
            PathBuf::from("aaa/tomato/tomato.txt"),
            PathBuf::from("aaa/tomato/tomoto.txt")
        )
    );

    assert_eq!(
        glob_vec("xyz/?"),
        vec!(
            PathBuf::from("xyz/x"),
            PathBuf::from("xyz/y"),
            PathBuf::from("xyz/z")
        )
    );

    assert_eq!(glob_vec("a*"), vec!(PathBuf::from("aaa")));
    assert_eq!(glob_vec("*a*"), vec!(PathBuf::from("aaa")));
    assert_eq!(glob_vec("a*a"), vec!(PathBuf::from("aaa")));
    assert_eq!(glob_vec("aaa*"), vec!(PathBuf::from("aaa")));
    assert_eq!(glob_vec("*aaa"), vec!(PathBuf::from("aaa")));
    assert_eq!(glob_vec("*aaa*"), vec!(PathBuf::from("aaa")));
    assert_eq!(glob_vec("*a*a*a*"), vec!(PathBuf::from("aaa")));
    assert_eq!(glob_vec("aaa*/"), vec!(PathBuf::from("aaa")));

    assert_eq!(
        glob_vec("aaa/*"),
        vec!(
            PathBuf::from("aaa/apple"),
            PathBuf::from("aaa/orange"),
            PathBuf::from("aaa/tomato")
        )
    );

    assert_eq!(
        glob_vec("aaa/*a*"),
        vec!(
            PathBuf::from("aaa/apple"),
            PathBuf::from("aaa/orange"),
            PathBuf::from("aaa/tomato")
        )
    );

    assert_eq!(
        glob_vec("*/*/*.txt"),
        vec!(
            PathBuf::from("aaa/tomato/tomato.txt"),
            PathBuf::from("aaa/tomato/tomoto.txt")
        )
    );

    assert_eq!(
        glob_vec("*/*/t[aob]m?to[.]t[!y]t"),
        vec!(
            PathBuf::from("aaa/tomato/tomato.txt"),
            PathBuf::from("aaa/tomato/tomoto.txt")
        )
    );

    assert_eq!(glob_vec("./aaa"), vec!(PathBuf::from("aaa")));
    assert_eq!(glob_vec("./*"), glob_vec("*"));
    assert_eq!(glob_vec("*/..").pop().unwrap(), PathBuf::from("xyz/.."));
    assert_eq!(glob_vec("aaa/../bbb"), vec!(PathBuf::from("aaa/../bbb")));
    assert_eq!(glob_vec("nonexistent/../bbb"), Vec::<PathBuf>::new());
    assert_eq!(glob_vec("aaa/tomato/tomato.txt/.."), Vec::<PathBuf>::new());

    assert_eq!(glob_vec("aaa/tomato/tomato.txt/"), Vec::<PathBuf>::new());

    assert_eq!(glob_vec("aa[a]"), vec!(PathBuf::from("aaa")));
    assert_eq!(glob_vec("aa[abc]"), vec!(PathBuf::from("aaa")));
    assert_eq!(glob_vec("a[bca]a"), vec!(PathBuf::from("aaa")));
    assert_eq!(glob_vec("aa[b]"), Vec::<PathBuf>::new());
    assert_eq!(glob_vec("aa[xyz]"), Vec::<PathBuf>::new());
    assert_eq!(glob_vec("aa[]]"), Vec::<PathBuf>::new());

    assert_eq!(glob_vec("aa[!b]"), vec!(PathBuf::from("aaa")));
    assert_eq!(glob_vec("aa[!bcd]"), vec!(PathBuf::from("aaa")));
    assert_eq!(glob_vec("a[!bcd]a"), vec!(PathBuf::from("aaa")));
    assert_eq!(glob_vec("aa[!a]"), Vec::<PathBuf>::new());
    assert_eq!(glob_vec("aa[!abc]"), Vec::<PathBuf>::new());

    assert_eq!(
        glob_vec("bbb/specials/[[]"),
        vec!(PathBuf::from("bbb/specials/["))
    );
    assert_eq!(
        glob_vec("bbb/specials/!"),
        vec!(PathBuf::from("bbb/specials/!"))
    );
    assert_eq!(
        glob_vec("bbb/specials/[]]"),
        vec!(PathBuf::from("bbb/specials/]"))
    );

    if env::consts::FAMILY != "windows" {
        assert_eq!(
            glob_vec("bbb/specials/[*]"),
            vec!(PathBuf::from("bbb/specials/*"))
        );
        assert_eq!(
            glob_vec("bbb/specials/[?]"),
            vec!(PathBuf::from("bbb/specials/?"))
        );
    }

    if env::consts::FAMILY == "windows" {
        assert_eq!(
            glob_vec("bbb/specials/[![]"),
            vec!(
                PathBuf::from("bbb/specials/!"),
                PathBuf::from("bbb/specials/]")
            )
        );

        assert_eq!(
            glob_vec("bbb/specials/[!]]"),
            vec!(
                PathBuf::from("bbb/specials/!"),
                PathBuf::from("bbb/specials/[")
            )
        );

        assert_eq!(
            glob_vec("bbb/specials/[!!]"),
            vec!(
                PathBuf::from("bbb/specials/["),
                PathBuf::from("bbb/specials/]")
            )
        );
    } else {
        assert_eq!(
            glob_vec("bbb/specials/[![]"),
            vec!(
                PathBuf::from("bbb/specials/!"),
                PathBuf::from("bbb/specials/*"),
                PathBuf::from("bbb/specials/?"),
                PathBuf::from("bbb/specials/]")
            )
        );

        assert_eq!(
            glob_vec("bbb/specials/[!]]"),
            vec!(
                PathBuf::from("bbb/specials/!"),
                PathBuf::from("bbb/specials/*"),
                PathBuf::from("bbb/specials/?"),
                PathBuf::from("bbb/specials/[")
            )
        );

        assert_eq!(
            glob_vec("bbb/specials/[!!]"),
            vec!(
                PathBuf::from("bbb/specials/*"),
                PathBuf::from("bbb/specials/?"),
                PathBuf::from("bbb/specials/["),
                PathBuf::from("bbb/specials/]")
            )
        );

        assert_eq!(
            glob_vec("bbb/specials/[!*]"),
            vec!(
                PathBuf::from("bbb/specials/!"),
                PathBuf::from("bbb/specials/?"),
                PathBuf::from("bbb/specials/["),
                PathBuf::from("bbb/specials/]")
            )
        );

        assert_eq!(
            glob_vec("bbb/specials/[!?]"),
            vec!(
                PathBuf::from("bbb/specials/!"),
                PathBuf::from("bbb/specials/*"),
                PathBuf::from("bbb/specials/["),
                PathBuf::from("bbb/specials/]")
            )
        );
    }
}
