#!/bin/sh

cargo build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/test_contract_type_serialization.wasm ./res/
