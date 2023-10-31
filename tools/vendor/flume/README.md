# Flume

A blazingly fast multi-producer, multi-consumer channel.

[![Cargo](https://img.shields.io/crates/v/flume.svg)](
https://crates.io/crates/flume)
[![Documentation](https://docs.rs/flume/badge.svg)](
https://docs.rs/flume)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](
https://github.com/zesterer/flume)
![actions-badge](https://github.com/zesterer/flume/workflows/Rust/badge.svg?branch=master)

```rust
use std::thread;

fn main() {
    println!("Hello, world!");

    let (tx, rx) = flume::unbounded();

    thread::spawn(move || {
        (0..10).for_each(|i| {
            tx.send(i).unwrap();
        })
    });

    let received: u32 = rx.iter().sum();

    assert_eq!((0..10).sum::<u32>(), received);
}
```

## Why Flume?

- **Featureful**: Unbounded, bounded and rendezvous queues
- **Fast**: Always faster than `std::sync::mpsc` and sometimes `crossbeam-channel`
- **Safe**: No `unsafe` code anywhere in the codebase!
- **Flexible**: `Sender` and `Receiver` both implement `Send + Sync + Clone`
- **Familiar**: Drop-in replacement for `std::sync::mpsc`
- **Capable**: Additional features like MPMC support and send timeouts/deadlines
- **Simple**: Few dependencies, minimal codebase, fast to compile
- **Asynchronous**: `async` support, including mix 'n match with sync code
- **Ergonomic**: Powerful `select`-like interface

## Usage

To use Flume, place the following line under the `[dependencies]` section in your `Cargo.toml`:

```
flume = "x.y"
```

## [Benchmarks](https://what-if.xkcd.com/147/)

Although Flume has its own extensive benchmarks, don't take it from here that Flume is quick.
The following graph is from the `crossbeam-channel` benchmark suite.

Tests were performed on an AMD Ryzen 7 3700x with 8/16 cores running Linux kernel 5.11.2 with the bfq scheduler.

# <img src="misc/benchmarks.png" alt="Flume benchmarks (crossbeam benchmark suite)" width="100%"/>

## License

Flume is licensed under either of:

- Apache License 2.0, (http://www.apache.org/licenses/LICENSE-2.0)

- MIT license (http://opensource.org/licenses/MIT)
