# `wasmer-vm` [![Build Status](https://github.com/wasmerio/wasmer/workflows/build/badge.svg?style=flat-square)](https://github.com/wasmerio/wasmer/actions?query=workflow%3Abuild) [![Join Wasmer Slack](https://img.shields.io/static/v1?label=Slack&message=join%20chat&color=brighgreen&style=flat-square)](https://slack.wasmer.io) [![MIT License](https://img.shields.io/github/license/wasmerio/wasmer.svg?style=flat-square)](https://github.com/wasmerio/wasmer/blob/master/LICENSE)

This crate contains the Wasmer VM runtime library, supporting the Wasm ABI used by wasmer.

The Wasmer runtime is modular by design, and provides several
libraries where each of them provides a specific set of features. This
`wasmer-vm` library contains the low-level foundation for the runtime
itself.

It provides all the APIs wasmer needs to operate,
from the `instance`, to `memory`, `probestack`, signature registry, `trap`,
`table`, `VMContext`, `libcalls` etc.

It is very unlikely that a user will need to deal with `wasmer-vm`
directly. The `wasmer` crate provides types that embed types from
`wasmer-vm` with a higher-level API.


[`wasmer`]: https://crates.io/crates/wasmer

### Acknowledgments

This project borrowed some of the code for the VM structure and trapping from the [wasmtime-runtime](https://crates.io/crates/wasmtime-runtime).

Please check [Wasmer ATTRIBUTIONS](https://github.com/wasmerio/wasmer/blob/master/ATTRIBUTIONS.md) to further see licenses and other attributions of the project. 
