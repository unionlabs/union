#!/bin/sh
set -e # Note we are not using bash here but the Alpine default shell

# This script is called in an Alpine container to build the demo binary in ./cmd/demo.
# We use a script to reduce the escaping hell when passing arguments to the linker.

# See "2. If you really need CGO, but not netcgo" in https://dubo-dubon-duponey.medium.com/a-beginners-guide-to-cross-compiling-static-cgo-pie-binaries-golang-1-16-792eea92d5aa
# See also https://github.com/rust-lang/rust/issues/78919 for why we need -Wl,-z,muldefs
go build -ldflags "-linkmode=external -extldflags '-Wl,-z,muldefs -static'" -tags muslc \
  -o demo ./cmd/demo

# Or static-pie if you really want to
# go build -buildmode=pie -ldflags "-linkmode=external -extldflags '-Wl,-z,muldefs -static-pie'" -tags muslc \
#   -o demo ./cmd/demo
