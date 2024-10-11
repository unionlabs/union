#!/bin/bash
set -o errexit -o nounset -o pipefail

export CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
export TARGET_DIR="/target" # write to /target in the guest's file system to avoid writing to the host

# No stripping implemented (see https://github.com/CosmWasm/wasmvm/issues/222#issuecomment-2260007943).

echo "Starting x86_64-unknown-linux-gnu build"
export CC=clang
export CXX=clang++
cargo build --release --target-dir="$TARGET_DIR" --target x86_64-unknown-linux-gnu
cp "$TARGET_DIR/x86_64-unknown-linux-gnu/release/libwasmvm.so" artifacts/libwasmvm.x86_64.so
