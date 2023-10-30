# Available Feature Flags

This crate has the following features which can be enabled.
Each entry will explain the feature in more detail.

`serde_with` is fully `no_std` compatible.
Using it in a `no_std` environment requires using `default-features = false`, since `std` is enabled by default.

1. [`alloc`](#alloc)
2. [`std` (default)](#std-default)
3. [`base64`](#base64)
4. [`chrono_0_4`](#chrono_0_4)
5. [`guide`](#guide)
6. [`hex`](#hex)
7. [`indexmap_1`](#indexmap_1)
8. [`json`](#json)
9. [`macros` (default)](#macros-default)
10. [`time_0_3`](#time_0_3)

## `alloc`

Enable support for types from the `alloc` crate when running in a `no_std` environment.

## `std` (default)

Enables support for various types from the std library.
This will enable `std` support in all dependencies too.
The feature enabled by default and also enables `alloc`.

## `base64`

The `base64` feature enables serializing data in base64 format.

This pulls in `base64` as a dependency.
It enables the `alloc` feature.

## `chrono_0_4`

The `chrono_0_4` feature enables integration of `chrono_0_4` specific conversions.
This includes support for the timestamp and duration types.
More features are available in combination with `alloc` or `std`.
The legacy feature name `chrono` is still available for v1 compatability.

This pulls in `chrono` v0.4 as a dependency.

## `guide`

The `guide` feature enables inclusion of this user guide.
The feature only changes the rustdoc output and enables no other effects.

## `hex`

The `hex` feature enables serializing data in hex format.

This pulls in `hex` as a dependency.
It enables the `alloc` feature.

## `indexmap_1`

The `indexmap_1` feature enables implementations of `indexmap_1` specific checks.
This includes support for checking duplicate keys and duplicate values.
The legacy feature name `indexmap` is still available for v1 compatability.

This pulls in `indexmap` as a dependency.
It enables the `alloc` feature.
Some functionality is only available when `std` is enabled too.

## `json`

The `json` features enables JSON conversions from the `json` module.

This pulls in `serde_json` as a dependency.
It enables the `alloc` feature.

## `macros` (default)

The `macros` features enables all helper macros and derives.
It is enabled by default, since the macros provide a usability benefit, especially for `serde_as`.

This pulls in `serde_with_macros` as a dependency.

## `time_0_3`

The `time_0_3` enables integration of `time` v0.3 specific conversions.
This includes support for the timestamp and duration types.

This pulls in `time` v0.3 as a dependency.
Some functionality is only available when `alloc` or `std` is enabled too.
