#![cfg_attr(docsrs, deny(broken_intra_doc_links))]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! `ExponentialBackoff` is a backoff implementation that increases the backoff
//! period for each retry attempt using a randomization function that grows exponentially.
//!
//! [`next_backoff`]: backoff/trait.Backoff.html#tymethod.next_backoff
//! [`reset`]: backoff/trait.Backoff.html#tymethod.reset
//!
//! [`next_backoff`] is calculated using the following formula:
//!
//!```text
//!  randomized interval =
//!      retry_interval * (random value in range [1 - randomization_factor, 1 + randomization_factor])
//!```
//!
//! In other words [`next_backoff`] will range between the randomization factor
//! percentage below and above the retry interval.
//!
//! For example, given the following parameters:
//!
//!```text
//!retry_interval = 2
//!randomization_factor = 0.5
//!multiplier = 2
//!```
//!
//! the actual backoff period used in the next retry attempt will range between 1 and 3 seconds,
//! multiplied by the exponential, that is, between 2 and 6 seconds.
//!
//! **Note**: `max_interval` caps the `retry_interval` and not the randomized interval.
//!
//! If the time elapsed since an [`ExponentialBackoff`](type.ExponentialBackoff.html) instance is created goes past the
//! `max_elapsed_time`, then the method [`next_backoff`] starts returning `None`.
//!
//! The elapsed time can be reset by calling [`reset`].
//!
//! Example: Given the following default arguments, for 10 tries the sequence will be,
//! and assuming we go over the `max_elapsed_time` on the 10th try:
//!
//!   Request # | `retry_interval` (seconds) |  Randomized Interval (seconds)
//!  -----------|--------------------------|--------------------------------
//!    1        |  0.5                     | [0.25,   0.75]
//!    2        |  0.75                    | [0.375,  1.125]
//!    3        |  1.125                   | [0.562,  1.687]
//!    4        |  1.687                   | [0.8435, 2.53]
//!    5        |  2.53                    | [1.265,  3.795]
//!    6        |  3.795                   | [1.897,  5.692]
//!    7        |  5.692                   | [2.846,  8.538]
//!    8        |  8.538                   | [4.269, 12.807]
//!    9        | 12.807                   | [6.403, 19.210]
//!   10        | 19.210                   | None
//!
//! # Examples
//!
//! ## Permanent errors
//!
//! Permanent errors are not retried. You have to wrap your error value explicitly
//! into `Error::Permanent`. You can use `Result`'s `map_err` method.
//!
//! `examples/permanent_error.rs`:
//!
//! ```rust,no_run
//! use backoff::{Error, ExponentialBackoff};
//! use reqwest::Url;
//!
//! use std::fmt::Display;
//! use std::io::{self, Read};
//!
//! fn new_io_err<E: Display>(err: E) -> io::Error {
//!     io::Error::new(io::ErrorKind::Other, err.to_string())
//! }
//!
//! fn fetch_url(url: &str) -> Result<String, Error<io::Error>> {
//!     let op = || {
//!         println!("Fetching {}", url);
//!         let url = Url::parse(url)
//!             .map_err(new_io_err)
//!             // Permanent errors need to be explicitly constructed.
//!             .map_err(Error::Permanent)?;
//!
//!         let mut resp = reqwest::blocking::get(url)
//!             // Transient errors can be constructed with the ? operator
//!             // or with the try! macro. No explicit conversion needed
//!             // from E: Error to backoff::Error;
//!             .map_err(new_io_err)?;
//!
//!         let mut content = String::new();
//!         let _ = resp.read_to_string(&mut content);
//!         Ok(content)
//!     };
//!
//!     let backoff = ExponentialBackoff::default();
//!     backoff::retry(backoff, op)
//! }
//!
//! fn main() {
//!     match fetch_url("https::///wrong URL") {
//!         Ok(_) => println!("Successfully fetched"),
//!         Err(err) => panic!("Failed to fetch: {}", err),
//!     }
//! }
//! ```
//!
//! ## Transient errors
//!
//! Transient errors can be constructed by wrapping your error value into `Error::transient`.
//! By using the ? operator or the `try!` macro, you always get transient errors.
//!
//! You can also construct transient errors that are retried after a given
//! interval with `Error::retry_after()` - useful for 429 errors.
//!
//! `examples/retry.rs`:
//!
//! ```rust
//! use backoff::{retry, Error, ExponentialBackoff};
//!
//! use std::io::Read;
//!
//! fn fetch_url(url: &str) -> Result<String, Error<reqwest::Error>> {
//!     let mut op = || {
//!         println!("Fetching {}", url);
//!         let mut resp = reqwest::blocking::get(url)?;
//!
//!         let mut content = String::new();
//!         let _ = resp.read_to_string(&mut content);
//!         Ok(content)
//!     };
//!
//!     let backoff = ExponentialBackoff::default();
//!     retry(backoff, op)
//! }
//!
//! fn main() {
//!     match fetch_url("https://www.rust-lang.org") {
//!         Ok(_) => println!("Sucessfully fetched"),
//!         Err(err) => panic!("Failed to fetch: {}", err),
//!     }
//! }
//! ```
//!
//! Output with internet connection:
//!
//! ```text
//! $ time cargo run --example retry
//!    Compiling backoff v0.1.0 (file:///home/tibi/workspace/backoff)
//!     Finished dev [unoptimized + debuginfo] target(s) in 1.54 secs
//!      Running `target/debug/examples/retry`
//! Fetching https://www.rust-lang.org
//! Sucessfully fetched
//!
//! real    0m2.003s
//! user    0m1.536s
//! sys    0m0.184s
//! ```
//!
//! Output without internet connection
//!
//! ```text
//! $ time cargo run --example retry
//!     Finished dev [unoptimized + debuginfo] target(s) in 0.0 secs
//!      Running `target/debug/examples/retry`
//! Fetching https://www.rust-lang.org
//! Fetching https://www.rust-lang.org
//! Fetching https://www.rust-lang.org
//! Fetching https://www.rust-lang.org
//! ^C
//!
//! real    0m2.826s
//! user    0m0.008s
//! sys    0m0.000s
//! ```
//!
//! ### Async
//!
//! Please set either the `tokio` or `async-std` features in Cargo.toml to enable the async support of this library, i.e.:
//!
//! ```toml
//! backoff = { version = "x.y.z", features = ["tokio"] }
//! ```
//!
//! A `Future<Output = Result<T, backoff::Error<E>>` can be easily retried:
//!
//! `examples/async.rs`:
//!
//! ```rust,no_run,ignore
//!
//! extern crate tokio_1 as tokio;
//!
//! use backoff::ExponentialBackoff;
//!
//! async fn fetch_url(url: &str) -> Result<String, reqwest::Error> {
//!     backoff::future::retry(ExponentialBackoff::default(), || async {
//!         println!("Fetching {}", url);
//!         Ok(reqwest::get(url).await?.text().await?)
//!     })
//!     .await
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     match fetch_url("https://www.rust-lang.org").await {
//!         Ok(_) => println!("Successfully fetched"),
//!         Err(err) => panic!("Failed to fetch: {}", err),
//!     }
//! }
//! ```
//! # Feature flags
//!
//! - `futures`: enables futures support,
//! - `tokio`: enables support for the [tokio](https://crates.io/crates/tokio) async runtime, implies `futures`,
//! - `async-std`: enables support for the [async-std](https://crates.io/crates/async-std) async runtime, implies `futures`,
//! - `wasm-bindgen`: enabled support for [wasm-bindgen](https://crates.io/crates/wasm-bindgen).

pub mod backoff;
mod clock;
pub mod default;
mod error;
pub mod exponential;

#[cfg(feature = "futures")]
#[cfg_attr(docsrs, doc(cfg(feature = "futures")))]
pub mod future;

mod retry;

pub use crate::clock::{Clock, SystemClock};
pub use crate::error::Error;
pub use crate::retry::{retry, retry_notify, Notify};

/// Exponential backoff policy with system's clock.
///
/// This type is preferred over
/// `exponential::ExponentialBackoff` as it is generic over any [Clocks](trait.Clock.html)
/// and in the real world mostly system's clock is used.
pub type ExponentialBackoff = exponential::ExponentialBackoff<SystemClock>;

/// Builder for exponential backoff policy with system's clock.
pub type ExponentialBackoffBuilder = exponential::ExponentialBackoffBuilder<SystemClock>;
