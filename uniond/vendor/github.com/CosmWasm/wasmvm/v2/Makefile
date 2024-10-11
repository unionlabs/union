.PHONY: all build build-rust build-go test

# Builds the Rust library libwasmvm
BUILDERS_PREFIX := cosmwasm/libwasmvm-builder:0100
# Contains a full Go dev environment including CGO support in order to run Go tests on the built shared library
# This image is currently not published.
ALPINE_TESTER := cosmwasm/alpine-tester:local

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
build-rust-release:
	(cd libwasmvm && cargo build --release)
	cp libwasmvm/target/release/$(SHARED_LIB_SRC) internal/api/$(SHARED_LIB_DST)
	make update-bindings

build-go:
	go build ./...
	go build -o build/demo ./cmd/demo

test:
	# Use package list mode to include all subdirectores. The -count=1 turns off caching.
	RUST_BACKTRACE=1 go test -v -count=1 ./...

test-safety:
	# Use package list mode to include all subdirectores. The -count=1 turns off caching.
	GOEXPERIMENT=cgocheck2 go test -race -v -count=1 ./...

# Run all Go benchmarks
.PHONY: bench
bench:
	go test -bench . -benchtime=2s -run=^Benchmark ./...

# Creates a release build in a containerized build environment of the static library for Alpine Linux (.a)
release-build-alpine:
	# build the muslc *.a file
	docker run --rm -v $(shell pwd)/libwasmvm:/code $(BUILDERS_PREFIX)-alpine
	cp libwasmvm/artifacts/libwasmvm_muslc.x86_64.a internal/api
	cp libwasmvm/artifacts/libwasmvm_muslc.aarch64.a internal/api
	make update-bindings

# Creates a release build in a containerized build environment of the shared library for glibc Linux (.so)
release-build-linux:
	docker run --rm -v $(shell pwd)/libwasmvm:/code $(BUILDERS_PREFIX)-debian build_gnu_x86_64.sh
	docker run --rm -v $(shell pwd)/libwasmvm:/code $(BUILDERS_PREFIX)-debian build_gnu_aarch64.sh
	cp libwasmvm/artifacts/libwasmvm.x86_64.so internal/api
	cp libwasmvm/artifacts/libwasmvm.aarch64.so internal/api
	make update-bindings

# Creates a release build in a containerized build environment of the shared library for macOS (.dylib)
release-build-macos:
	docker run --rm -v $(shell pwd)/libwasmvm:/code $(BUILDERS_PREFIX)-cross build_macos.sh
	cp libwasmvm/artifacts/libwasmvm.dylib internal/api
	make update-bindings

# Creates a release build in a containerized build environment of the static library for macOS (.a)
release-build-macos-static:
	docker run --rm -v $(shell pwd)/libwasmvm:/code $(BUILDERS_PREFIX)-cross build_macos_static.sh
	cp libwasmvm/artifacts/libwasmvmstatic_darwin.a internal/api/libwasmvmstatic_darwin.a
	make update-bindings

# Creates a release build in a containerized build environment of the shared library for Windows (.dll)
release-build-windows:
	docker run --rm -v $(shell pwd)/libwasmvm:/code $(BUILDERS_PREFIX)-cross build_windows.sh
	cp libwasmvm/artifacts/wasmvm.dll internal/api
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

.PHONY: create-tester-image
create-tester-image:
	docker build -t $(ALPINE_TESTER) - < ./Dockerfile.alpine_tester

test-alpine: release-build-alpine create-tester-image
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

.PHONY: lint
lint:
	golangci-lint run

.PHONY: lint-fix
lint-fix:
	golangci-lint run --fix
