<h1 align="center">async-std</h1>
<div align="center">
 <strong>
   Async version of the Rust standard library
 </strong>
</div>

<br />

<div align="center">
   <!-- CI status -->
  <a href="https://github.com/async-rs/async-std/actions">
    <img src="https://github.com/async-rs/async-std/workflows/CI/badge.svg"
      alt="CI Status" />
  </a>
  <!-- Crates version -->
  <a href="https://crates.io/crates/async-std">
    <img src="https://img.shields.io/crates/v/async-std.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/async-std">
    <img src="https://img.shields.io/crates/d/async-std.svg?style=flat-square"
      alt="Download" />
  </a>
  <!-- docs.rs docs -->
  <a href="https://docs.rs/async-std">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>

  <a href="https://discord.gg/JvZeVNe">
    <img src="https://img.shields.io/discord/598880689856970762.svg?logo=discord&style=flat-square"
      alt="chat" />
  </a>
</div>

<div align="center">
  <h3>
    <a href="https://docs.rs/async-std">
      API Docs
    </a>
    <span> | </span>
    <a href="https://book.async.rs">
      Book
    </a>
    <span> | </span>
    <a href="https://github.com/async-rs/async-std/releases">
      Releases
    </a>
    <span> | </span>
    <a href="https://async.rs/contribute">
      Contributing
    </a>
  </h3>
</div>

<br/>

This crate provides an async version of [`std`]. It provides all the interfaces
you are used to, but in an async version and ready for Rust's `async`/`await`
syntax.

[`std`]: https://doc.rust-lang.org/std/index.html

## Features

- __Modern:__ Built from the ground up for `std::future` and `async/await` with
    blazing fast compilation time.
- __Fast:__ Our robust allocator and threadpool designs provide ultra-high
    throughput with predictably low latency.
- __Intuitive:__ Complete parity with the stdlib means you only need to learn
    APIs once.
- __Clear:__ [Detailed documentation][docs] and [accessible guides][book] mean
    using async Rust was never easier.

[docs]: https://docs.rs/async-std
[book]: https://book.async.rs

## Examples

```rust
use async_std::task;

async fn say_hello() {
    println!("Hello, world!");
}

fn main() {
    task::block_on(say_hello())
}
```

More examples, including networking and file access, can be found in our
[`examples`] directory and in our [documentation].

[`examples`]: https://github.com/async-rs/async-std/tree/HEAD/examples
[documentation]: https://docs.rs/async-std#examples
[`task::block_on`]: https://docs.rs/async-std/*/async_std/task/fn.block_on.html
[`"attributes"` feature]: https://docs.rs/async-std/#features

## Philosophy

We believe Async Rust should be as easy to pick up as Sync Rust. We also believe
that the best API is the one you already know. And finally, we believe that
providing an asynchronous counterpart to the standard library is the best way
stdlib provides a reliable basis for both performance and productivity.

Async-std is the embodiment of that vision. It combines single-allocation task
creation, with an adaptive lock-free executor, threadpool and network driver to
create a smooth system that processes work at a high pace with low latency,
using Rust's familiar stdlib API.

## Installation

With [cargo-edit](https://github.com/killercup/cargo-edit) installed run:

```sh
$ cargo add async-std
```

We also provide a set of "unstable" features with async-std. See the [features
documentation] on how to enable them.

[cargo-add]: https://github.com/killercup/cargo-edit
[features documentation]: https://docs.rs/async-std/#features

## Ecosystem
 
 * [async-tls](https://crates.io/crates/async-tls) — Async TLS/SSL streams using **Rustls**. 
  
 * [async-native-tls](https://crates.io/crates/async-native-tls) — **Native TLS** for Async. Native TLS for futures and async-std.
 
 * [async-tungstenite](https://crates.io/crates/async-tungstenite) — Asynchronous **WebSockets** for async-std, tokio, gio and any std Futures runtime.
 
 * [Tide](https://crates.io/crates/tide) — Serve the web. A modular **web framework** built around async/await.

 * [SQLx](https://crates.io/crates/sqlx) — The Rust **SQL** Toolkit. SQLx is a 100% safe Rust library for Postgres and MySQL with compile-time checked queries.

 * [Surf](https://crates.io/crates/surf) — Surf the web. Surf is a friendly **HTTP client** built for casual Rustaceans and veterans alike.
 
 * [Xactor](https://crates.io/crates/xactor) — Xactor is a rust actors framework based on async-std.
 
 * [async-graphql](https://crates.io/crates/async-graphql) — A GraphQL server library implemented in rust, with full support for async/await.
 
## License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br/>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
