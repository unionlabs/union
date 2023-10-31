# Contributing to `wasm-bindgen`

This section contains instructions on how to get this project up and running for
development. You may want to browse the [unpublished guide documentation] for
`wasm-bindgen` as well as it may have more up-to-date information.

[unpublished documentation]: https://rustwasm.github.io/wasm-bindgen/

## Prerequisites

1. Rust. [Install Rust]. Once Rust is installed, run

    ```shell
    rustup target add wasm32-unknown-unknown
    ```

[install Rust]: https://www.rust-lang.org/en-US/install.html

2. The tests for this project use Node. Make sure you have node >= 10 installed,
   as that is when WebAssembly support was introduced. [Install Node].

[Install Node]: https://nodejs.org/en/

## Code Formatting

Although formatting rules are not mandatory, it is encouraged to run `cargo run` (`rustfmt`) with its default rules within a PR to maintain a more organized code base. If necessary, a PR with a single commit that formats the entire project is also welcome.