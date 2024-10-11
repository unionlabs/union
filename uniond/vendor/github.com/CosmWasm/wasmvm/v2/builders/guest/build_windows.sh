#!/bin/bash
set -o errexit -o nounset -o pipefail

export CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
export TARGET_DIR="/target" # write to /target in the guest's file system to avoid writing to the host

# ref: https://www.reddit.com/r/rust/comments/5k8uab/crosscompiling_from_ubuntu_to_windows_with_rustup/

cargo build --release --target-dir="$TARGET_DIR" --target x86_64-pc-windows-gnu

cp "$TARGET_DIR/x86_64-pc-windows-gnu/release/wasmvm.dll" artifacts/wasmvm.dll
