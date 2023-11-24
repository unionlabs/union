.PHONY: all build build-rust build-go test

# Builds the Rust library libwasmvm
BUILDERS_PREFIX := cosmwasm/go-ext-builder:0017
# Contains a full Go dev environment in order to run Go tests on the built library
ALPINE_TESTER := cosmwasm/go-ext-builder:0017-alpine

USER_ID := $(shell id -u)
USER_GROUP = $(shell id -g)

SHARED_LIB_SRC = "" # File name of the shared library as created by the Rust build system
SHARED_LIB_DST = "" # File name of the shared library that we store
ifeq ($(OS),Windows_NT)
	SHARED_LIB_SRC = wasmvm.dll
	SHARED_LIB_DST = wasmvm.dll
else
	UNAME_S := $(shell uname -s)
	ifeq ($(UNAME_S),Linux)
		SHARED_LIB_SRC = libwasmvm.so
		SHARED_LIB_DST = libwasmvm.$(shell rustc --print cfg | grep target_arch | cut  -d '"' -f 2).so
	endif
	ifeq ($(UNAME_S),Darwin)
		SHARED_LIB_SRC = libwasmvm.dylib
		SHARED_LIB_DST = libwasmvm.dylib
	endif
endif

test-filenames:
	echo $(SHARED_LIB_DST)
	echo $(SHARED_LIB_SRC)

all: build test

build: build-rust build-go

build-rust: build-rust-release

# Use debug build for quick testing.
# In order to use "--features backtraces" here we need a Rust nightly toolchain, which we don't have by default
build-rust-debug:
	(cd libwasmvm && cargo build)
	cp libwasmvm/target/debug/$(SHARED_LIB_SRC) internal/api/$(SHARED_LIB_DST)
	make update-bindings

# use release build to actually ship - smaller and much faster
#
# See https://github.com/CosmWasm/wasmvm/issues/222#issuecomment-880616953 for two approaches to
# enable stripping through cargo (if that is desired).
build-rust-release:
	(cd libwasmvm && cargo build --release)
	cp libwasmvm/target/release/$(SHARED_LIB_SRC) internal/api/$(SHARED_LIB_DST)
	make update-bindings
	@ #this pulls out ELF symbols, 80% size reduction!

build-go:
	go build ./...
	go build -o build/demo ./cmd/demo

test:
	# Use package list mode to include all subdirectores. The -count=1 turns off caching.
	RUST_BACKTRACE=1 go test -v -count=1 ./...

test-safety:
	# Use package list mode to include all subdirectores. The -count=1 turns off caching.
	GODEBUG=cgocheck=2 go test -race -v -count=1 ./...

# Creates a release build in a containerized build environment of the static library for Alpine Linux (.a)
release-build-alpine:
	# Builders should not write their target folder into the host file system (https://github.com/CosmWasm/wasmvm/issues/437)
	rm -rf libwasmvm/target/aarch64-unknown-linux-musl/release
	rm -rf libwasmvm/target/x86_64-unknown-linux-musl/release
	# build the muslc *.a file
	docker run --rm -u $(USER_ID):$(USER_GROUP) -v $(shell pwd)/libwasmvm:/code $(BUILDERS_PREFIX)-alpine
	cp libwasmvm/artifacts/libwasmvm_muslc.a internal/api
	cp libwasmvm/artifacts/libwasmvm_muslc.aarch64.a internal/api
	make update-bindings

# Creates a release build in a containerized build environment of the shared library for glibc Linux (.so)
release-build-linux:
	# Builders should not write their target folder into the host file system (https://github.com/CosmWasm/wasmvm/issues/437)
	rm -rf libwasmvm/target/x86_64-unknown-linux-gnu/release
	rm -rf libwasmvm/target/aarch64-unknown-linux-gnu/release
	docker run --rm -u $(USER_ID):$(USER_GROUP) -v $(shell pwd)/libwasmvm:/code $(BUILDERS_PREFIX)-centos7 build_linux.sh
	cp libwasmvm/artifacts/libwasmvm.x86_64.so internal/api
	cp libwasmvm/artifacts/libwasmvm.aarch64.so internal/api
	make update-bindings

# Creates a release build in a containerized build environment of the shared library for macOS (.dylib)
release-build-macos:
	# Builders should not write their target folder into the host file system (https://github.com/CosmWasm/wasmvm/issues/437)
	rm -rf libwasmvm/target/x86_64-apple-darwin/release
	rm -rf libwasmvm/target/aarch64-apple-darwin/release
	docker run --rm -u $(USER_ID):$(USER_GROUP) -v $(shell pwd)/libwasmvm:/code $(BUILDERS_PREFIX)-cross build_macos.sh
	cp libwasmvm/artifacts/libwasmvm.dylib internal/api
	make update-bindings

# Creates a release build in a containerized build environment of the static library for macOS (.a)
release-build-macos-static:
	# Builders should not write their target folder into the host file system (https://github.com/CosmWasm/wasmvm/issues/437)
	rm -rf libwasmvm/target/x86_64-apple-darwin/release
	rm -rf libwasmvm/target/aarch64-apple-darwin/release
	docker run --rm -u $(USER_ID):$(USER_GROUP) -v $(shell pwd)/libwasmvm:/code $(BUILDERS_PREFIX)-cross build_macos_static.sh
	cp libwasmvm/artifacts/libwasmvmstatic_darwin.a internal/api/libwasmvmstatic_darwin.a
	make update-bindings

# Creates a release build in a containerized build environment of the shared library for Windows (.dll)
release-build-windows:
	# Builders should not write their target folder into the host file system (https://github.com/CosmWasm/wasmvm/issues/437)
	rm -rf libwasmvm/target/x86_64-pc-windows-gnu/release
	docker run --rm -u $(USER_ID):$(USER_GROUP) -v $(shell pwd)/libwasmvm:/code $(BUILDERS_PREFIX)-cross build_windows.sh
	cp libwasmvm/target/x86_64-pc-windows-gnu/release/wasmvm.dll internal/api
	make update-bindings

update-bindings:
# After we build libwasmvm, we have to copy the generated bindings for Go code to use.
# We cannot use symlinks as those are not reliably resolved by `go get` (https://github.com/CosmWasm/wasmvm/pull/235).
	cp libwasmvm/bindings.h internal/api

release-build:
	# Write like this because those must not run in parallel
	make release-build-alpine
	make release-build-linux
	make release-build-macos
	make release-build-windows

test-alpine: release-build-alpine
# try running go tests using this lib with muslc
	docker run --rm -u $(USER_ID):$(USER_GROUP) -v $(shell pwd):/mnt/testrun -w /mnt/testrun $(ALPINE_TESTER) go build -tags muslc ./...
# Use package list mode to include all subdirectores. The -count=1 turns off caching.
	docker run --rm -u $(USER_ID):$(USER_GROUP) -v $(shell pwd):/mnt/testrun -w /mnt/testrun $(ALPINE_TESTER) go test -tags muslc -count=1 ./...

	@# Build a Go demo binary called ./demo that links the static library from the previous step.
	@# Whether the result is a statically linked or dynamically linked binary is decided by `go build`
	@# and it's a bit unclear how this is decided. We use `file` to see what we got.
	docker run --rm -u $(USER_ID):$(USER_GROUP) -v $(shell pwd):/mnt/testrun -w /mnt/testrun $(ALPINE_TESTER) ./build_demo.sh
	docker run --rm -u $(USER_ID):$(USER_GROUP) -v $(shell pwd):/mnt/testrun -w /mnt/testrun $(ALPINE_TESTER) file ./demo

	@# Run the demo binary on Alpine machines
	@# See https://de.wikipedia.org/wiki/Alpine_Linux#Versionen for supported versions
	docker run --rm --read-only -v $(shell pwd):/mnt/testrun -w /mnt/testrun alpine:3.18 ./demo ./testdata/hackatom.wasm
	docker run --rm --read-only -v $(shell pwd):/mnt/testrun -w /mnt/testrun alpine:3.17 ./demo ./testdata/hackatom.wasm
	docker run --rm --read-only -v $(shell pwd):/mnt/testrun -w /mnt/testrun alpine:3.16 ./demo ./testdata/hackatom.wasm
	docker run --rm --read-only -v $(shell pwd):/mnt/testrun -w /mnt/testrun alpine:3.15 ./demo ./testdata/hackatom.wasm
	docker run --rm --read-only -v $(shell pwd):/mnt/testrun -w /mnt/testrun alpine:3.14 ./demo ./testdata/hackatom.wasm

	@# Run binary locally if you are on Linux
	@# ./demo ./testdata/hackatom.wasm

.PHONY: format
format:
	find . -name '*.go' -type f | xargs gofumpt -w -s
	find . -name '*.go' -type f | xargs misspell -w
	find . -name '*.go' -type f | xargs goimports -w -local github.com/CosmWasm/wasmvm

.PHONY: lint
lint:
	golangci-lint run
