# remove_dir_all

[![Latest Version](https://img.shields.io/crates/v/remove_dir_all.svg)](https://crates.io/crates/remove_dir_all)
[![Docs](https://docs.rs/remove_dir_all/badge.svg)](https://docs.rs/remove_dir_all)
[![License](https://img.shields.io/github/license/XAMPPRocky/remove_dir_all.svg)](https://github.com/XAMPPRocky/remove_dir_all)

## Description

A reliable implementation of `remove_dir_all` for Windows. For Unix systems
re-exports `std::fs::remove_dir_all`.

```rust,no_run
extern crate remove_dir_all;

use remove_dir_all::*;

fn main() {
    remove_dir_all("./temp/").unwrap();
}
```
