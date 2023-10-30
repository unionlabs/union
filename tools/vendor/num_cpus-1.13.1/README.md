# num_cpus

[![crates.io](https://img.shields.io/crates/v/num_cpus.svg)](https://crates.io/crates/num_cpus)
[![Travis CI Status](https://travis-ci.org/seanmonstar/num_cpus.svg?branch=master)](https://travis-ci.org/seanmonstar/num_cpus)
[![AppVeyor status](https://ci.appveyor.com/api/projects/status/qn8t6grhko5jwno6?svg=true)](https://ci.appveyor.com/project/seanmonstar/num-cpus)

- [Documentation](https://docs.rs/num_cpus)
- [CHANGELOG](CHANGELOG.md)

Count the number of CPUs on the current machine.

## Usage

Add to Cargo.toml:

```toml
[dependencies]
num_cpus = "1.0"
```

In your `main.rs` or `lib.rs`:

```rust
extern crate num_cpus;

// count logical cores this process could try to use
let num = num_cpus::get();
```
