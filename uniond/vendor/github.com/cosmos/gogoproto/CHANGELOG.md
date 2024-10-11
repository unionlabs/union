# Changelog

## Unreleased

## [v1.7.0](https://github.com/cosmos/gogoproto/releases/tag/v1.7.0) - 2024-08-12

### Improvements

- [#145](https://github.com/cosmos/gogoproto/pull/145) Remove `x/exp` dependency for greater compatibility.
- [#144](https://github.com/cosmos/gogoproto/pull/144) Change proto.Message and jsonpb.Anyresolver to aliased types to allow different implementations of the same interface.

## [v1.6.0](https://github.com/cosmos/gogoproto/releases/tag/v1.6.0) - 2024-08-08

### Features

- [#142](https://github.com/cosmos/gogoproto/pull/142) Update code generator to make grpc `ServiceDesc` public.

## [v1.5.0](https://github.com/cosmos/gogoproto/releases/tag/v1.5.0) - 2024-06-05

### Improvements

- [#119](https://github.com/cosmos/gogoproto/pull/119) The functions `proto.Marshal`, `proto.Unmarshal`, and `proto.MessageName`, as well as the `jsonpb` marshaling and unmarshaling methods now all support official `google.golang.org/protobuf` types. This allows users to gradually start using these types instead of gogo proto without needing to refactor all of their code with type switch statements.

## [v1.4.12](https://github.com/cosmos/gogoproto/releases/tag/v1.4.12) - 2024-03-20

### Improvements

- [#115](https://github.com/cosmos/gogoproto/pull/115) Move any type from `cosmos-sdk/codec/types` to `types/any`.

## [v1.4.11](https://github.com/cosmos/gogoproto/releases/tag/v1.4.11) - 2023-08-18

### Improvements

- [#83](https://github.com/cosmos/gogoproto/pull/83) Bump `golang.org/x/exp` to latest version. This solves API incompatibilities for users bumping `x/exp` to latest version in their app.

## [v1.4.10](https://github.com/cosmos/gogoproto/releases/tag/v1.4.10) - 2023-05-11

### Improvements

- [#67](https://github.com/cosmos/gogoproto/pull/67) Remove warning about double registration. The Cosmos SDK does purposely double-registration so this warning is not useful.

## [v1.4.9](https://github.com/cosmos/gogoproto/releases/tag/v1.4.9) - 2023-05-03

### Breaking changes

- [#62](https://github.com/cosmos/gogoproto/pull/62) Change public API for `MergedFileDescriptors`, `MergedGlobalFileDescriptors`, etc. introduced in v1.4.8, retracting that release.

### Improvements

- [#62](https://github.com/cosmos/gogoproto/pull/62) Add the `proto.HybridResolver` var  which exposes the merged `*protoregistry.Files` functionality in a high-performance way. Also add the `proto.GogoResolver` var.

## [v1.4.8](https://github.com/cosmos/gogoproto/releases/tag/v1.4.8) - 2023-04-18

**RETRACTED**

### Breaking changes

- [#61](https://github.com/cosmos/gogoproto/pull/60) Use all available cores when merging registries.
  Existing calls to `proto.MergedRegistry()` do not need to change.
  The signature of `proto.MergedFileDescriptors` has changed to accept explicit arguments for the "global files" and "app files".
  Calls to `proto.MergedFileDescriptors()` should change to `proto.MergedGlobalFileDescriptors()` or `proto.MergedGlobalFileDescriptorsWithValidation()`.

### Improvements

- [#59](https://github.com/cosmos/gogoproto/pull/59) Reuse buffers and gzip readers to reduce memory allocations during MergedFileDescriptors.
- [#60](https://github.com/cosmos/gogoproto/pull/60) Skip work to check import path and file descriptor differences during MergedFileDescriptors, when not in debug mode.

## [v1.4.7](https://github.com/cosmos/gogoproto/releases/tag/v1.4.7) - 2023-03-30

### Bug Fixes

- [#55](https://github.com/cosmos/gogoproto/pull/55) Get a file descriptor diff only when debug is enabled.

## [v1.4.6](https://github.com/cosmos/gogoproto/releases/tag/v1.4.6) - 2023-02-21

### Improvements

- [#45](https://github.com/cosmos/gogoproto/pull/45) Remove the StdErr warning about file descriptor mismatches, which was deemed unclear and too invasive.

## [v1.4.5](https://github.com/cosmos/gogoproto/releases/tag/v1.4.5) - 2023-02-20

### Improvements

- [#43](https://github.com/cosmos/gogoproto/pull/43) Relax runtime linter checks introduced in #37: instead of throwing an error, simply log a warning to StdErr. Also provide a helper function `DebugFileDescriptorsMismatch` to debug these errors.
- [#37](https://github.com/cosmos/gogoproto/pull/37) Add `MergedFileDescriptors` and `MergedRegistry` to retrieve a registry with merged file descriptors from both gogo and protoregistry.

### Bug Fixes

- [#34](https://github.com/cosmos/gogoproto/pull/34) Allow empty package name, as per gogo original behavior. Fix regression introduced in v1.4.4

## [v1.4.4](https://github.com/cosmos/gogoproto/releases/tag/v1.4.4) - 2023-01-30

### Improvements

- [#32](https://github.com/cosmos/gogoproto/pull/32) The prtoc-gen-gogo generator requires that all proto files import paths match their fully-qualified package name.

## [v1.4.3](https://github.com/cosmos/gogoproto/releases/tag/v1.4.3) - 2022-10-14

### Bug Fixes

- [#24](https://github.com/cosmos/gogoproto/pull/24) Fix `CompactTextString` panics with nested Anys and private fields.
- [#14](https://github.com/cosmos/gogoproto/pull/14) Fix `make regenerate`.

## [v1.4.2](https://github.com/cosmos/gogoproto/releases/tag/v1.4.2) - 2022-09-14

### Features

- [#13](https://github.com/cosmos/gogoproto/pull/13) Add `AllFileDescriptors` function.

### Improvements

- [#8](https://github.com/cosmos/gogoproto/pull/8) Fix typo in `doc.go`.
- [#8](https://github.com/cosmos/gogoproto/pull/8) Support for merging messages implementing Merger which are embedded by value.
- [#8](https://github.com/cosmos/gogoproto/pull/8) Use reflect.Value.String() for String kinds in proto equal.

## [v1.4.1](https://github.com/cosmos/gogoproto/releases/tag/v1.4.1) - 2022-08-30

### Improvements

- [#6](https://github.com/cosmos/gogoproto/pull/6) Add buf.yaml for cosmos/gogo-proto module.

### Bug Fixes

- [226206f](https://github.com/cosmos/gogoproto/commit/226206f39bd7276e88ec684ea0028c18ec2c91ae) Fixed order of imports, make stable generation result.

## [v1.4.0](https://github.com/cosmos/gogoproto/releases/tag/v1.4.0) - 2022-03-18

- Migration from [regen-network/protobuf](https://github.com/regen-network/protobuf), a fork of [gogo/protobuf](https://github.com/gogo/protobuf) used by the Cosmos SDK to [cosmos/gogoproto](https://github.com/cosmos/gogoproto) (this repository).
