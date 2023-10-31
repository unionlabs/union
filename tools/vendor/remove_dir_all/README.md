# remove_dir_all

[![Latest Version](https://img.shields.io/crates/v/remove_dir_all.svg)](https://crates.io/crates/remove_dir_all)
[![Docs](https://docs.rs/remove_dir_all/badge.svg)](https://docs.rs/remove_dir_all)
[![License](https://img.shields.io/crates/l/remove_dir_all.svg)](https://github.com/XAMPPRocky/remove_dir_all)

## Description

Reliable and fast directory removal functions.

* `remove_dir_all` - on non-Windows this is a re-export of
  `std::fs::remove_dir_all`. For Windows an implementation that handles the
  locking of directories that occurs when deleting directory trees rapidly.

* `remove_dir_contents` - as for `remove_dir_all` but does not delete the
  supplied root directory.

* `ensure_empty_dir` - as for `remove_dir_contents` but will create the
  directory if it does not exist.

```rust,no_run
extern crate remove_dir_all;

use remove_dir_all::*;

fn main() {
    remove_dir_all("./temp/").unwrap();
    remove_dir_contents("./cache/").unwrap();
}
```

## Minimum Rust Version
The minimum rust version for `remove_dir_all` is the latest stable release, and the minimum version may be bumped through patch releases. You can pin to a specific version by setting by add `=` to your version (e.g. `=0.6.0`), or commiting a `Cargo.lock` file to your project.
