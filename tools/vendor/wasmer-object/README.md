# `wasmer-object` [![Build Status](https://github.com/wasmerio/wasmer/workflows/build/badge.svg?style=flat-square)](https://github.com/wasmerio/wasmer/actions?query=workflow%3Abuild) [![Join Wasmer Slack](https://img.shields.io/static/v1?label=Slack&message=join%20chat&color=brighgreen&style=flat-square)](https://slack.wasmer.io) [![MIT License](https://img.shields.io/github/license/wasmerio/wasmer.svg?style=flat-square)](https://github.com/wasmerio/wasmer/blob/master/LICENSE)

The Wasmer Native Object crate aims at cross-generating native objects
for various platforms.

This crate is the foundation of [the `wasmer-engine-dylib`
crate](../engine-dylib/). Given a compilation result, i.e. the result
of `wasmer_compiler::Compiler::compile_module`, this crate exposes
functions to create an `Object` file for a given target. It is a
useful thin layer on top of [the `object`
crate](https://github.com/gimli-rs/object).
