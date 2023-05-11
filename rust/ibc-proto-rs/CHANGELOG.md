# CHANGELOG

## v0.29.0

*April 12th, 2023*

In this update, Protobuf definitions have been included for Interchain Security v1 CCV within
the `ibc_proto::interchain_security::ccv` module.

It should also be noted that the return type of `Protobuf::encode{,_length_delimited}_vec`
has been modified from `Result<Vec<u8>, Error>` to `Vec<u8>`.

Furthermore, the version of `tonic` has been raised from 0.8 to 0.9.

### BREAKING CHANGES

- Remove errors for `encode_vec` and `encode_length_delimited_vec` in
  `Protobuf` ([#73](https://github.com/cosmos/ibc-proto-rs/issues/73))
- Update `tonic` to 0.9 and re-generate the protos
  ([\#79](https://github.com/cosmos/ibc-proto-rs/issues/79))

### FEATURES

- Add Interchain Security v1 CCV Protobuf definitions
  ([\#76](https://github.com/cosmos/ibc-proto-rs/issues/76))

### BUG FIXES

- Automatically patch the generated Rust code for it to compile
  ([\#2](https://github.com/cosmos/ibc-proto-rs/issues/2))

## v0.28.0

*March 10th, 2023*

This release updates the `ibc-go` proto files from version `v5.0.0` to `v5.1.0`.

This includes the `memo` field in the following struct:

* `ibc.applications.transfer.v1 MsgTransfer`
* `ibc.applications.transfer.v2 FungibleTokenPacketData`

As well as the `sequence` field in:

* `ibc.applications.transfer.v1 MsgTransferResponse`

### IMPROVEMENTS

- Update `ibc-go` commit from `v5.0.0` to `v5.1.0`
([#71](https://github.com/cosmos/ibc-proto-rs/issues/71))

## v0.27.0

*March 7th, 2023*

This release updates the `tendermint-proto` crate to v0.30.0.

At the moment, only the Tendermint Protobuf definitions for CometBFT 0.37 are exported
and supported. In the future, side-by-side support for 0.34 and 0.37 definitions may be provided.

### BREAKING CHANGE

- Update `tendermint-proto` to v0.30.0 ([#64](https://github.com/cosmos/ibc-proto-rs/issues/64))

## v0.26.0

*February 17, 2023*

This release updates tendermint protobuf defintions to `v0.29.0`.

## v0.25.0

*February 9th, 2023*

This release updates borsh to v0.10.0 and fixes a typo in borsh deserialization of `Any`
([#59](https://github.com/cosmos/ibc-proto-rs/pull/59)).

## v0.24.1

*January 10th, 2023*

This release adds `parity-scale-codec` and `borsh` serialize/deserialize for the `Any` type.

### FEATURES

- Add parity-scale-codec and borsh for Any ([#47](https://github.com/cosmos/ibc-
  proto-rs/issues/47))

## v0.24.0

*December 13th, 2022*

This release updates the Tendermint Protobuf definitons to v0.28.0.

### BREAKING CHANGES

- Update to tendermint-proto 0.28 ([#45](https://github.com/cosmos/ibc-proto-rs/issues/45))

## v0.23.0

*November 29th, 2022*

This release updates the Tendermint Protobuf definitons to v0.27.0.

### BREAKING CHANGES

- Update to tendermint-proto 0.27 ([#40](https://github.com/cosmos/ibc-proto-rs/pull/40))

## v0.22.0

*November 9, 2022*

This release updates the Cosmos SDK protobufs to v0.46.4.

### BREAKING CHANGES

- Update tendermint-rs libraries to v0.26
  ([#33](https://github.com/cosmos/ibc-proto-rs/issues/33))
- Update protobufs for Cosmos SDK to v0.46.4
  - Adds the `module_account_by_name` method to the `Query` trait
  ([#2776](https://github.com/informalsystems/hermes/2776))

## v0.21.0

*October 19, 2022*

This is the first release of ibc-proto with its own changelog. For past releases, please check the [Hermes](https://github.com/informalsystems/hermes/blob/c34b354e310da7f59631ae315ea22c5f2b420d44/CHANGELOG.md) changelog.

### BREAKING CHANGES

- Update protos to IBC-Go v5.0.0 and Cosmos SDK v0.46.1
  ([#24](https://github.com/cosmos/ibc-proto-rs/issues/24))
- Update tendermint-proto requirement from =0.23.9 to =0.25.0
  ([#26](https://github.com/cosmos/ibc-proto-rs/issues/26))
