# Test data

The contracts here are compilations of the Hackatom contract.

## contract.wasm

Is a symbolic link to a recent hackatom contract.

## corrupted.wasm

A corrupted contract files, created by

```sh
cp contract.wasm corrupted.wasm
printf '\x11\x11\x11\x11\x11\x11\x11\x11' | dd of=corrupted.wasm bs=1 seek=1000 count=8 conv=notrunc
```
