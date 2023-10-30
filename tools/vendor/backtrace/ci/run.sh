#!/bin/sh

set -ex

cargo test --target $TARGET
cargo build --target $TARGET --manifest-path crates/as-if-std/Cargo.toml
