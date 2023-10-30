# backoff

Exponential backoff and retry.

Inspired by the retry mechanism in Google's [google-http-java-client](https://github.com/google/google-http-java-client) library and
its [Golang port](https://github.com/cenkalti/backoff).

[![Build Status](https://travis-ci.org/ihrwein/backoff.svg?branch=master)](https://travis-ci.org/ihrwein/backoff)
[![crates.io](http://meritbadge.herokuapp.com/backoff)](https://crates.io/crates/backoff)
[![Documentation](https://docs.rs/backoff/badge.svg)](https://docs.rs/backoff)

Compile with feature `wasm-bindgen` or `stdweb` for use in WASM environments. `retry_notify` is not yet supported, as it uses `std::thread::sleep`.

:warning: **BREAKING CHANGES**: migration instructions under [Breaking changes](#breaking-changes).

## Overview

`backoff` is small crate which allows you to retry operations according to backoff policies. It provides:

- Error type to wrap errors as either transient of permanent,
- different backoff algorithms, including exponential,
- supporting both sync and async code.

## Sync example

Just wrap your fallible operation into a closure, and pass it into `retry`:

```rust
use backoff::{retry, ExponentialBackoff, Error};

let op = || {
    reqwest::blocking::get("http://example.com").map_err(Error::transient)
};

let _ = retry(&mut ExponentialBackoff::default(), op);
```

The retry policy will use jitters according to the `randomization_factor` field of `ExponentialBackoff`. Check the documentation for more parameters.

## Async example

Futures are supported by the `futures` module:

```rust
use backoff::ExponentialBackoff;
use backoff::future::retry;

async fn fetch_url(url: &str) -> Result<String, reqwest::Error> {
    retry(ExponentialBackoff::default(), || async {
        println!("Fetching {}", url);
        Ok(reqwest::get(url).await?.text().await?)
    })
    .await
}
```

## Breaking changes

### 0.3.x -> 0.4.x

#### Adding new field to Error::Transient

`Transient` errors got a second field. Useful for handling ratelimits like a HTTP 429 response.

To fix broken code, just replace calls of `Error::Transient()` with `Error::transient()`.

### 0.2.x -> 0.3.x

#### Removal of Operation trait

https://github.com/ihrwein/backoff/pull/28

The `Operation` trait has been removed, please use normal closures implementing `FnMut` instead. The `retry` and `retry_notify` methods were converted to free functions, available in the crate's root.

[Example](examples/retry.rs).

#### Removal of FutureOperation trait

https://github.com/ihrwein/backoff/pull/28

The `FutureOperation` trait has been removed. The `retry` and `retry_notify` methods were converted to free functions, available in the crate's root.

[Example](examples/async.rs).

#### Changes in feature flags

- `stdweb` flag was removed, as the project is abandoned.

#### `retry`, `retry_notify` taking ownership of Backoff instances (previously &mut)

[Example](examples/retry.rs).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
  at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the Work by You, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
