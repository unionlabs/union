@echo off
rem Copyright 2018 Google Inc. All rights reserved.
rem
rem Licensed under the Apache License, Version 2.0 (the "License");
rem you may not use this file except in compliance with the License.
rem You may obtain a copy of the License at
rem
rem     http://www.apache.org/licenses/LICENSE-2.0
rem
rem Unless required by applicable law or agreed to in writing, software
rem distributed under the License is distributed on an "AS IS" BASIS,
rem WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
rem See the License for the specific language governing permissions and
rem limitations under the License.

rem Compile then run the Rust test.

cd rust_serialize_test
cargo run -- --quiet || exit /b 1

cd ../rust_no_std_compilation_test
rustup install nightly
rustup component add rust-src --toolchain nightly
rustup target add thumbv7m-none-eabi
cargo +nightly build || exit /b 1

cd ../rust_usage_test
cargo test -- --quiet || exit /b 1
cargo run --bin=flatbuffers_alloc_check || exit /b 1
cargo run --bin=flexbuffers_alloc_check || exit /b 1
cargo run --bin=monster_example || exit /b 1
cd ..

