# Tokio

A runtime for writing reliable, asynchronous, and slim applications with
the Rust programming language. It is:

* **Fast**: Tokio's zero-cost abstractions give you bare-metal
  performance.

* **Reliable**: Tokio leverages Rust's ownership, type system, and
  concurrency model to reduce bugs and ensure thread safety.

* **Scalable**: Tokio has a minimal footprint, and handles backpressure
  and cancellation naturally.

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]
[![Discord chat][discord-badge]][discord-url]

[crates-badge]: https://img.shields.io/crates/v/tokio.svg
[crates-url]: https://crates.io/crates/tokio
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/tokio-rs/tokio/blob/master/LICENSE
[actions-badge]: https://github.com/tokio-rs/tokio/workflows/CI/badge.svg
[actions-url]: https://github.com/tokio-rs/tokio/actions?query=workflow%3ACI+branch%3Amaster
[discord-badge]: https://img.shields.io/discord/500028886025895936.svg?logo=discord&style=flat-square
[discord-url]: https://discord.gg/tokio

[Website](https://tokio.rs) |
[Guides](https://tokio.rs/tokio/tutorial) |
[API Docs](https://docs.rs/tokio/latest/tokio) |
[Chat](https://discord.gg/tokio)

## Overview

Tokio is an event-driven, non-blocking I/O platform for writing
asynchronous applications with the Rust programming language. At a high
level, it provides a few major components:

* A multithreaded, work-stealing based task [scheduler].
* A reactor backed by the operating system's event queue (epoll, kqueue,
  IOCP, etc...).
* Asynchronous [TCP and UDP][net] sockets.

These components provide the runtime components necessary for building
an asynchronous application.

[net]: https://docs.rs/tokio/latest/tokio/net/index.html
[scheduler]: https://docs.rs/tokio/latest/tokio/runtime/index.html

## Example

A basic TCP echo server with Tokio.

Make sure you activated the full features of the tokio crate on Cargo.toml:

```toml
[dependencies]
tokio = { version = "1.20.1", features = ["full"] }
```
Then, on your main.rs:

```rust,no_run
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = [0; 1024];

            // In a loop, read data from the socket and write the data back.
            loop {
                let n = match socket.read(&mut buf).await {
                    // socket closed
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                // Write the data back
                if let Err(e) = socket.write_all(&buf[0..n]).await {
                    eprintln!("failed to write to socket; err = {:?}", e);
                    return;
                }
            }
        });
    }
}
```

More examples can be found [here][examples]. For a larger "real world" example, see the
[mini-redis] repository.

[examples]: https://github.com/tokio-rs/tokio/tree/master/examples
[mini-redis]: https://github.com/tokio-rs/mini-redis/

To see a list of the available features flags that can be enabled, check our
[docs][feature-flag-docs].

## Getting Help

First, see if the answer to your question can be found in the [Guides] or the
[API documentation]. If the answer is not there, there is an active community in
the [Tokio Discord server][chat]. We would be happy to try to answer your
question. You can also ask your question on [the discussions page][discussions].

[Guides]: https://tokio.rs/tokio/tutorial
[API documentation]: https://docs.rs/tokio/latest/tokio
[chat]: https://discord.gg/tokio
[discussions]: https://github.com/tokio-rs/tokio/discussions
[feature-flag-docs]: https://docs.rs/tokio/#feature-flags

## Contributing

:balloon: Thanks for your help improving the project! We are so happy to have
you! We have a [contributing guide][guide] to help you get involved in the Tokio
project.

[guide]: https://github.com/tokio-rs/tokio/blob/master/CONTRIBUTING.md

## Related Projects

In addition to the crates in this repository, the Tokio project also maintains
several other libraries, including:

* [`hyper`]: A fast and correct HTTP/1.1 and HTTP/2 implementation for Rust.

* [`tonic`]: A gRPC over HTTP/2 implementation focused on high performance, interoperability, and flexibility.

* [`warp`]: A super-easy, composable, web server framework for warp speeds.

* [`tower`]: A library of modular and reusable components for building robust networking clients and servers.

* [`tracing`] (formerly `tokio-trace`): A framework for application-level tracing and async-aware diagnostics.

* [`rdbc`]: A Rust database connectivity library for MySQL, Postgres and SQLite.

* [`mio`]: A low-level, cross-platform abstraction over OS I/O APIs that powers
  `tokio`.

* [`bytes`]: Utilities for working with bytes, including efficient byte buffers.

* [`loom`]: A testing tool for concurrent Rust code

[`warp`]: https://github.com/seanmonstar/warp
[`hyper`]: https://github.com/hyperium/hyper
[`tonic`]: https://github.com/hyperium/tonic
[`tower`]: https://github.com/tower-rs/tower
[`loom`]: https://github.com/tokio-rs/loom
[`rdbc`]: https://github.com/tokio-rs/rdbc
[`tracing`]: https://github.com/tokio-rs/tracing
[`mio`]: https://github.com/tokio-rs/mio
[`bytes`]: https://github.com/tokio-rs/bytes

## Supported Rust Versions

<!--
When updating this, also update:
- .github/workflows/ci.yml
- CONTRIBUTING.md
- README.md
- tokio/README.md
- tokio/Cargo.toml
- tokio-util/Cargo.toml
- tokio-test/Cargo.toml
- tokio-stream/Cargo.toml
-->

Tokio will keep a rolling MSRV (minimum supported rust version) policy of **at
least** 6 months. When increasing the MSRV, the new Rust version must have been
released at least six months ago. The current MSRV is 1.49.0.

## Release schedule

Tokio doesn't follow a fixed release schedule, but we typically make one to two
new minor releases each month. We make patch releases for bugfixes as necessary.

## Bug patching policy

For the purposes of making patch releases with bugfixes, we have designated
certain minor releases as LTS (long term support) releases. Whenever a bug
warrants a patch release with a fix for the bug, it will be backported and
released as a new patch release for each LTS minor version. Our current LTS
releases are:

 * `1.14.x` - LTS release until June 2022.
 * `1.18.x` - LTS release until January 2023

Each LTS release will continue to receive backported fixes for at least half a
year. If you wish to use a fixed minor release in your project, we recommend
that you use an LTS release.

To use a fixed minor version, you can specify the version with a tilde. For
example, to specify that you wish to use the newest `1.14.x` patch release, you
can use the following dependency specification:
```text
tokio = { version = "~1.14", features = [...] }
```

## License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/tokio-rs/tokio/blob/master/LICENSE

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Tokio by you, shall be licensed as MIT, without any additional
terms or conditions.
