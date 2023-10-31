# strsim-rs [![Crates.io](https://img.shields.io/crates/v/strsim.svg)](https://crates.io/crates/strsim) [![Crates.io](https://img.shields.io/crates/l/strsim.svg?maxAge=2592000)](https://github.com/dguo/strsim-rs/blob/master/LICENSE) [![Linux build status](https://travis-ci.org/dguo/strsim-rs.svg?branch=master)](https://travis-ci.org/dguo/strsim-rs) [![Windows build status](https://ci.appveyor.com/api/projects/status/ggue6i785618a39w?svg=true)](https://ci.appveyor.com/project/dguo/strsim-rs)

[Rust](https://www.rust-lang.org) implementations of [string similarity metrics]:
  - [Hamming]
  - [Levenshtein] - distance & normalized
  - [Optimal string alignment]
  - [Damerau-Levenshtein] - distance & normalized
  - [Jaro and Jaro-Winkler] - this implementation of Jaro-Winkler does not limit the common prefix length

### Installation
```toml
# Cargo.toml
[dependencies]
strsim = "0.8.0"
```

### [Documentation](https://docs.rs/strsim/)
You can change the version in the url to see the documentation for an older
version in the
[changelog](https://github.com/dguo/strsim-rs/blob/master/CHANGELOG.md).

### Usage
```rust
extern crate strsim;

use strsim::{hamming, levenshtein, normalized_levenshtein, osa_distance,
             damerau_levenshtein, normalized_damerau_levenshtein, jaro,
             jaro_winkler};

fn main() {
    match hamming("hamming", "hammers") {
        Ok(distance) => assert_eq!(3, distance),
        Err(why) => panic!("{:?}", why)
    }

    assert_eq!(3, levenshtein("kitten", "sitting"));

    assert!((normalized_levenshtein("kitten", "sitting") - 0.57142).abs() < 0.00001);

    assert_eq!(3, osa_distance("ac", "cba"));

    assert_eq!(2, damerau_levenshtein("ac", "cba"));

    assert!((normalized_damerau_levenshtein("levenshtein", "löwenbräu") - 0.27272).abs() < 0.00001)

    assert!((0.392 - jaro("Friedrich Nietzsche", "Jean-Paul Sartre")).abs() <
            0.001);

    assert!((0.911 - jaro_winkler("cheeseburger", "cheese fries")).abs() <
            0.001);
}
```

### Development
If you don't want to install Rust itself, you can run `$ ./dev` for a
development CLI if you have [Docker] installed.

Benchmarks require a Nightly toolchain. They are run by `cargo +nightly bench`.

### License
[MIT](https://github.com/dguo/strsim-rs/blob/master/LICENSE)

[string similarity metrics]:http://en.wikipedia.org/wiki/String_metric
[Damerau-Levenshtein]:http://en.wikipedia.org/wiki/Damerau%E2%80%93Levenshtein_distance
[Jaro and Jaro-Winkler]:http://en.wikipedia.org/wiki/Jaro%E2%80%93Winkler_distance
[Levenshtein]:http://en.wikipedia.org/wiki/Levenshtein_distance
[Hamming]:http://en.wikipedia.org/wiki/Hamming_distance
[Optimal string alignment]:https://en.wikipedia.org/wiki/Damerau%E2%80%93Levenshtein_distance#Optimal_string_alignment_distance
[Docker]:https://docs.docker.com/engine/installation/
