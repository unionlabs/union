# Cosmos SDK v0.52.0-beta.2 Release Notes

There are no release notes for pre-releases.

This is the first beta of the Cosmos SDK v0.52.0 release since the internal audit has been completed.
When integrating, use the latest pseudo version from the release branch (`release/v0.52.x`) for SDK modules or main for other packages. Cosmos SDK modules won't be tagged before RC. Refer to the [version matrix](https://github.com/cosmos/cosmos-sdk?tab=readme-ov-file#version-matrix) to understand what it means.
Note, this beta contains some dependencies annoyance that will be resolved before the RC (i.e. the SDK is temporarily importing `bank` and `staking`).

Please see the [CHANGELOG](https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/CHANGELOG.md) for an exhaustive list of changes.  
Refer to the [UPGRADING.md](https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/UPGRADING.md) for upgrading your application.

Full Commit History: https://github.com/cosmos/cosmos-sdk/compare/release/v0.50.x...release/v0.52.x
