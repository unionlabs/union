#!/bin/bash
set -e
#
# Copyright 2018 Google Inc. All rights reserved.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

if [[ "$1" == "mips-unknown-linux-gnu" ]]; then
    TARGET_FLAG="--target mips-unknown-linux-gnu"
    export CARGO_TARGET_MIPS_UNKNOWN_LINUX_GNU_LINKER=mips-linux-gnu-gcc
    export CARGO_TARGET_MIPS_UNKNOWN_LINUX_GNU_RUNNER="qemu-mips -L /usr/mips-linux-gnu"
fi


function check_test_result() {
    if [[ $? == 0 ]]; then
        echo OK: $1 passed.
    else
        echo KO: $1 failed.
        exit 1
    fi
}

cd ./rust_serialize_test
cargo run $TARGET_FLAG -- --quiet
check_test_result "Rust serde tests"

cd ../rust_no_std_compilation_test
rustup install nightly
rustup component add rust-src --toolchain nightly
rustup target add thumbv7m-none-eabi
cargo +nightly build
check_test_result "Rust flatbuffers test no_std compilation"

cd ../rust_usage_test
cargo test $TARGET_FLAG -- --quiet
check_test_result "Rust tests"

cargo test $TARGET_FLAG --no-default-features -- --quiet
check_test_result "Rust tests (no_std)"

cargo run $TARGET_FLAG --bin=flatbuffers_alloc_check
check_test_result "Rust flatbuffers heap alloc test"

cargo run $TARGET_FLAG --bin=flexbuffers_alloc_check
check_test_result "Rust flexbuffers heap alloc test"

rustup component add clippy
cargo clippy $TARGET_FLAG
check_test_result "No Cargo clippy lints test"

cargo bench $TARGET_FLAG

# This test is dependent on flatc.
if [[ -f ../../flatc ]]; then
    cd outdir
    cargo test
    check_test_result "Rust generated file in \$OUT_DIR"
    cd ..
fi

# RUST_NIGHTLY environment variable set in dockerfile.
if [[ $RUST_NIGHTLY == 1 ]]; then
  rustup +nightly component add miri
  MIRIFLAGS="-Zmiri-disable-isolation" cargo +nightly miri test
fi
