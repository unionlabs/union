# ibc-proto-rs

[![Crate][crate-image]][crate-link]
[![Docs][docs-image]][docs-link]
[![Build Status][build-image]][build-link]
[![Apache 2.0 Licensed][license-image]][license-link]
![Rust Stable][rustc-image]
![Rust 1.56.1+][rustc-version]

Rust crate for interacting with Cosmos SDK IBC structs.
This repository maintains all the data structures relevant for interacting with on-chain IBC data.
Whether you're building an IBC relayer, IBC modules, or any client software that consumes IBC data structures in Rust, you're at the right place.


## If you are a user of this repository

Please check the official [documentation][docs-link].

## If you are a maintainer of this repository

You will likely want to look at [scripts/sync-protobuf.sh](scripts/sync-protobuf.sh).

## Requirements

- Rust 1.56.1+
- [Buf CLI](https://buf.build/product/cli/)

## License

Copyright © 2023 Informal Systems

Licensed under the Apache License, Version 2.0 (the "License");
you may not use the files in this repository except in compliance with the License.
You may obtain a copy of the License at

    https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

[//]: # (badges)

[crate-image]: https://img.shields.io/crates/v/ibc-proto.svg
[crate-link]: https://crates.io/crates/ibc-proto
[docs-image]: https://docs.rs/ibc-proto/badge.svg
[docs-link]: https://docs.rs/ibc-proto/

[build-image]: https://github.com/cosmos/ibc-rs/workflows/Rust/badge.svg
[build-link]: https://github.com/cosmos/ibc-rs/actions?query=workflow%3ARust

[license-image]: https://img.shields.io/badge/license-Apache2.0-blue.svg
[license-link]: https://github.com/cosmos/ibc-proto-rs/blob/main/LICENSE
[rustc-image]: https://img.shields.io/badge/rustc-stable-blue.svg
[rustc-version]: https://img.shields.io/badge/rustc-1.56.1+-blue.svg

[//]: # (general links)

[Cosmos SDK]: https://github.com/cosmos/cosmos-sdk
