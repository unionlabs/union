#!/bin/sh
set -e # Note we are not using bash here but the Alpine default shell

export CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
export TARGET_DIR="/target" # write to /target in the guest's file system to avoid writing to the host

# No stripping implemented (see https://github.com/CosmWasm/wasmvm/issues/222#issuecomment-2260007943).

echo "Starting aarch64-unknown-linux-musl build"
export CC=/opt/aarch64-linux-musl-cross/bin/aarch64-linux-musl-gcc
cargo build --release --target-dir="$TARGET_DIR" --target aarch64-unknown-linux-musl --example wasmvmstatic
unset CC

echo "Starting x86_64-unknown-linux-musl build"
cargo build --release --target-dir="$TARGET_DIR" --target x86_64-unknown-linux-musl --example wasmvmstatic

cp "$TARGET_DIR/aarch64-unknown-linux-musl/release/examples/libwasmvmstatic.a" artifacts/libwasmvm_muslc.aarch64.a
cp "$TARGET_DIR/x86_64-unknown-linux-musl/release/examples/libwasmvmstatic.a" artifacts/libwasmvm_muslc.x86_64.a
