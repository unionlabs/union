# Cross Compilation Scripts

As this library is targetting go developers, we cannot assume a properly set up
rust environment on their system. Further, when importing this library, there is
no clean way to add a `libwasmvm.{so,dll,dylib}`. It needs to be committed with
the tagged (go) release in order to be easily usable.

The solution is to precompile the rust code into libraries for the major
platforms (Linux, Windows, MacOS) and commit them to the repository at each
tagged release. This should be doable from one host machine, but is a bit
tricky. This folder contains build scripts and a Docker image to create all
dynamic libraries from one host. In general this is set up for a Linux host, but
any machine that can run Docker can do the cross-compilation.

## Docker Hub images

See those DockerHub repos for all available versions of the builder images.

- From version 0100: https://hub.docker.com/r/cosmwasm/libwasmvm-builder/tags
- Before version 0100: https://hub.docker.com/r/cosmwasm/go-ext-builder/tags

## Changelog

**Unreleased**

**Version 0100:**

- Rename builder image from cosmwasm/go-ext-builder to
  cosmwasm/libwasmvm-builder
- Replace CentOS with Debian image for GNU linux builds
- Avoid using a target folder in the host system. Instead the folder /target in
  the guest is used. Due to this change we can now drop the argument
  `-u $(USER_ID):$(USER_GROUP)` when using builders. ([#437])
- Build all images with `--platform=linux/amd64` to avoid accidental ARM builds

[#437]: https://github.com/CosmWasm/wasmvm/issues/437

**Version 0019:**

- Bump `OSX_VERSION_MIN` to 10.15.
- Update Rust to 1.77.0.

**Version 0018:**

- Remove Go dev environment from `cosmwasm/go-ext-builder:XXXX-alpine`
- Write x86_64 muslc output in `libwasmvm_muslc.x86_64.a` instead of
  `libwasmvm_muslc.a`

**Version 0017:**

- Update Rust to 1.73.0.
- Update Go to 1.20.10 (for testing only).

**Version 0016:**

- Update Rust to 1.69.0.
- Let `build_muslc.sh` use `--example wasmvmstatic` instead of `--example muslc`

**Version 0015:**

- Update Rust to 1.68.2.
- Update Go (for testing only) to 1.19.7.
- Add `build_macos_static.sh` to cross builders for macOS build.

**Version 0014:**

- Update Rust to 1.65.0.
- Update Go (for testing only) to 1.18.8.

**Version 0013:**

- Update Rust to 1.63.0 in `Dockerfile.alpine` and `Dockerfile.cross`;
  `Dockerfile.centos7` was accidentally not updated and remained on 1.60.0
  ([#350]).
- Add Windows support to cosmwasm/go-ext-builder:0013-cross. This image builds
  for macOS and Windows now.

[#350]: https://github.com/CosmWasm/wasmvm/pull/350

**Version 0012:**

- Add cross-compilation setup to build `libwasmvm.x86_64.so` and
  `libwasmvm.aarch64.so` from the CentOS builder image.
- Update Rust to 1.60.0.

**Version 0011:**

- Update Rust to 1.59.0.

**Version 0010:**

- Add cross-compilation setup to build `libwasmvm_muslc.a` and
  `libwasmvm_muslc.aarch64.a` from the alpine builder image.

**Version 0009:**

- Let macOS build dylib files with both aarch64 and x86_64 code.
- Update Go (for testing only) to 1.17.7.

**Version 0008:**

- Update Rust to 1.55.0 and Go (for testing only) to 1.17.5.

**Version 0007:**

- Do not copy output from the target folder to final destination. The caller
  should do that.
- Update Rust to 1.53.0.

**Version 0006:**

- Update Rust to 1.51.0.

**Version 0005:**

- Update Rust to 1.50.0.

**Version 0004:**

- Update Rust to 1.49.0.
- Alpine: Update Go to 1.15

**Version 0003:**

- Avoid pre-fetching of dependences to decouple builders from source code.
- Bump `OSX_VERSION_MIN` to 10.10.
- Use `rust:1.47.0-buster` as base image for cross compilation to macOS

**Version 0002:**

- Update hardcoded library name from `libgo_cosmwasm` to `libwasmvm`.

**Version 0001:**

- First release of builders that is versioned separately of CosmWasm.
- Update Rust to nightly-2020-10-24.

## Usage

Create the Docker images, capable of cross-compling Linux and MacOS dynamic
libs. As the builder images are all x86_64, it can be slow and memory intense to
do this on a different architecture:

```sh
(cd builders && make docker-images)
```

Then in the repo root, `make release-build` will use the above docker image and
copy the generated `{so,dylib}` files into `internal/api` directory to be
linked.

## Future Work

- Add support for cross-compiling to Windows as well.
- Publish docker images when they are stable
