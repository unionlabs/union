# Compiler versions

In this repository, various different Rust and Go compiler versions are used.
The following explains those compilers and their role.

## Go

This repository is a Go project that is included in the destination project as a
source code import. As such, the Go version used here is never the one building
the final binary. Also no library is built with Go here.

The Go version here has the following goals:

- Serve as a minimal supported version. We keep the range of supported Go
  versions reasonably wide to avoid unnecessary friction for users. I.e. just
  because Cosmos SDK now uses Go 1.19 does not mean we make 1.19 the minimal
  supported version here. However, the project should work with the latest
  stable Go version. When the majority of our users is between 1.18 and 1.19, we
  can slowly remove 1.17 support by bumping the min version to 1.18.
- Be stable enough to test Go code. We always pin the patch version to ensure CI
  runs are reproducible. Those versions will contain security issues from time
  to time, but that's fine for how they are used here.

Go version locations:

1. `go.mod`, e.g. `go 1.17`: The min Go version supported by the source code
2. CI config, e.g. `image: cimg/go:1.17.4`: The min Go version we test
3. Alpine docker, e.g. `FROM golang:1.17.7-alpine`: Used for testing the Go code
   (see `ALPINE_TESTER` in `Makefile`)

## Rust

In contrast to Go, the Rust compiler used here produces actual artifacts used
directly by consumer projects. This are the shared .dylib, .so, .dll libraries
as well as the static .a libraries. Those libwasmvm builds contain all the Rust
code executing contracts, especially cosmwasm-vm. New Rust versions usually add
features which are not necessarily used or needed. But we should move with the
ecosystem to keep the dependency tree compiling. Also new Rust versions tend to
increase runtime speed through optimizer improvements, which never hurts.

While we should update the Rust version for the production builds, there is also
a Rust version which serves as a min version for the project. This is lower to
ensure we can switch between compiler versions in some range.

## Production Rust compiler

This is the version set in the builders: `builders/Dockerfile.alpine`,
`builders/Dockerfile.debian` and `Dockerfile.cross`.

## Min Rust compiler

This is the version used in the CI. It should always be >= the min Rust version
supported by cosmwasm-std/cosmwasm-vm.

## Tooling Rust compiler

This Rust version is not related to our codebase directy. It's sufficiently
modern to install and execute tools like `cargo-audit`.

## Versions in use

We currently use the following version:

| Type                     | Rust version | Note                              |
| ------------------------ | ------------ | --------------------------------- |
| Production Rust compiler | 1.77.0       | Builders version 0019             |
| Min Rust compiler        | 1.74.0       | Supports builder versions >= 0019 |
| Tooling Rust compiler    | 1.75.0       |                                   |
